package com.example.petstore.apis

import com.example.petstore.Paths
import com.example.petstore.models.*
import com.example.petstore.repository.PetstoreRepository
import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.resources.delete
import io.ktor.server.resources.get
import io.ktor.server.resources.post
import io.ktor.server.response.*
import io.ktor.server.routing.Route

fun Route.StoreApi(repo: PetstoreRepository) {

    get<Paths.getInventory> { _ ->
        call.respond(HttpStatusCode.OK, repo.getInventory())
    }

    post<Paths.placeOrder> { _ ->
        val order = call.receive<Order>()
        call.respond(HttpStatusCode.OK, repo.placeOrder(order))
    }

    get<Paths.getOrderById> { params ->
        val order = repo.getOrderById(params.orderId)
            ?: run { call.respond(HttpStatusCode.NotFound); return@get }
        call.respond(HttpStatusCode.OK, order)
    }

    delete<Paths.deleteOrder> { params ->
        repo.deleteOrder(params.orderId)
        call.respond(HttpStatusCode.OK)
    }
}
