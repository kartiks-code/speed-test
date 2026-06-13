#pragma once
#include <drogon/HttpController.h>

class PetController : public drogon::HttpController<PetController> {
public:
    METHOD_LIST_BEGIN
    // Static paths MUST come before parameterised paths to avoid misrouting
    ADD_METHOD_TO(PetController::findByStatus,   "/api/v3/pet/findByStatus",    drogon::Get);
    ADD_METHOD_TO(PetController::findByTags,     "/api/v3/pet/findByTags",      drogon::Get);
    ADD_METHOD_TO(PetController::addPet,         "/api/v3/pet",                 drogon::Post);
    ADD_METHOD_TO(PetController::updatePet,      "/api/v3/pet",                 drogon::Put);
    ADD_METHOD_TO(PetController::uploadImage,    "/api/v3/pet/{id}/uploadImage", drogon::Post);
    ADD_METHOD_TO(PetController::updateWithForm, "/api/v3/pet/{id}",            drogon::Post);
    ADD_METHOD_TO(PetController::getPetById,     "/api/v3/pet/{id}",            drogon::Get);
    ADD_METHOD_TO(PetController::deletePet,      "/api/v3/pet/{id}",            drogon::Delete);
    METHOD_LIST_END

    void addPet(const drogon::HttpRequestPtr& req,
                std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void updatePet(const drogon::HttpRequestPtr& req,
                   std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void findByStatus(const drogon::HttpRequestPtr& req,
                      std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void findByTags(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb);
    void getPetById(const drogon::HttpRequestPtr& req,
                    std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                    int64_t id);
    void updateWithForm(const drogon::HttpRequestPtr& req,
                        std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                        int64_t id);
    void deletePet(const drogon::HttpRequestPtr& req,
                   std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                   int64_t id);
    void uploadImage(const drogon::HttpRequestPtr& req,
                     std::function<void(const drogon::HttpResponsePtr&)>&& cb,
                     int64_t id);
};
