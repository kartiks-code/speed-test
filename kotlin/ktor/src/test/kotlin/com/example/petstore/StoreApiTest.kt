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

private val json = Json { ignoreUnknownKeys = true; encodeDefaults = false }

class StoreApiTest {

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

    private suspend fun ApplicationTestBuilder.placeOrder(order: Order) =
        client.post("/api/v3/store/order") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(order))
        }

    @Test
    fun `getInventory returns empty map initially`() = testApp {
        val resp = client.get("/api/v3/store/inventory")
        assertEquals(HttpStatusCode.OK, resp.status)
    }

    @Test
    fun `getInventory counts by status`() = testApp {
        postPet(Pet(name = "P1", photoUrls = listOf("u"), status = Pet.Status.available))
        postPet(Pet(name = "P2", photoUrls = listOf("u"), status = Pet.Status.available))
        postPet(Pet(name = "P3", photoUrls = listOf("u"), status = Pet.Status.sold))
        val resp = client.get("/api/v3/store/inventory")
        assertEquals(HttpStatusCode.OK, resp.status)
        val map = json.decodeFromString<Map<String, Int>>(resp.bodyAsText())
        assertEquals(2, map["available"])
        assertEquals(1, map["sold"])
    }

    @Test
    fun `placeOrder returns order with id`() = testApp {
        val resp = placeOrder(Order(petId = 1L, quantity = 2, status = Order.Status.placed))
        assertEquals(HttpStatusCode.OK, resp.status)
        val body = json.decodeFromString<Order>(resp.bodyAsText())
        assertNotNull(body.id)
        assertEquals(Order.Status.placed, body.status)
    }

    @Test
    fun `getOrderById returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/store/order/9999").status)
    }

    @Test
    fun `getOrderById returns order after placeOrder`() = testApp {
        val placed = json.decodeFromString<Order>(
            placeOrder(Order(petId = 5L, quantity = 1, status = Order.Status.placed)).bodyAsText()
        )
        val fetched = json.decodeFromString<Order>(client.get("/api/v3/store/order/${placed.id}").bodyAsText())
        assertEquals(placed.id, fetched.id)
        assertEquals(5L, fetched.petId)
    }

    @Test
    fun `deleteOrder removes order`() = testApp {
        val placed = json.decodeFromString<Order>(
            placeOrder(Order(petId = 3L, quantity = 1)).bodyAsText()
        )
        assertEquals(HttpStatusCode.NoContent, client.delete("/api/v3/store/order/${placed.id}").status)
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/store/order/${placed.id}").status)
    }

    @Test
    fun `deleteOrder returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.delete("/api/v3/store/order/9999").status)
    }
}
