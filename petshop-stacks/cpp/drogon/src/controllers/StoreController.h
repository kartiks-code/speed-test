#pragma once
#include <drogon/HttpController.h>

class StoreController : public drogon::HttpController<StoreController> {
public:
    METHOD_LIST_BEGIN
    ADD_METHOD_TO(StoreController::getInventory, "/api/v3/store/inventory",    drogon::Get);
    ADD_METHOD_TO(StoreController::placeOrder,   "/api/v3/store/order",        drogon::Post);
    ADD_METHOD_TO(StoreController::getOrderById, "/api/v3/store/order/{id}",   drogon::Get);
    ADD_METHOD_TO(StoreController::deleteOrder,  "/api/v3/store/order/{id}",   drogon::Delete);
    METHOD_LIST_END

    void getInventory(const drogon::HttpRequestPtr& req,
                      std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void placeOrder(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void getOrderById(const drogon::HttpRequestPtr& req,
                      std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                      int64_t id);
    void deleteOrder(const drogon::HttpRequestPtr& req,
                     std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                     int64_t id);
};
