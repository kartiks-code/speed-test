package com.example.petstore

import io.ktor.server.routing.*
import com.example.petstore.apis.PetApi
import com.example.petstore.apis.StoreApi
import com.example.petstore.apis.UserApi
import com.example.petstore.repository.PetstoreRepository

fun Route.AllApis(repo: PetstoreRepository) {
    PetApi(repo)
    StoreApi(repo)
    UserApi(repo)
}
