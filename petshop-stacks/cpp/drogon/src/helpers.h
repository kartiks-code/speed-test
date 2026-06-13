#pragma once
#include "models.h"
#include <cstdint>
#include <optional>
#include <string>

// Construct a Pet from individual column values (strings/optionals as returned from DB)
Pet row_to_pet(
    std::optional<int64_t> id,
    const std::string& name,
    std::optional<std::string> category_json,
    const std::string& photo_urls_json,
    std::optional<std::string> tags_json,
    std::optional<std::string> status_text
);

Order row_to_order(
    std::optional<int64_t> id,
    std::optional<int64_t> pet_id,
    std::optional<int32_t> quantity,
    std::optional<std::string> ship_date,
    std::optional<std::string> status_text,
    std::optional<bool> complete
);

User row_to_user(
    std::optional<int64_t> id,
    std::optional<std::string> username,
    std::optional<std::string> first_name,
    std::optional<std::string> last_name,
    std::optional<std::string> email,
    std::optional<std::string> password,
    std::optional<std::string> phone,
    std::optional<int32_t> user_status
);

// Serialize to JSON string
std::string pet_to_json(const Pet& p);
std::string order_to_json(const Order& o);
std::string user_to_json(const User& u);
