#include "UserController.h"
#include "db.h"
#include "helpers.h"
#include <drogon/HttpResponse.h>
#include <nlohmann/json.hpp>
#include <memory>
#include <string>
#include <vector>

// ---------------------------------------------------------------------------
// Shared helpers (local to translation unit)
// ---------------------------------------------------------------------------

static drogon::HttpResponsePtr user_json_resp(drogon::HttpStatusCode code,
                                              const std::string& body) {
    auto r = drogon::HttpResponse::newHttpResponse();
    r->setStatusCode(code);
    r->setContentTypeCode(drogon::CT_APPLICATION_JSON);
    r->setBody(body);
    return r;
}

static drogon::HttpResponsePtr user_error(drogon::HttpStatusCode code,
                                          const std::string& msg) {
    nlohmann::json j;
    j["message"] = msg;
    return user_json_resp(code, j.dump());
}

static void user_db_err(const drogon::orm::DrogonDbException& e,
                        std::function<void(const drogon::HttpResponsePtr&)> cb) {
    (void)e;
    cb(user_error(drogon::k500InternalServerError, "internal server error"));
}

static User user_from_row(const drogon::orm::Row& row) {
    auto opt_str = [&](const char* col) -> std::optional<std::string> {
        if (row[col].isNull()) return std::nullopt;
        return row[col].as<std::string>();
    };
    auto opt_int64 = [&](const char* col) -> std::optional<int64_t> {
        if (row[col].isNull()) return std::nullopt;
        return row[col].as<int64_t>();
    };
    auto opt_int32 = [&](const char* col) -> std::optional<int32_t> {
        if (row[col].isNull()) return std::nullopt;
        return row[col].as<int32_t>();
    };
    return row_to_user(
        opt_int64("id"),
        opt_str("username"),
        opt_str("first_name"),
        opt_str("last_name"),
        opt_str("email"),
        opt_str("password"),
        opt_str("phone"),
        opt_int32("user_status")
    );
}

// Upsert a single user; replies on cb.
static void upsert_user(drogon::orm::DbClientPtr db,
                        User user,
                        std::function<void(const drogon::HttpResponsePtr&)> cb) {
    if (!user.username || user.username->empty()) {
        cb(user_error(drogon::k400BadRequest, "username required")); return;
    }

    auto do_insert = [db, user, cb](int64_t uid) mutable {
        std::string username  = user.username.value_or("");
        std::string firstName = user.firstName.value_or("");
        std::string lastName  = user.lastName.value_or("");
        std::string email     = user.email.value_or("");
        std::string password  = user.password.value_or("");
        std::string phone     = user.phone.value_or("");
        int32_t     status    = user.userStatus.value_or(0);

        db->execSqlAsync(
            "INSERT INTO \"user\" (id, username, first_name, last_name, email, password, phone, user_status) "
            "VALUES ($1, $2, $3, $4, $5, $6, $7, $8) "
            "ON CONFLICT (username) DO UPDATE SET "
            "id=EXCLUDED.id, first_name=EXCLUDED.first_name, last_name=EXCLUDED.last_name, "
            "email=EXCLUDED.email, password=EXCLUDED.password, phone=EXCLUDED.phone, "
            "user_status=EXCLUDED.user_status "
            "RETURNING id, username, first_name, last_name, email, password, phone, user_status",
            [cb](const drogon::orm::Result& r) {
                cb(user_json_resp(drogon::k200OK, user_to_json(user_from_row(r[0]))));
            },
            [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
            uid, username, firstName, lastName, email, password, phone, status
        );
    };

    if (user.id) {
        do_insert(*user.id);
    } else {
        db->execSqlAsync(
            "SELECT nextval('user_id_seq') AS id",
            [do_insert = std::move(do_insert)](const drogon::orm::Result& r) mutable {
                do_insert(r[0]["id"].as<int64_t>());
            },
            [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); }
        );
    }
}

// ---------------------------------------------------------------------------
// Bulk-insert helper: insert users sequentially, replying with the last one.
// Uses a shared struct + free function to avoid self-capturing lambdas.
// ---------------------------------------------------------------------------

struct BulkInsertState {
    std::vector<User> users;
    size_t idx = 0;
    User last;
    drogon::orm::DbClientPtr db;
    std::function<void(const drogon::HttpResponsePtr&)> cb;
};

static void bulk_insert_step(std::shared_ptr<BulkInsertState> state) {
    if (state->idx >= state->users.size()) {
        state->cb(user_json_resp(drogon::k200OK, user_to_json(state->last)));
        return;
    }
    User u = state->users[state->idx++];
    upsert_user(state->db, u, [state](const drogon::HttpResponsePtr& resp) {
        if (resp->getStatusCode() != drogon::k200OK) {
            state->cb(resp); return;
        }
        try {
            auto j = nlohmann::json::parse(resp->getBody());
            from_json(j, state->last);
        } catch (...) {}
        bulk_insert_step(state);
    });
}

// ---------------------------------------------------------------------------
// createUser  POST /api/v3/user
// ---------------------------------------------------------------------------
void UserController::createUser(const drogon::HttpRequestPtr& req,
                                std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(user_error(drogon::k400BadRequest, "invalid JSON")); return; }

    User user;
    try { from_json(body, user); }
    catch (...) { cb(user_error(drogon::k400BadRequest, "invalid input")); return; }

    upsert_user(getDb(), user, std::move(cb));
}

// ---------------------------------------------------------------------------
// createUsersWithList  POST /api/v3/user/createWithList
// ---------------------------------------------------------------------------
void UserController::createUsersWithList(const drogon::HttpRequestPtr& req,
                                         std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(user_error(drogon::k400BadRequest, "invalid JSON")); return; }

    if (!body.is_array()) { cb(user_error(drogon::k400BadRequest, "expected array")); return; }

    std::vector<User> users;
    try {
        for (const auto& item : body) { User u; from_json(item, u); users.push_back(u); }
    } catch (...) { cb(user_error(drogon::k400BadRequest, "invalid input")); return; }

    if (users.empty()) { cb(drogon::HttpResponse::newHttpResponse()); return; }

    auto state = std::make_shared<BulkInsertState>();
    state->users = std::move(users);
    state->db    = getDb();
    state->cb    = std::move(cb);
    bulk_insert_step(state);
}

// ---------------------------------------------------------------------------
// loginUser  GET /api/v3/user/login
// ---------------------------------------------------------------------------
void UserController::loginUser(const drogon::HttpRequestPtr& req,
                               std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    std::string username = req->getParameter("username");
    std::string password = req->getParameter("password");

    auto db = getDb();
    db->execSqlAsync(
        "SELECT EXISTS(SELECT 1 FROM \"user\" WHERE username=$1 AND password=$2) AS ok",
        [cb, username](const drogon::orm::Result& r) {
            bool ok = r[0]["ok"].as<bool>();
            if (!ok) { cb(user_error(drogon::k400BadRequest, "Invalid username/password")); return; }
            nlohmann::json j;
            j["message"] = "logged in user session: " + username;
            cb(user_json_resp(drogon::k200OK, j.dump()));
        },
        [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
        username, password
    );
}

// ---------------------------------------------------------------------------
// logoutUser  GET /api/v3/user/logout  (stateless no-op)
// ---------------------------------------------------------------------------
void UserController::logoutUser(const drogon::HttpRequestPtr&,
                                std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    cb(drogon::HttpResponse::newHttpResponse()); // 200
}

// ---------------------------------------------------------------------------
// getUserByName  GET /api/v3/user/{username}
// ---------------------------------------------------------------------------
void UserController::getUserByName(const drogon::HttpRequestPtr&,
                                   std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                   const std::string& username) {
    auto db = getDb();
    db->execSqlAsync(
        "SELECT id, username, first_name, last_name, email, password, phone, user_status "
        "FROM \"user\" WHERE username=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.empty()) { cb(user_error(drogon::k404NotFound, "User not found")); return; }
            cb(user_json_resp(drogon::k200OK, user_to_json(user_from_row(r[0]))));
        },
        [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
        username
    );
}

// ---------------------------------------------------------------------------
// updateUser  PUT /api/v3/user/{username}
// ---------------------------------------------------------------------------
void UserController::updateUser(const drogon::HttpRequestPtr& req,
                                std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                const std::string& username) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(user_error(drogon::k400BadRequest, "invalid JSON")); return; }

    User user;
    try { from_json(body, user); }
    catch (...) { cb(user_error(drogon::k400BadRequest, "invalid input")); return; }

    auto db = getDb();
    db->execSqlAsync(
        "SELECT id FROM \"user\" WHERE username=$1",
        [db, user, username, cb = std::move(cb)](const drogon::orm::Result& r) mutable {
            if (r.empty()) { cb(user_error(drogon::k404NotFound, "User not found")); return; }

            int64_t     existing_id   = r[0]["id"].as<int64_t>();
            int64_t     uid           = user.id.value_or(existing_id);
            std::string new_username  = user.username.value_or(username);
            std::string firstName     = user.firstName.value_or("");
            std::string lastName      = user.lastName.value_or("");
            std::string email         = user.email.value_or("");
            std::string password      = user.password.value_or("");
            std::string phone         = user.phone.value_or("");
            int32_t     status        = user.userStatus.value_or(0);

            db->execSqlAsync(
                "UPDATE \"user\" SET id=$1, username=$2, first_name=$3, last_name=$4, "
                "email=$5, password=$6, phone=$7, user_status=$8 WHERE username=$9",
                [cb](const drogon::orm::Result&) {
                    cb(drogon::HttpResponse::newHttpResponse());
                },
                [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
                uid, new_username, firstName, lastName, email, password, phone, status, username
            );
        },
        [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
        username
    );
}

// ---------------------------------------------------------------------------
// deleteUser  DELETE /api/v3/user/{username}
// ---------------------------------------------------------------------------
void UserController::deleteUser(const drogon::HttpRequestPtr&,
                                std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                const std::string& username) {
    auto db = getDb();
    db->execSqlAsync(
        "DELETE FROM \"user\" WHERE username=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.affectedRows() == 0) { cb(user_error(drogon::k404NotFound, "User not found")); return; }
            cb(drogon::HttpResponse::newHttpResponse());
        },
        [cb](const drogon::orm::DrogonDbException& e) { user_db_err(e, cb); },
        username
    );
}
