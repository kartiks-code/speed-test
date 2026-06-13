#include "config.h"
#include <cstdlib>
#include <stdexcept>
#include <string>

static std::string env_or(const char* name, const char* fallback) {
    const char* v = std::getenv(name);
    return (v && v[0] != '\0') ? std::string(v) : std::string(fallback);
}

std::string build_connection_string() {
    if (const char* url = std::getenv("DATABASE_URL")) {
        if (url[0] != '\0') return url;
    }

    std::string host     = env_or("POSTGRES_HOST",     "localhost");
    std::string port     = env_or("POSTGRES_PORT",     "5434");
    std::string user     = env_or("POSTGRES_USER",     "myuser");
    std::string password = env_or("POSTGRES_PASSWORD", "mypassword");
    std::string db       = env_or("POSTGRES_DB",       "cpp-drogon");

    return "host=" + host +
           " port=" + port +
           " user=" + user +
           " password=" + password +
           " dbname=" + db +
           " sslmode=disable";
}

int pool_size() {
    if (const char* v = std::getenv("DB_POOL_MAX")) {
        if (v[0] != '\0') {
            try {
                int n = std::stoi(v);
                return n > 0 ? n : 10;
            } catch (...) {}
        }
    }
    return 10;
}
