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
        assertEquals(listOf("http://example.com/img.jpg"), body.photoUrls)
    }

    @Test
    fun `addPet returns 400 for blank name`() = testApp {
        val response = postPet(Pet(name = "", photoUrls = listOf("u")))
        assertEquals(HttpStatusCode.BadRequest, response.status)
    }

    @Test
    fun `addPet returns 400 for whitespace-only name`() = testApp {
        val response = postPet(Pet(name = "   ", photoUrls = listOf("u")))
        assertEquals(HttpStatusCode.BadRequest, response.status)
    }

    @Test
    fun `addPet preserves status field`() = testApp {
        val response = postPet(Pet(name = "StatusPet", photoUrls = listOf("u"), status = Pet.Status.pending))
        assertEquals(HttpStatusCode.OK, response.status)
        val body = json.decodeFromString<Pet>(response.bodyAsText())
        assertEquals(Pet.Status.pending, body.status)
        assertEquals("StatusPet", body.name)
    }

    @Test
    fun `addPet preserves tags`() = testApp {
        val response = postPet(Pet(name = "TagPet", photoUrls = listOf("u"),
            tags = listOf(Tag(id = 1L, name = "fluffy"))))
        assertEquals(HttpStatusCode.OK, response.status)
        val body = json.decodeFromString<Pet>(response.bodyAsText())
        assertEquals(1, body.tags?.size)
        assertEquals("fluffy", body.tags?.first()?.name)
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
        val resp = client.get("/api/v3/pet/${created.id}")
        assertEquals(HttpStatusCode.OK, resp.status)
        val fetched = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals("Rex", fetched.name)
        assertEquals(created.id, fetched.id)
        assertEquals(Pet.Status.available, fetched.status)
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
        assertTrue(list.none { it.name == "B" })
    }

    @Test
    fun `findPetsByStatus defaults to available when no param`() = testApp {
        postPet(Pet(name = "Avail", photoUrls = listOf("u"), status = Pet.Status.available))
        postPet(Pet(name = "Sold", photoUrls = listOf("u"), status = Pet.Status.sold))
        val resp = client.get("/api/v3/pet/findByStatus")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertTrue(list.any { it.name == "Avail" })
        assertTrue(list.none { it.name == "Sold" })
    }

    @Test
    fun `findPetsByStatus returns empty list for no matches`() = testApp {
        postPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.sold))
        val resp = client.get("/api/v3/pet/findByStatus?status=pending")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertEquals(0, list.size)
    }

    @Test
    fun `findPetsByTags returns matching pets`() = testApp {
        postPet(Pet(name = "Tagged", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "alpha"))))
        val resp = client.get("/api/v3/pet/findByTags?tags=alpha")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertEquals(1, list.size)
        assertEquals("Tagged", list[0].name)
    }

    @Test
    fun `findPetsByTags with no tags returns empty list`() = testApp {
        postPet(Pet(name = "NoTags", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "alpha"))))
        val resp = client.get("/api/v3/pet/findByTags")
        assertEquals(HttpStatusCode.OK, resp.status)
        val list = json.decodeFromString<List<Pet>>(resp.bodyAsText())
        assertEquals(0, list.size)
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
    fun `updatePet returns 400 when id is null`() = testApp {
        val response = client.put("/api/v3/pet") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(Pet(name = "NoId", photoUrls = listOf("u"))))
        }
        assertEquals(HttpStatusCode.BadRequest, response.status)
    }

    @Test
    fun `updatePet updates existing pet`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "Old", photoUrls = listOf("u"), status = Pet.Status.available)).bodyAsText()
        )
        val resp = client.put("/api/v3/pet") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(created.copy(name = "New", status = Pet.Status.sold)))
        }
        assertEquals(HttpStatusCode.OK, resp.status)
        val updated = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals("New", updated.name)
        assertEquals(Pet.Status.sold, updated.status)
        assertEquals(created.id, updated.id)
    }

    @Test
    fun `updatePetWithForm updates name via query param`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "FormPet", photoUrls = listOf("u"), status = Pet.Status.available)).bodyAsText()
        )
        val resp = client.post("/api/v3/pet/${created.id}?name=Updated")
        assertEquals(HttpStatusCode.OK, resp.status)
        val updated = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals("Updated", updated.name)
        assertEquals(Pet.Status.available, updated.status)
        assertEquals(created.id, updated.id)
    }

    @Test
    fun `updatePetWithForm updates status via query param`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "StatusChangePet", photoUrls = listOf("u"), status = Pet.Status.available)).bodyAsText()
        )
        val resp = client.post("/api/v3/pet/${created.id}?status=sold")
        assertEquals(HttpStatusCode.OK, resp.status)
        val updated = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals(Pet.Status.sold, updated.status)
        assertEquals("StatusChangePet", updated.name)
    }

    @Test
    fun `updatePetWithForm with no params keeps existing values`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "NoChange", photoUrls = listOf("u"), status = Pet.Status.pending)).bodyAsText()
        )
        val resp = client.post("/api/v3/pet/${created.id}")
        assertEquals(HttpStatusCode.OK, resp.status)
        val updated = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals("NoChange", updated.name)
        assertEquals(Pet.Status.pending, updated.status)
    }

    @Test
    fun `updatePetWithForm returns 404 for missing pet`() = testApp {
        val resp = client.post("/api/v3/pet/9999?name=Ghost")
        assertEquals(HttpStatusCode.NotFound, resp.status)
    }

    @Test
    fun `updatePetWithForm updates both name and status`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "BothChange", photoUrls = listOf("u"), status = Pet.Status.available)).bodyAsText()
        )
        val resp = client.post("/api/v3/pet/${created.id}?name=NewName&status=pending")
        assertEquals(HttpStatusCode.OK, resp.status)
        val updated = json.decodeFromString<Pet>(resp.bodyAsText())
        assertEquals("NewName", updated.name)
        assertEquals(Pet.Status.pending, updated.status)
    }

    @Test
    fun `deletePet removes pet`() = testApp {
        val created = json.decodeFromString<Pet>(
            postPet(Pet(name = "ToDelete", photoUrls = listOf("u"))).bodyAsText()
        )
        assertEquals(HttpStatusCode.OK, client.delete("/api/v3/pet/${created.id}").status)
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
        val content = "fake-image-bytes"
        val resp = client.post("/api/v3/pet/${created.id}/uploadImage") {
            setBody(content.toByteArray())
        }
        assertEquals(HttpStatusCode.OK, resp.status)
        val apiResp = json.decodeFromString<ModelApiResponse>(resp.bodyAsText())
        assertEquals(200, apiResp.code)
        assertTrue(apiResp.message!!.contains("${content.length}"))
        assertTrue(apiResp.message!!.contains("bytes"))
    }

    @Test
    fun `uploadFile returns 404 for missing pet`() = testApp {
        val resp = client.post("/api/v3/pet/9999/uploadImage") {
            setBody("data".toByteArray())
        }
        assertEquals(HttpStatusCode.NotFound, resp.status)
    }

    @Test
    fun `addPet assigns sequential ids`() = testApp {
        val p1 = json.decodeFromString<Pet>(postPet(Pet(name = "P1", photoUrls = listOf("u"))).bodyAsText())
        val p2 = json.decodeFromString<Pet>(postPet(Pet(name = "P2", photoUrls = listOf("u"))).bodyAsText())
        assertTrue(p2.id!! > p1.id!!)
    }
}
