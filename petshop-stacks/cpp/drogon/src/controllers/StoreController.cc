#include "StoreController.h"
#include "db.h"
#include "helpers.h"
#include <drogon/HttpResponse.h>
#include <nlohmann/json.hpp>
#include <string>

// ---------------------------------------------------------------------------
// Shared helpers (local to translation unit)
// ---------------------------------------------------------------------------

static drogon::HttpResponsePtr store_json_resp(drogon::HttpStatusCode code,
                                               const std::string& body) {
    auto r = drogon::HttpResponse::newHttpResponse();
    r->setStatusCode(code);
    r->setContentTypeCode(drogon::CT_APPLICATION_JSON);
    r->setBody(body);
    return r;
}

static drogon::HttpResponsePtr store_error(drogon::HttpStatusCode code,
                                           const std::string& msg) {
    nlohmann::json j;
    j["message"] = msg;
    return store_json_resp(code, j.dump());
}

static void store_db_err(const drogon::orm::DrogonDbException& e,
                         std::function<void(const drogon::HttpResponsePtr&)> cb) {
    (void)e;
    cb(store_error(drogon::k500InternalServerError, "internal server error"));
}

static Order order_from_row(const drogon::orm::Row& row) {
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
    auto opt_bool = [&](const char* col) -> std::optional<bool> {
        if (row[col].isNull()) return std::nullopt;
        return row[col].as<bool>();
    };
    return row_to_order(
        opt_int64("id"),
        opt_int64("pet_id"),
        opt_int32("quantity"),
        opt_str("ship_date"),
        opt_str("status"),
        opt_bool("complete")
    );
}

// Use NULLIF to convert sentinel values back to NULL for optional columns.
// Callers pass petId=0 and shipDate="" when those fields are absent.
static const char* UPSERT_ORDER_SQL_V2 =
    "INSERT INTO \"order\" (id, pet_id, quantity, ship_date, status, complete) "
    "VALUES ($1, NULLIF($2::bigint, 0), NULLIF($3::int, 0), "
    "        NULLIF($4,'')::timestamptz, NULLIF($5,'')::order_status, $6) "
    "ON CONFLICT (id) DO UPDATE SET "
    "  pet_id=EXCLUDED.pet_id, quantity=EXCLUDED.quantity, "
    "  ship_date=EXCLUDED.ship_date, status=EXCLUDED.status, complete=EXCLUDED.complete "
    "RETURNING id, pet_id, quantity, ship_date::text, status::text, complete";

static void do_upsert_order(drogon::orm::DbClientPtr db,
                             int64_t id, const Order& order,
                             std::function<void(const drogon::HttpResponsePtr&)> cb) {
    int64_t pet_id    = order.petId.value_or(0);
    int32_t quantity  = order.quantity.value_or(0);
    std::string sdate = order.shipDate.value_or("");
    std::string status= order.status.value_or("");
    bool complete     = order.complete.value_or(false);

    db->execSqlAsync(
        UPSERT_ORDER_SQL_V2,
        [cb](const drogon::orm::Result& r) {
            cb(store_json_resp(drogon::k200OK, order_to_json(order_from_row(r[0]))));
        },
        [cb](const drogon::orm::DrogonDbException& e) { store_db_err(e, cb); },
        id, pet_id, quantity, sdate, status, complete
    );
}

// ---------------------------------------------------------------------------
// getInventory  GET /api/v3/store/inventory
// ---------------------------------------------------------------------------
void StoreController::getInventory(const drogon::HttpRequestPtr&,
                                   std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    auto db = getDb();
    db->execSqlAsync(
        "SELECT status::text, COUNT(*) AS cnt FROM pet "
        "WHERE status IS NOT NULL GROUP BY status ORDER BY status",
        [cb](const drogon::orm::Result& r) {
            nlohmann::json j = nlohmann::json::object();
            for (const auto& row : r) {
                j[row["status"].as<std::string>()] = row["cnt"].as<int64_t>();
            }
            cb(store_json_resp(drogon::k200OK, j.dump()));
        },
        [cb](const drogon::orm::DrogonDbException& e) { store_db_err(e, cb); }
    );
}

// ---------------------------------------------------------------------------
// placeOrder  POST /api/v3/store/order
// ---------------------------------------------------------------------------
void StoreController::placeOrder(const drogon::HttpRequestPtr& req,
                                 std::function<void(const drogon::HttpResponsePtr&)>&& cb) {
    nlohmann::json body;
    try { body = nlohmann::json::parse(req->getBody()); }
    catch (...) { cb(store_error(drogon::k400BadRequest, "invalid JSON")); return; }

    Order order;
    try { from_json(body, order); }
    catch (...) { cb(store_error(drogon::k400BadRequest, "invalid input")); return; }

    if (order.status) {
        const auto& s = *order.status;
        if (!s.empty() && s != "placed" && s != "approved" && s != "delivered") {
            cb(store_error(drogon::k400BadRequest, "invalid status")); return;
        }
    }

    auto db = getDb();
    if (order.id) {
        do_upsert_order(db, *order.id, order, std::move(cb));
    } else {
        db->execSqlAsync(
            "SELECT nextval('order_id_seq') AS id",
            [order, db, cb = std::move(cb)](const drogon::orm::Result& r) mutable {
                do_upsert_order(db, r[0]["id"].as<int64_t>(), order, std::move(cb));
            },
            [cb](const drogon::orm::DrogonDbException& e) { store_db_err(e, cb); }
        );
    }
}

// ---------------------------------------------------------------------------
// getOrderById  GET /api/v3/store/order/{id}
// ---------------------------------------------------------------------------
void StoreController::getOrderById(const drogon::HttpRequestPtr&,
                                   std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                   int64_t id) {
    auto db = getDb();
    db->execSqlAsync(
        "SELECT id, pet_id, quantity, ship_date::text, status::text, complete "
        "FROM \"order\" WHERE id=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.empty()) { cb(store_error(drogon::k404NotFound, "Order not found")); return; }
            cb(store_json_resp(drogon::k200OK, order_to_json(order_from_row(r[0]))));
        },
        [cb](const drogon::orm::DrogonDbException& e) { store_db_err(e, cb); },
        id
    );
}

// ---------------------------------------------------------------------------
// deleteOrder  DELETE /api/v3/store/order/{id}
// ---------------------------------------------------------------------------
void StoreController::deleteOrder(const drogon::HttpRequestPtr&,
                                  std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                                  int64_t id) {
    auto db = getDb();
    db->execSqlAsync(
        "DELETE FROM \"order\" WHERE id=$1",
        [cb](const drogon::orm::Result& r) {
            if (r.affectedRows() == 0) { cb(store_error(drogon::k404NotFound, "Order not found")); return; }
            cb(drogon::HttpResponse::newHttpResponse());
        },
        [cb](const drogon::orm::DrogonDbException& e) { store_db_err(e, cb); },
        id
    );
}
