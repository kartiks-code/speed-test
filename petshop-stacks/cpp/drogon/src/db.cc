#include "db.h"
#include <stdexcept>

drogon::orm::DbClientPtr getDb() {
    // Fast clients are registered with isFast=true and retrieved via getFastDbClient().
    auto db = drogon::app().getFastDbClient();
    if (!db) {
        // Fallback to regular client (for non-fast registration)
        db = drogon::app().getDbClient();
    }
    if (!db) {
        throw std::runtime_error("No database client available");
    }
    return db;
}
