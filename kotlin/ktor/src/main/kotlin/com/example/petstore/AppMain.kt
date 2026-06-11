package com.example.petstore

import com.example.petstore.repository.InMemoryPetstoreRepository
import com.example.petstore.repository.InvalidInputException
import com.example.petstore.repository.NotFoundException
import com.example.petstore.repository.PetstoreRepository
import com.example.petstore.repository.createDataSource
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import io.ktor.server.application.*
import io.ktor.server.plugins.autohead.*
import io.ktor.server.plugins.contentnegotiation.*
import io.ktor.server.plugins.defaultheaders.*
import io.ktor.server.plugins.statuspages.*
import io.ktor.server.resources.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.json.Json

// Entry point called by EngineMain via application.conf
fun Application.main() {
    configureModule(createPostgresRepository())
}

// Overload exposed for tests — accepts any PetstoreRepository implementation
fun Application.configureModule(repo: PetstoreRepository = InMemoryPetstoreRepository()) {
    install(DefaultHeaders)
    install(AutoHeadResponse)
    install(Resources)

    install(ContentNegotiation) {
        json(Json {
            encodeDefaults = false
            ignoreUnknownKeys = true
        })
    }

    install(StatusPages) {
        exception<NotFoundException> { call, e ->
            call.respond(HttpStatusCode.NotFound, mapOf("message" to e.message))
        }
        exception<InvalidInputException> { call, e ->
            call.respond(HttpStatusCode.BadRequest, mapOf("message" to e.message))
        }
        exception<Throwable> { call, e ->
            call.application.log.error("Unhandled exception", e)
            call.respond(HttpStatusCode.InternalServerError, mapOf("message" to "Internal server error"))
        }
    }

    routing {
        route("/api/v3") {
            AllApis(repo)
        }
    }
}

private fun createPostgresRepository(): com.example.petstore.repository.PostgresPetstoreRepository =
    com.example.petstore.repository.PostgresPetstoreRepository(createDataSource())
