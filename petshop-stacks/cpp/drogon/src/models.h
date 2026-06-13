#pragma once
#include <cstdint>
#include <optional>
#include <string>
#include <vector>
#include <nlohmann/json.hpp>

struct Category {
    std::optional<int64_t> id;
    std::optional<std::string> name;
};

struct Tag {
    std::optional<int64_t> id;
    std::optional<std::string> name;
};

struct Pet {
    std::optional<int64_t> id;
    std::string name;
    std::optional<Category> category;
    std::vector<std::string> photoUrls;
    std::optional<std::vector<Tag>> tags;
    std::optional<std::string> status; // "available", "pending", "sold"
};

struct Order {
    std::optional<int64_t> id;
    std::optional<int64_t> petId;
    std::optional<int32_t> quantity;
    std::optional<std::string> shipDate; // ISO 8601 string
    std::optional<std::string> status;   // "placed", "approved", "delivered"
    std::optional<bool> complete;
};

struct User {
    std::optional<int64_t> id;
    std::optional<std::string> username;
    std::optional<std::string> firstName;
    std::optional<std::string> lastName;
    std::optional<std::string> email;
    std::optional<std::string> password;
    std::optional<std::string> phone;
    std::optional<int32_t> userStatus;
};

inline void to_json(nlohmann::json& j, const Category& c) {
    j = nlohmann::json::object();
    if (c.id)   j["id"]   = *c.id;
    if (c.name) j["name"] = *c.name;
}
inline void from_json(const nlohmann::json& j, Category& c) {
    c = Category{};
    if (j.contains("id")   && !j["id"].is_null())   c.id   = j["id"].get<int64_t>();
    if (j.contains("name") && !j["name"].is_null())  c.name = j["name"].get<std::string>();
}

inline void to_json(nlohmann::json& j, const Tag& t) {
    j = nlohmann::json::object();
    if (t.id)   j["id"]   = *t.id;
    if (t.name) j["name"] = *t.name;
}
inline void from_json(const nlohmann::json& j, Tag& t) {
    t = Tag{};
    if (j.contains("id")   && !j["id"].is_null())   t.id   = j["id"].get<int64_t>();
    if (j.contains("name") && !j["name"].is_null())  t.name = j["name"].get<std::string>();
}

void to_json(nlohmann::json& j, const Pet& p);
void from_json(const nlohmann::json& j, Pet& p);

void to_json(nlohmann::json& j, const Order& o);
void from_json(const nlohmann::json& j, Order& o);

void to_json(nlohmann::json& j, const User& u);
void from_json(const nlohmann::json& j, User& u);
