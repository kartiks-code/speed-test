package com.example.petstore.apis

import com.example.petstore.Paths
import com.example.petstore.models.*
import com.example.petstore.repository.InvalidInputException
import com.example.petstore.repository.NotFoundException
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
import io.ktor.utils.io.toByteArray

fun Route.PetApi(repo: PetstoreRepository) {

    post<Paths.addPet> { _ ->
        val pet = call.receive<Pet>()
        if (pet.name.isBlank()) throw InvalidInputException("Pet name required")
        call.respond(HttpStatusCode.OK, repo.addPet(pet))
    }

    put<Paths.updatePet> { _ ->
        val pet = call.receive<Pet>()
        if (pet.id == null) throw InvalidInputException("Pet id required")
        val updated = try {
            repo.updatePet(pet)
        } catch (e: NotFoundException) {
            call.respond(HttpStatusCode.NotFound); return@put
        }
        call.respond(HttpStatusCode.OK, updated)
    }

    get<Paths.findPetsByStatus> { params ->
        val status = params.status ?: "available"
        call.respond(HttpStatusCode.OK, repo.findPetsByStatus(status))
    }

    get<Paths.findPetsByTags> { params ->
        val tags = params.tags ?: emptyList()
        call.respond(HttpStatusCode.OK, repo.findPetsByTags(tags))
    }

    get<Paths.getPetById> { params ->
        val pet = repo.getPetById(params.petId)
            ?: throw NotFoundException("Pet not found: ${params.petId}")
        call.respond(HttpStatusCode.OK, pet)
    }

    post<Paths.updatePetWithForm> { params ->
        val form = try { call.receiveParameters() } catch (_: Exception) { Parameters.Empty }
        val name = form["name"] ?: params.name
        val status = form["status"] ?: params.status
        val updated = repo.updatePetWithForm(params.petId, name, status)
        call.respond(HttpStatusCode.OK, updated)
    }

    delete<Paths.deletePet> { params ->
        repo.deletePet(params.petId)
        call.respond(HttpStatusCode.OK)
    }

    post<Paths.uploadFile> { params ->
        val bytes = call.receiveChannel().toByteArray()
        val response = repo.uploadFile(params.petId, params.additionalMetadata, bytes)
        call.respond(HttpStatusCode.OK, response)
    }
}
