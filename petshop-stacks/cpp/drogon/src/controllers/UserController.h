#pragma once
#include <drogon/HttpController.h>

class UserController : public drogon::HttpController<UserController> {
public:
    METHOD_LIST_BEGIN
    ADD_METHOD_TO(UserController::createUser,         "/api/v3/user",                  drogon::Post);
    ADD_METHOD_TO(UserController::createUsersWithList,"/api/v3/user/createWithList",    drogon::Post);
    ADD_METHOD_TO(UserController::loginUser,          "/api/v3/user/login",             drogon::Get);
    ADD_METHOD_TO(UserController::logoutUser,         "/api/v3/user/logout",            drogon::Get);
    ADD_METHOD_TO(UserController::getUserByName,      "/api/v3/user/{username}",        drogon::Get);
    ADD_METHOD_TO(UserController::updateUser,         "/api/v3/user/{username}",        drogon::Put);
    ADD_METHOD_TO(UserController::deleteUser,         "/api/v3/user/{username}",        drogon::Delete);
    METHOD_LIST_END

    void createUser(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void createUsersWithList(const drogon::HttpRequestPtr& req,
                             std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void loginUser(const drogon::HttpRequestPtr& req,
                   std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void logoutUser(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void getUserByName(const drogon::HttpRequestPtr& req,
                       std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                       const std::string& username);
    void updateUser(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                    const std::string& username);
    void deleteUser(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                    const std::string& username);
};
