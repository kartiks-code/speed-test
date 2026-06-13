#include <drogon/drogon.h>
#include <drogon/orm/DbConfig.h>
#include <trantor/utils/Logger.h>
#include <thread>
#include <cstdlib>
#include <string>
#include "config.h"

int main() {
    int port = 8080;
    if (const char* p = std::getenv("PORT")) {
        try { port = std::stoi(p); } catch (...) {}
    }

    const int nThreads = static_cast<int>(std::thread::hardware_concurrency());
    const int nConns   = pool_size();

    const char* pg_host = std::getenv("POSTGRES_HOST");
    const char* pg_port = std::getenv("POSTGRES_PORT");
    const char* pg_db   = std::getenv("POSTGRES_DB");
    const char* pg_user = std::getenv("POSTGRES_USER");
    const char* pg_pass = std::getenv("POSTGRES_PASSWORD");

    drogon::orm::PostgresConfig pgcfg;
    pgcfg.host             = pg_host     ? pg_host     : "localhost";
    pgcfg.port             = static_cast<unsigned short>(pg_port ? std::stoi(pg_port) : 5432);
    pgcfg.databaseName     = pg_db       ? pg_db       : "cpp-drogon";
    pgcfg.username         = pg_user     ? pg_user     : "myuser";
    pgcfg.password         = pg_pass     ? pg_pass     : "mypassword";
    pgcfg.connectionNumber = static_cast<size_t>(nConns);
    pgcfg.name             = "default";
    pgcfg.isFast           = true;
    pgcfg.characterSet     = "";
    pgcfg.timeout          = -1.0;
    pgcfg.autoBatch        = false;

    drogon::app()
        .setLogLevel(trantor::Logger::kWarn)
        .addDbClient(pgcfg)
        .setThreadNum(nThreads > 0 ? nThreads : 4)
        .addListener("0.0.0.0", port)
        .run();

    return 0;
}
