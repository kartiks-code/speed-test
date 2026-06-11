package com.example.petstore.apis

import com.example.petstore.Paths
import com.example.petstore.models.*
import com.example.petstore.repository.InvalidInputException
import com.example.petstore.repository.PetstoreRepository
import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.resources.delete
import io.ktor.server.resources.get
import io.ktor.server.resources.post
import io.ktor.server.resources.put
import io.ktor.server.response.*
import io.ktor.server.routing.Route

fun Route.UserApi(repo: PetstoreRepository) {

    post<Paths.createUser> { _ ->
        val user = call.receive<User>()
        call.respond(HttpStatusCode.OK, repo.createUser(user))
    }

    post<Paths.createUsersWithListInput> { _ ->
        val users = call.receive<List<User>>()
        if (users.isEmpty()) throw InvalidInputException("User list is empty")
        call.respond(HttpStatusCode.OK, repo.createUsersWithList(users))
    }

    get<Paths.loginUser> { params ->
        val username = params.username ?: throw InvalidInputException("Username required")
        val password = params.password ?: throw InvalidInputException("Password required")
        val token = repo.loginUser(username, password)
        call.respond(HttpStatusCode.OK, token)
    }

    get<Paths.logoutUser> { _ ->
        repo.logoutUser()
        call.respond(HttpStatusCode.OK)
    }

    get<Paths.getUserByName> { params ->
        val user = repo.getUserByName(params.username)
            ?: run { call.respond(HttpStatusCode.NotFound); return@get }
        call.respond(HttpStatusCode.OK, user)
    }

    put<Paths.updateUser> { params ->
        val user = call.receive<User>()
        repo.updateUser(params.username, user)
        call.respond(HttpStatusCode.NoContent)
    }

    delete<Paths.deleteUser> { params ->
        repo.deleteUser(params.username)
        call.respond(HttpStatusCode.NoContent)
    }
}
