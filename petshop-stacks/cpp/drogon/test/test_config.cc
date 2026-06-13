// test_config.cc — unit tests for config.cc (no Drogon dependency)
// Note: DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN is defined in test_helpers.cc,
// so this file must NOT define it again.
#include "doctest.h"
#include "config.h"
#include <cstdlib>
#include <string>

// RAII helper: sets an env var for the duration of a test, then restores it.
struct EnvGuard {
    std::string key;
    std::string old_val;
    bool had_old;

    EnvGuard(const char* k, const char* v) : key(k) {
        const char* existing = std::getenv(k);
        had_old = (existing != nullptr);
        if (had_old) old_val = existing;
        if (v) ::setenv(k, v, 1);
        else   ::unsetenv(k);
    }
    ~EnvGuard() {
        if (had_old) ::setenv(key.c_str(), old_val.c_str(), 1);
        else         ::unsetenv(key.c_str());
    }
};

// ---------------------------------------------------------------------------
// build_connection_string
// ---------------------------------------------------------------------------

TEST_CASE("build_connection_string uses defaults when no env vars set") {
    EnvGuard g1("DATABASE_URL",        nullptr);
    EnvGuard g2("POSTGRES_HOST",       nullptr);
    EnvGuard g3("POSTGRES_PORT",       nullptr);
    EnvGuard g4("POSTGRES_USER",       nullptr);
    EnvGuard g5("POSTGRES_PASSWORD",   nullptr);
    EnvGuard g6("POSTGRES_DB",         nullptr);

    auto cs = build_connection_string();
    CHECK(cs.find("host=localhost") != std::string::npos);
    CHECK(cs.find("port=5434")      != std::string::npos);
    CHECK(cs.find("user=myuser")    != std::string::npos);
    CHECK(cs.find("password=mypassword") != std::string::npos);
    CHECK(cs.find("dbname=cpp-drogon")   != std::string::npos);
}

TEST_CASE("build_connection_string prefers DATABASE_URL when set") {
    EnvGuard g("DATABASE_URL", "postgres://host:5432/db");
    auto cs = build_connection_string();
    CHECK(cs == "postgres://host:5432/db");
}

TEST_CASE("build_connection_string uses POSTGRES_HOST when set") {
    EnvGuard g1("DATABASE_URL",  nullptr);
    EnvGuard g2("POSTGRES_HOST", "mydbhost");
    auto cs = build_connection_string();
    CHECK(cs.find("host=mydbhost") != std::string::npos);
}

TEST_CASE("build_connection_string uses POSTGRES_PORT when set") {
    EnvGuard g1("DATABASE_URL",  nullptr);
    EnvGuard g2("POSTGRES_PORT", "5432");
    auto cs = build_connection_string();
    CHECK(cs.find("port=5432") != std::string::npos);
}

TEST_CASE("build_connection_string uses POSTGRES_DB when set") {
    EnvGuard g1("DATABASE_URL", nullptr);
    EnvGuard g2("POSTGRES_DB",  "my_custom_db");
    auto cs = build_connection_string();
    CHECK(cs.find("dbname=my_custom_db") != std::string::npos);
}

TEST_CASE("build_connection_string uses POSTGRES_USER and PASSWORD when set") {
    EnvGuard g1("DATABASE_URL",      nullptr);
    EnvGuard g2("POSTGRES_USER",     "testuser");
    EnvGuard g3("POSTGRES_PASSWORD", "testpass");
    auto cs = build_connection_string();
    CHECK(cs.find("user=testuser")    != std::string::npos);
    CHECK(cs.find("password=testpass") != std::string::npos);
}

TEST_CASE("build_connection_string ignores empty DATABASE_URL") {
    EnvGuard g1("DATABASE_URL",  "");
    EnvGuard g2("POSTGRES_HOST", nullptr);
    auto cs = build_connection_string();
    // Should fall back to keyword=value form, not return empty string
    CHECK(cs.find("host=") != std::string::npos);
}

// ---------------------------------------------------------------------------
// pool_size
// ---------------------------------------------------------------------------

TEST_CASE("pool_size returns default 10 when DB_POOL_MAX unset") {
    EnvGuard g("DB_POOL_MAX", nullptr);
    CHECK(pool_size() == 10);
}

TEST_CASE("pool_size returns configured value") {
    EnvGuard g("DB_POOL_MAX", "50");
    CHECK(pool_size() == 50);
}

TEST_CASE("pool_size returns default for non-numeric DB_POOL_MAX") {
    EnvGuard g("DB_POOL_MAX", "not-a-number");
    CHECK(pool_size() == 10);
}

TEST_CASE("pool_size returns default for zero DB_POOL_MAX") {
    EnvGuard g("DB_POOL_MAX", "0");
    CHECK(pool_size() == 10);
}

TEST_CASE("pool_size returns default for negative DB_POOL_MAX") {
    EnvGuard g("DB_POOL_MAX", "-5");
    CHECK(pool_size() == 10);
}

TEST_CASE("pool_size returns default for empty DB_POOL_MAX") {
    EnvGuard g("DB_POOL_MAX", "");
    CHECK(pool_size() == 10);
}
