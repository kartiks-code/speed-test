#pragma once
#include <drogon/drogon.h>

// Returns the shared Drogon DbClient configured at startup in main.cc.
// All controllers obtain the connection pool through this function.
drogon::orm::DbClientPtr getDb();
