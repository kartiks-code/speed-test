#include "PetController.h"
#include "db.h"
#include "helpers.h"
#include <drogon/HttpResponse.h>
#include <nlohmann/json.hpp>
#include <sstream>
#include <string>
#include <vector>

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

static drogon::HttpResponsePtr json_resp(drogon::HttpStatusCode code,
                                         const std::string& body) {
    auto r = drogon::HttpResponse::newHttpResponse();
    r->setStatusCode(code);
    r->setContentTypeCode(drogon::CT_APPLICATION_JSON);
    r->setBody(body);
    return r;
}

static drogon::HttpResponsePtr error_resp(drogon::HttpStatusCode code,
                                          const std::string& msg) {
    nlohmann::json j;
    j["message"] = msg;
    return json_resp(code, j.dump());
}

static drogon::HttpResponsePtr internal_error() {
    return error_resp(drogon::k500InternalServerError, "internal server error");
}

static void db_err(const drogon::orm::DrogonDbException& e,
                   std::function<void(const drogon::HttpResponsePtr&)> cb) {
    (void)e;
    cb(internal_error());
}

// Build Pet from a Drogon DB result row
static Pet pet_from_row(const drogon::orm::Row& row) {
    auto opt_str = [&](const char* col) -> std::optional<std::string> {
        if (row[col].isNull()) return std::nullopt;
        return row[col].as<std::string>();
    };
    std::optional<int64_t> id;
    if (!row["id"].isNull()) id = row["id"].as<int64_t>();
    std::string name = row["name"].as<std::string>();
    std::string photo_urls = row["photo_urls"].isNull() ? "[]" : row["photo_urls"].as<std::string>();
    return row_to_pet(id, name, opt_str("category"), photo_urls, opt_str("tags"), opt_str("status"));
}

// Collect query-param list (handles repeated key=v1&key=v2 and comma-separated values).
// Drogon's getParameters() is an unordered_map so we parse the raw query string.
static std::vector<std::string> query_list(const drogon::HttpRequestPtr& req,
                                           const std::string& key) {
    std::vector<std::string> result;
    std::string query = req->getQuery();
    std::istringstream ss(query);
    std::string pair;
    while (std::getline(ss, pair, '&')) {
        auto eq = pair.find('=');
        if (eq == std::string::npos) continue;
        if (pair.substr(0, eq) != key) continue;
        std::string val = pair.substr(eq + 1);
        std::istringstream vs(val);
        std::string token;
        while (std::getline(vs, token, ',')) {
            if (!token.empty()) result.push_back(token);
        }
    }
    return result;
}

// Build a parameterised IN-clause placeholder list: ($1,$2,...)
static std::string placeholders(int count, int start = 1) {
    std::string s = "(";
    for (int i = 0; i < count; ++i) {
        if (i > 0) s += ',';
        s += '$';
        s += std::to_string(start + i);
    }
    s += ')';
    return s;
}

// ---------------------------------------------------------------------------
// addPet  POST /api/v3/pet
// ---------------------------------------------------------------------------
void PetController::addPet(const drogon::HttpRequestPtr& req,
                           std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(error_resp(drogon::k400BadRequest, "invalid JSON")); return; }

    Pet pet;
    try { from_json(body, pet); } catch (...) { cb(error_resp(drogon::k400BadRequest, "invalid input")); return; }

    if (pet.name.empty() || pet.photoUrls.empty()) {
        cb(error_resp(drogon::k405MethodNotAllowed, "Invalid input")); return;
    }

    auto db = getDb();

    if (pet.id) {
        // ID provided — go straight to upsert
        nlohmann::json jcat = pet.category ? nlohmann::json(*pet.category) : nlohmann::json(nullptr);
        nlohmann::json jpu  = pet.photoUrls;
        nlohmann::json jtags= pet.tags ? nlohmann::json(*pet.tags) : nlohmann::json::array();
        std::string status  = pet.status.value_or("");

        db->execSqlAsync(
            "INSERT INTO pet (id, name, category, photo_urls, tags, status) "
            "VALUES ($1, $2, $3::json, $4::json, $5::json, NULLIF($6,'')::pet_status) "
            "ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, category=EXCLUDED.category, "
            "photo_urls=EXCLUDED.photo_urls, tags=EXCLUDED.tags, status=EXCLUDED.status "
            "RETURNING id, name, category, photo_urls, tags, status::text",
            [cb](const drogon::orm::Result& r) {
                auto p = pet_from_row(r[0]);
                cb(json_resp(drogon::k200OK, pet_to_json(p)));
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            *pet.id, pet.name, jcat.dump(), jpu.dump(), jtags.dump(), status
        );
    } else {
        // Need to fetch nextval first
        db->execSqlAsync(
            "SELECT nextval('pet_id_seq') AS id",
            [pet, db, cb = std::move(cb)](const drogon::orm::Result& r) mutable {
                int64_t new_id = r[0]["id"].as<int64_t>();
                nlohmann::json jcat = pet.category ? nlohmann::json(*pet.category) : nlohmann::json(nullptr);
                nlohmann::json jpu  = pet.photoUrls;
                nlohmann::json jtags= pet.tags ? nlohmann::json(*pet.tags) : nlohmann::json::array();
                std::string status  = pet.status.value_or("");

                db->execSqlAsync(
                    "INSERT INTO pet (id, name, category, photo_urls, tags, status) "
                    "VALUES ($1, $2, $3::json, $4::json, $5::json, NULLIF($6,'')::pet_status) "
                    "ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, category=EXCLUDED.category, "
                    "photo_urls=EXCLUDED.photo_urls, tags=EXCLUDED.tags, status=EXCLUDED.status "
                    "RETURNING id, name, category, photo_urls, tags, status::text",
                    [cb](const drogon::orm::Result& res) {
                        auto p = pet_from_row(res[0]);
                        cb(json_resp(drogon::k200OK, pet_to_json(p)));
                    },
                    [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
                    new_id, pet.name, jcat.dump(), jpu.dump(), jtags.dump(), status
                );
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); }
        );
    }
}

// ---------------------------------------------------------------------------
// updatePet  PUT /api/v3/pet
// ---------------------------------------------------------------------------
void PetController::updatePet(const drogon::HttpRequestPtr& req,
                              std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(error_resp(drogon::k400BadRequest, "invalid JSON")); return; }

    Pet pet;
    try { from_json(body, pet); } catch (...) { cb(error_resp(drogon::k400BadRequest, "invalid input")); return; }

    if (!pet.id) { cb(error_resp(drogon::k400BadRequest, "id required")); return; }
    if (pet.name.empty() || pet.photoUrls.empty()) {
        cb(error_resp(drogon::k405MethodNotAllowed, "Invalid input")); return;
    }

    auto db = getDb();
    // Verify pet exists first
    db->execSqlAsync(
        "SELECT 1 FROM pet WHERE id=$1",
        [pet, db, cb = std::move(cb)](const drogon::orm::Result& r) mutable {
            if (r.empty()) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }

            nlohmann::json jcat = pet.category ? nlohmann::json(*pet.category) : nlohmann::json(nullptr);
            nlohmann::json jpu  = pet.photoUrls;
            nlohmann::json jtags= pet.tags ? nlohmann::json(*pet.tags) : nlohmann::json::array();
            std::string status  = pet.status.value_or("");

            db->execSqlAsync(
                "INSERT INTO pet (id, name, category, photo_urls, tags, status) "
                "VALUES ($1, $2, $3::json, $4::json, $5::json, NULLIF($6,'')::pet_status) "
                "ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, category=EXCLUDED.category, "
                "photo_urls=EXCLUDED.photo_urls, tags=EXCLUDED.tags, status=EXCLUDED.status "
                "RETURNING id, name, category, photo_urls, tags, status::text",
                [cb](const drogon::orm::Result& res) {
                    auto p = pet_from_row(res[0]);
                    cb(json_resp(drogon::k200OK, pet_to_json(p)));
                },
                [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
                *pet.id, pet.name, jcat.dump(), jpu.dump(), jtags.dump(), status
            );
        },
        [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
        *pet.id
    );
}

// ---------------------------------------------------------------------------
// findByStatus  GET /api/v3/pet/findByStatus
// ---------------------------------------------------------------------------
void PetController::findByStatus(const drogon::HttpRequestPtr& req,
                                 std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    auto statuses = query_list(req, "status");
    if (statuses.empty()) statuses = {"available"};

    for (const auto& s : statuses) {
        if (s != "available" && s != "pending" && s != "sold") {
            cb(error_resp(drogon::k400BadRequest, "invalid status")); return;
        }
    }

    std::string sql = "SELECT id, name, category, photo_urls, tags, status::text FROM pet "
                      "WHERE status::text IN " + placeholders(static_cast<int>(statuses.size())) +
                      " ORDER BY id";

    auto db = getDb();
    // Build a lambda capturing statuses
    auto capture = [cb](const drogon::orm::Result& r) {
        nlohmann::json arr = nlohmann::json::array();
        for (const auto& row : r) {
            nlohmann::json j;
            to_json(j, pet_from_row(row));
            arr.push_back(j);
        }
        cb(json_resp(drogon::k200OK, arr.dump()));
    };

    if (statuses.size() == 1) {
        db->execSqlAsync(sql, capture,
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            statuses[0]);
    } else if (statuses.size() == 2) {
        db->execSqlAsync(sql, capture,
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            statuses[0], statuses[1]);
    } else {
        db->execSqlAsync(sql, capture,
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            statuses[0], statuses[1], statuses[2]);
    }
}

// ---------------------------------------------------------------------------
// findByTags  GET /api/v3/pet/findByTags
// ---------------------------------------------------------------------------
void PetController::findByTags(const drogon::HttpRequestPtr& req,
                               std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    auto tags = query_list(req, "tags");
    if (tags.empty()) {
        cb(json_resp(drogon::k200OK, "[]")); return;
    }

    // Pass tags as a JSON array string; use json_array_elements_text to do the IN check.
    // This avoids variable-arity parameter binding.
    nlohmann::json jtags = tags;
    std::string tags_json = jtags.dump();

    static const char* sql =
        "SELECT id, name, category, photo_urls, tags, status::text FROM pet "
        "WHERE EXISTS ("
        "  SELECT 1 FROM json_array_elements(COALESCE(tags,'[]'::json)) elem,"
        "               json_array_elements_text($1::json) t"
        "  WHERE elem->>'name' = t.value"
        ") ORDER BY id";

    auto db = getDb();
    db->execSqlAsync(
        sql,
        [cb](const drogon::orm::Result& r) {
            nlohmann::json arr = nlohmann::json::array();
            for (const auto& row : r) {
                nlohmann::json j;
                to_json(j, pet_from_row(row));
                arr.push_back(j);
            }
            cb(json_resp(drogon::k200OK, arr.dump()));
        },
        [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
        tags_json
    );
}

// ---------------------------------------------------------------------------
// getPetById  GET /api/v3/pet/{id}
// ---------------------------------------------------------------------------
void PetController::getPetById(const drogon::HttpRequestPtr&,
                               std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                               int64_t id) {
    auto db = getDb();
    db->execSqlAsync(
        "SELECT id, name, category, photo_urls, tags, status::text FROM pet WHERE id=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.empty()) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
            cb(json_resp(drogon::k200OK, pet_to_json(pet_from_row(r[0]))));
        },
        [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
        id
    );
}

// ---------------------------------------------------------------------------
// updateWithForm  POST /api/v3/pet/{id}
// ---------------------------------------------------------------------------
void PetController::updateWithForm(const drogon::HttpRequestPtr& req,
                                   std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                   int64_t id) {
    std::string name   = req->getParameter("name");
    std::string status = req->getParameter("status");

    if (name.empty() && status.empty()) {
        // No-op: just confirm pet exists
        auto db = getDb();
        db->execSqlAsync(
            "SELECT 1 FROM pet WHERE id=$1",
            [cb, id](const drogon::orm::Result& r) {
                if (r.empty()) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
                cb(drogon::HttpResponse::newHttpResponse()); // 200 empty
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            id
        );
        return;
    }

    if (!status.empty() && status != "available" && status != "pending" && status != "sold") {
        cb(error_resp(drogon::k400BadRequest, "invalid status")); return;
    }

    std::string sql;
    auto db = getDb();
    if (!name.empty() && !status.empty()) {
        sql = "UPDATE pet SET name=$1, status=NULLIF($2,'')::pet_status WHERE id=$3";
        db->execSqlAsync(sql,
            [cb, id](const drogon::orm::Result& r) {
                if (r.affectedRows() == 0) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
                cb(drogon::HttpResponse::newHttpResponse());
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            name, status, id
        );
    } else if (!name.empty()) {
        sql = "UPDATE pet SET name=$1 WHERE id=$2";
        db->execSqlAsync(sql,
            [cb](const drogon::orm::Result& r) {
                if (r.affectedRows() == 0) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
                cb(drogon::HttpResponse::newHttpResponse());
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            name, id
        );
    } else {
        sql = "UPDATE pet SET status=NULLIF($1,'')::pet_status WHERE id=$2";
        db->execSqlAsync(sql,
            [cb](const drogon::orm::Result& r) {
                if (r.affectedRows() == 0) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
                cb(drogon::HttpResponse::newHttpResponse());
            },
            [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
            status, id
        );
    }
}

// ---------------------------------------------------------------------------
// deletePet  DELETE /api/v3/pet/{id}
// ---------------------------------------------------------------------------
void PetController::deletePet(const drogon::HttpRequestPtr&,
                              std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                              int64_t id) {
    auto db = getDb();
    db->execSqlAsync(
        "DELETE FROM pet WHERE id=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.affectedRows() == 0) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }
            cb(drogon::HttpResponse::newHttpResponse()); // 200
        },
        [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
        id
    );
}

// ---------------------------------------------------------------------------
// uploadImage  POST /api/v3/pet/{id}/uploadImage
// ---------------------------------------------------------------------------
void PetController::uploadImage(const drogon::HttpRequestPtr& req,
                                std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                int64_t id) {
    auto db = getDb();
    db->execSqlAsync(
        "SELECT 1 FROM pet WHERE id=$1",
        [req, id, db, cb = std::move(cb)](const drogon::orm::Result& r) mutable {
            if (r.empty()) { cb(error_resp(drogon::k404NotFound, "Pet not found")); return; }

            const std::string body(req->getBody());
            size_t bytes = body.size();
            std::string metadata = req->getParameter("additionalMetadata");

            // Encode body as hex for safe PostgreSQL BYTEA insertion via decode().
            static const char hex_chars[] = "0123456789abcdef";
            std::string hex;
            hex.reserve(body.size() * 2);
            for (unsigned char c : body) {
                hex += hex_chars[c >> 4];
                hex += hex_chars[c & 0xf];
            }

            db->execSqlAsync(
                "INSERT INTO pet_photo (id, pet_id, content_type, metadata, content) "
                "VALUES (nextval('pet_photo_id_seq'), $1, $2, NULLIF($3,''), decode($4,'hex'))",
                [bytes, id, metadata, cb](const drogon::orm::Result&) {
                    nlohmann::json j;
                    j["code"]    = 200;
                    j["type"]    = "upload";
                    j["message"] = "File uploaded to ./petId_" + std::to_string(id) +
                                   (metadata.empty() ? "" : "_" + metadata) +
                                   ", " + std::to_string(bytes) + " bytes";
                    cb(json_resp(drogon::k200OK, j.dump()));
                },
                [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
                id,
                std::string("application/octet-stream"),
                metadata,
                hex
            );
        },
        [cb](const drogon::orm::DrogonDbException& e) { db_err(e, cb); },
        id
    );
}
