#include "helpers.h"
#include "models.h"
#include <nlohmann/json.hpp>
#include <stdexcept>

// ---------------------------------------------------------------------------
// models.h to_json / from_json implementations
// ---------------------------------------------------------------------------

void to_json(nlohmann::json& j, const Pet& p) {
    j = nlohmann::json::object();
    if (p.id) j["id"] = *p.id;
    j["name"] = p.name;
    if (p.category) j["category"] = *p.category;
    j["photoUrls"] = p.photoUrls;
    if (p.tags) j["tags"] = *p.tags;
    if (p.status) j["status"] = *p.status;
}

void from_json(const nlohmann::json& j, Pet& p) {
    p = Pet{};
    if (j.contains("id") && !j["id"].is_null()) p.id = j["id"].get<int64_t>();
    if (j.contains("name")) p.name = j["name"].get<std::string>();
    if (j.contains("category") && !j["category"].is_null())
        p.category = j["category"].get<Category>();
    if (j.contains("photoUrls") && j["photoUrls"].is_array())
        p.photoUrls = j["photoUrls"].get<std::vector<std::string>>();
    if (j.contains("tags") && !j["tags"].is_null() && j["tags"].is_array())
        p.tags = j["tags"].get<std::vector<Tag>>();
    if (j.contains("status") && !j["status"].is_null())
        p.status = j["status"].get<std::string>();
}

void to_json(nlohmann::json& j, const Order& o) {
    j = nlohmann::json::object();
    if (o.id)       j["id"]       = *o.id;
    if (o.petId)    j["petId"]    = *o.petId;
    if (o.quantity) j["quantity"] = *o.quantity;
    if (o.shipDate) j["shipDate"] = *o.shipDate;
    if (o.status)   j["status"]   = *o.status;
    if (o.complete) j["complete"] = *o.complete;
}

void from_json(const nlohmann::json& j, Order& o) {
    o = Order{};
    if (j.contains("id") && !j["id"].is_null())             o.id       = j["id"].get<int64_t>();
    if (j.contains("petId") && !j["petId"].is_null())        o.petId    = j["petId"].get<int64_t>();
    if (j.contains("quantity") && !j["quantity"].is_null())  o.quantity = j["quantity"].get<int32_t>();
    if (j.contains("shipDate") && !j["shipDate"].is_null())  o.shipDate = j["shipDate"].get<std::string>();
    if (j.contains("status") && !j["status"].is_null())      o.status   = j["status"].get<std::string>();
    if (j.contains("complete") && !j["complete"].is_null())  o.complete = j["complete"].get<bool>();
}

void to_json(nlohmann::json& j, const User& u) {
    j = nlohmann::json::object();
    if (u.id)         j["id"]         = *u.id;
    if (u.username)   j["username"]   = *u.username;
    if (u.firstName)  j["firstName"]  = *u.firstName;
    if (u.lastName)   j["lastName"]   = *u.lastName;
    if (u.email)      j["email"]      = *u.email;
    if (u.password)   j["password"]   = *u.password;
    if (u.phone)      j["phone"]      = *u.phone;
    if (u.userStatus) j["userStatus"] = *u.userStatus;
}

void from_json(const nlohmann::json& j, User& u) {
    u = User{};
    if (j.contains("id") && !j["id"].is_null())                 u.id         = j["id"].get<int64_t>();
    if (j.contains("username") && !j["username"].is_null())     u.username   = j["username"].get<std::string>();
    if (j.contains("firstName") && !j["firstName"].is_null())   u.firstName  = j["firstName"].get<std::string>();
    if (j.contains("lastName") && !j["lastName"].is_null())     u.lastName   = j["lastName"].get<std::string>();
    if (j.contains("email") && !j["email"].is_null())           u.email      = j["email"].get<std::string>();
    if (j.contains("password") && !j["password"].is_null())     u.password   = j["password"].get<std::string>();
    if (j.contains("phone") && !j["phone"].is_null())           u.phone      = j["phone"].get<std::string>();
    if (j.contains("userStatus") && !j["userStatus"].is_null()) u.userStatus = j["userStatus"].get<int32_t>();
}

// ---------------------------------------------------------------------------
// Row-to-struct helpers (no Drogon dependency)
// ---------------------------------------------------------------------------

Pet row_to_pet(
    std::optional<int64_t> id,
    const std::string& name,
    std::optional<std::string> category_json,
    const std::string& photo_urls_json,
    std::optional<std::string> tags_json,
    std::optional<std::string> status_text)
{
    Pet p;
    p.id   = id;
    p.name = name;

    if (category_json && !category_json->empty()) {
        try {
            auto jcat = nlohmann::json::parse(*category_json);
            if (!jcat.is_null()) p.category = jcat.get<Category>();
        } catch (...) {}
    }

    if (!photo_urls_json.empty()) {
        try {
            auto jpu = nlohmann::json::parse(photo_urls_json);
            if (jpu.is_array()) p.photoUrls = jpu.get<std::vector<std::string>>();
        } catch (...) {}
    }

    if (tags_json && !tags_json->empty()) {
        try {
            auto jtags = nlohmann::json::parse(*tags_json);
            if (jtags.is_array()) p.tags = jtags.get<std::vector<Tag>>();
        } catch (...) {}
    }

    p.status = status_text;
    return p;
}

Order row_to_order(
    std::optional<int64_t> id,
    std::optional<int64_t> pet_id,
    std::optional<int32_t> quantity,
    std::optional<std::string> ship_date,
    std::optional<std::string> status_text,
    std::optional<bool> complete)
{
    Order o;
    o.id       = id;
    o.petId    = pet_id;
    o.quantity = quantity;
    o.shipDate = ship_date;
    o.status   = status_text;
    o.complete = complete;
    return o;
}

User row_to_user(
    std::optional<int64_t> id,
    std::optional<std::string> username,
    std::optional<std::string> first_name,
    std::optional<std::string> last_name,
    std::optional<std::string> email,
    std::optional<std::string> password,
    std::optional<std::string> phone,
    std::optional<int32_t> user_status)
{
    User u;
    u.id         = id;
    u.username   = username;
    u.firstName  = first_name;
    u.lastName   = last_name;
    u.email      = email;
    u.password   = password;
    u.phone      = phone;
    u.userStatus = user_status;
    return u;
}

// ---------------------------------------------------------------------------
// JSON serialization helpers
// ---------------------------------------------------------------------------

std::string pet_to_json(const Pet& p) {
    nlohmann::json j;
    to_json(j, p);
    return j.dump();
}

std::string order_to_json(const Order& o) {
    nlohmann::json j;
    to_json(j, o);
    return j.dump();
}

std::string user_to_json(const User& u) {
    nlohmann::json j;
    to_json(j, u);
    return j.dump();
}
