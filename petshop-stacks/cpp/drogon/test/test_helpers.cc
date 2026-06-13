#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest.h"
#include "helpers.h"
#include <nlohmann/json.hpp>

// ---------------------------------------------------------------------------
// row_to_pet
// ---------------------------------------------------------------------------

TEST_CASE("row_to_pet decodes all columns") {
    auto pet = row_to_pet(
        int64_t{7}, "Fido",
        std::string{R"({"id":1,"name":"dogs"})"},
        R"(["http://example.com/a.jpg"])",
        std::string{R"([{"id":3,"name":"cute"}])"},
        std::string{"available"}
    );
    CHECK(pet.id == int64_t{7});
    CHECK(pet.name == "Fido");
    CHECK(pet.status == "available");
    REQUIRE(pet.category.has_value());
    CHECK(pet.category->name == "dogs");
    CHECK(pet.category->id == int64_t{1});
    REQUIRE(pet.tags.has_value());
    REQUIRE(pet.tags->size() == 1u);
    CHECK(pet.tags->at(0).name == "cute");
    REQUIRE(pet.photoUrls.size() == 1u);
    CHECK(pet.photoUrls[0] == "http://example.com/a.jpg");
}

TEST_CASE("row_to_pet handles null optionals") {
    auto pet = row_to_pet(std::nullopt, "Rex", std::nullopt, "[]", std::nullopt, std::nullopt);
    CHECK(!pet.id.has_value());
    CHECK(pet.name == "Rex");
    CHECK(pet.photoUrls.empty());
    CHECK(!pet.status.has_value());
    CHECK(!pet.category.has_value());
    CHECK(!pet.tags.has_value());
}

TEST_CASE("row_to_pet tolerates malformed category JSON") {
    auto pet = row_to_pet(std::nullopt, "X", std::string{"not json"}, "[]", std::nullopt, std::nullopt);
    CHECK(!pet.category.has_value());
    CHECK(pet.photoUrls.empty());
}

TEST_CASE("row_to_pet tolerates malformed photo_urls JSON") {
    auto pet = row_to_pet(std::nullopt, "X", std::nullopt, "also not json", std::nullopt, std::nullopt);
    CHECK(pet.photoUrls.empty());
}

TEST_CASE("row_to_pet tolerates malformed tags JSON") {
    auto pet = row_to_pet(std::nullopt, "X", std::nullopt, "[]", std::string{"{"}, std::nullopt);
    CHECK(!pet.tags.has_value());
}

TEST_CASE("row_to_pet handles multiple photo URLs") {
    auto pet = row_to_pet(std::nullopt, "Y", std::nullopt,
                          R"(["http://a.com","http://b.com"])",
                          std::nullopt, std::nullopt);
    REQUIRE(pet.photoUrls.size() == 2u);
    CHECK(pet.photoUrls[0] == "http://a.com");
    CHECK(pet.photoUrls[1] == "http://b.com");
}

TEST_CASE("row_to_pet handles multiple tags") {
    auto pet = row_to_pet(std::nullopt, "Z", std::nullopt, "[]",
                          std::string{R"([{"id":1,"name":"a"},{"id":2,"name":"b"}])"},
                          std::nullopt);
    REQUIRE(pet.tags.has_value());
    CHECK(pet.tags->size() == 2u);
    CHECK(pet.tags->at(1).name == "b");
}

TEST_CASE("row_to_pet round-trips through pet_to_json") {
    auto pet = row_to_pet(int64_t{42}, "Buddy",
                          std::string{R"({"id":5,"name":"cats"})"},
                          R"(["http://img.com/cat.jpg"])",
                          std::string{R"([{"id":10,"name":"fluffy"}])"},
                          std::string{"pending"});
    auto json_str = pet_to_json(pet);
    auto j = nlohmann::json::parse(json_str);
    CHECK(j["id"] == 42);
    CHECK(j["name"] == "Buddy");
    CHECK(j["status"] == "pending");
    CHECK(j["category"]["name"] == "cats");
    CHECK(j["tags"][0]["name"] == "fluffy");
    CHECK(j["photoUrls"][0] == "http://img.com/cat.jpg");
}

// ---------------------------------------------------------------------------
// row_to_order
// ---------------------------------------------------------------------------

TEST_CASE("row_to_order decodes all columns") {
    auto o = row_to_order(int64_t{1}, int64_t{2}, int32_t{3},
                          std::string{"2025-01-01T00:00:00Z"},
                          std::string{"placed"}, bool{false});
    CHECK(o.id == int64_t{1});
    CHECK(o.petId == int64_t{2});
    CHECK(o.quantity == int32_t{3});
    CHECK(o.shipDate == "2025-01-01T00:00:00Z");
    CHECK(o.status == "placed");
    REQUIRE(o.complete.has_value());
    CHECK(*o.complete == false);
}

TEST_CASE("row_to_order handles nulls") {
    auto o = row_to_order(std::nullopt, std::nullopt, std::nullopt,
                          std::nullopt, std::nullopt, std::nullopt);
    CHECK(!o.id.has_value());
    CHECK(!o.petId.has_value());
    CHECK(!o.quantity.has_value());
    CHECK(!o.shipDate.has_value());
    CHECK(!o.status.has_value());
    CHECK(!o.complete.has_value());
}

TEST_CASE("row_to_order round-trips through order_to_json") {
    auto o = row_to_order(int64_t{10}, int64_t{20}, int32_t{5},
                          std::string{"2025-06-01T12:00:00Z"},
                          std::string{"approved"}, bool{true});
    auto j = nlohmann::json::parse(order_to_json(o));
    CHECK(j["id"] == 10);
    CHECK(j["petId"] == 20);
    CHECK(j["quantity"] == 5);
    CHECK(j["status"] == "approved");
    CHECK(j["complete"] == true);
}

// ---------------------------------------------------------------------------
// row_to_user
// ---------------------------------------------------------------------------

TEST_CASE("row_to_user decodes all columns") {
    auto u = row_to_user(int64_t{1},
                         std::string{"alice"},
                         std::string{"Alice"},
                         std::string{"Smith"},
                         std::string{"alice@example.com"},
                         std::string{"secret"},
                         std::string{"+1555"},
                         int32_t{1});
    CHECK(u.id == int64_t{1});
    CHECK(u.username == "alice");
    CHECK(u.firstName == "Alice");
    CHECK(u.lastName == "Smith");
    CHECK(u.email == "alice@example.com");
    CHECK(u.password == "secret");
    CHECK(u.phone == "+1555");
    CHECK(u.userStatus == int32_t{1});
}

TEST_CASE("row_to_user handles nulls") {
    auto u = row_to_user(std::nullopt, std::nullopt, std::nullopt,
                         std::nullopt, std::nullopt, std::nullopt,
                         std::nullopt, std::nullopt);
    CHECK(!u.id.has_value());
    CHECK(!u.username.has_value());
    CHECK(!u.userStatus.has_value());
}

TEST_CASE("row_to_user round-trips through user_to_json") {
    auto u = row_to_user(int64_t{7}, std::string{"bob"}, std::string{"Bob"},
                         std::string{"Jones"}, std::string{"bob@test.com"},
                         std::string{"pw"}, std::string{"555"}, int32_t{0});
    auto j = nlohmann::json::parse(user_to_json(u));
    CHECK(j["id"] == 7);
    CHECK(j["username"] == "bob");
    CHECK(j["firstName"] == "Bob");
    CHECK(j["lastName"] == "Jones");
}

// ---------------------------------------------------------------------------
// JSON serialization edge cases
// ---------------------------------------------------------------------------

TEST_CASE("pet_to_json omits null fields") {
    Pet p;
    p.name = "NoId";
    p.photoUrls = {};
    auto j = nlohmann::json::parse(pet_to_json(p));
    CHECK_FALSE(j.contains("id"));
    CHECK_FALSE(j.contains("status"));
    CHECK_FALSE(j.contains("category"));
    CHECK_FALSE(j.contains("tags"));
}

TEST_CASE("order_to_json omits null fields") {
    Order o;
    auto j = nlohmann::json::parse(order_to_json(o));
    CHECK_FALSE(j.contains("id"));
    CHECK_FALSE(j.contains("status"));
}

TEST_CASE("user_to_json omits null fields") {
    User u;
    auto j = nlohmann::json::parse(user_to_json(u));
    CHECK_FALSE(j.contains("id"));
    CHECK_FALSE(j.contains("username"));
}

TEST_CASE("from_json/to_json Pet round-trip with camelCase") {
    nlohmann::json input = {
        {"id", 99},
        {"name", "TestPet"},
        {"photoUrls", {"http://a.com"}},
        {"status", "sold"},
        {"category", {{"id", 2}, {"name", "cats"}}},
        {"tags", nlohmann::json::array({{{"id", 1}, {"name", "tag1"}}})}
    };
    Pet p;
    from_json(input, p);
    CHECK(p.id == int64_t{99});
    CHECK(p.name == "TestPet");
    CHECK(p.status == "sold");
    REQUIRE(p.category.has_value());
    CHECK(p.category->name == "cats");

    nlohmann::json out;
    to_json(out, p);
    CHECK(out["id"] == 99);
    CHECK(out["photoUrls"][0] == "http://a.com");
}
