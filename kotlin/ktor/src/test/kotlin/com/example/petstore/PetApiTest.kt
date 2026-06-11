package com.example.petstore

import com.example.petstore.models.*
import com.example.petstore.repository.InMemoryPetstoreRepository
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.server.testing.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private val json = Json { ignoreUnknownKeys = true; encodeDefaults = false }

class PetApiTest {

    private fun testApp(block: suspend ApplicationTestBuilder.() -> Unit) =
        testApplication {
            application { configureModule(InMemoryPetstoreRepository()) }
            block()
        }

    private suspend fun ApplicationTestBuilder.postPet(pet: Pet) =
        client.post("/api/v3/pet") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(pet))
        }

    @Test
    fun `addPet returns 200 with assigned id`() = testApp {
        val response = postPet(Pet(name = "Fluffy", photoUrls = listOf("http://example.com/img.jpg")))
        assertEquals(HttpStatusCode.OK, response.status)
        val body = json.decodeFromString<Pet>(response.bodyAsText())
        assertNotNull(body.id)
        assertEquals("Fluffy", body.name)
    }

    @Test
    fun `getPetById returns 404 for unknown`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/pet/9999").status)
    }

    @Test
    fun `getPetById returns pet after addPet`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "Rex", photoUrls = listOf("u"), status = Pet.Status.available)).bodyAsText()
        )
        val fetched = json.decodeFromString<Pet>(client.get("/api/v3/pet/${created.id}").bodyAsText())
        assertEquals("Rex", fetched.name)
    }

    @Test
    fun `findPetsByStatus returns only matching`() = testApp {
        postPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.available))
        postPet(Pet(name = "B", photoUrls = listOf("u"), status = Pet.Status.sold))
        val resp = client.get("/api/v3/pet/findByStatus?status=available")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertTrue(list.all { it.status == Pet.Status.available })
        assertTrue(list.any { it.name == "A" })
    }

    @Test
    fun `findPetsByTags returns matching pets`() = testApp {
        postPet(Pet(name = "Tagged", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "alpha"))))
        val resp = client.get("/api/v3/pet/findByTags?tags=alpha")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertTrue(list.any { it.name == "Tagged" })
    }

    @Test
    fun `updatePet returns 404 for missing pet`() = testApp {
        val response = client.put("/api/v3/pet") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(Pet(id = 9999L, name = "Ghost", photoUrls = listOf("u"))))
        }
        assertEquals(HttpStatusCode.NotFound, response.status)
    }

    @Test
    fun `updatePet updates existing pet`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "Old", photoUrls = listOf("u"))).bodyAsText()
        )
        val updated = json.decodeFromString<Pet>(client.put("/api/v3/pet") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(created.copy(name = "New")))
        }.bodyAsText())
        assertEquals("New", updated.name)
    }

    @Test
    fun `deletePet removes pet`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "ToDelete", photoUrls = listOf("u"))).bodyAsText()
        )
        assertEquals(HttpStatusCode.NoContent, client.delete("/api/v3/pet/${created.id}").status)
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/pet/${created.id}").status)
    }

    @Test
    fun `deletePet returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.delete("/api/v3/pet/9999").status)
    }

    @Test
    fun `uploadFile stores bytes and returns message`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "PicPet", photoUrls = listOf("u"))).bodyAsText()
        )
        val resp = client.post("/api/v3/pet/${created.id}/uploadImage") {
            setBody("fake-image-bytes".toByteArray())
        }
        assertEquals(HttpStatusCode.OK, resp.status)
        val apiResp = json.decodeFromString<ModelApiResponse>(resp.bodyAsText())
        assertTrue(apiResp.message!!.contains("bytes"))
    }
}
