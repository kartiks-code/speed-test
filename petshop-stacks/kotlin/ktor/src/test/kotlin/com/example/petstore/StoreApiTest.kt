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
        val map = json.decodeFromString<Map<String, Int>>(resp.bodyAsText())
        assertTrue(map.isEmpty())
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
        assertEquals(null, map["pending"])
    }

    @Test
    fun `getInventory counts all statuses`() = testApp {
        postPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.available))
        postPet(Pet(name = "B", photoUrls = listOf("u"), status = Pet.Status.sold))
        postPet(Pet(name = "C", photoUrls = listOf("u"), status = Pet.Status.pending))
        val resp = client.get("/api/v3/store/inventory")
        assertEquals(HttpStatusCode.OK, resp.status)
        val map = json.decodeFromString<Map<String, Int>>(resp.bodyAsText())
        assertEquals(1, map["available"])
        assertEquals(1, map["sold"])
        assertEquals(1, map["pending"])
        assertEquals(3, map.values.sum())
    }

    @Test
    fun `getInventory excludes pets with null status`() = testApp {
        postPet(Pet(name = "NullStatus", photoUrls = listOf("u"), status = null))
        val resp = client.get("/api/v3/store/inventory")
        assertEquals(HttpStatusCode.OK, resp.status)
        val map = json.decodeFromString<Map<String, Int>>(resp.bodyAsText())
        assertTrue(map.isEmpty())
    }

    @Test
    fun `placeOrder returns order with id`() = testApp {
        val resp = placeOrder(Order(petId = 1L, quantity = 2, status = Order.Status.placed))
        assertEquals(HttpStatusCode.OK, resp.status)
        val body = json.decodeFromString<Order>(resp.bodyAsText())
        assertNotNull(body.id)
        assertEquals(Order.Status.placed, body.status)
        assertEquals(1L, body.petId)
        assertEquals(2, body.quantity)
    }

    @Test
    fun `placeOrder assigns sequential ids`() = testApp {
        val o1 = json.decodeFromString<Order>(placeOrder(Order(petId = 1L, quantity = 1)).bodyAsText())
        val o2 = json.decodeFromString<Order>(placeOrder(Order(petId = 2L, quantity = 1)).bodyAsText())
        assertTrue(o2.id!! > o1.id!!)
    }

    @Test
    fun `placeOrder preserves all fields`() = testApp {
        val resp = placeOrder(Order(petId = 7L, quantity = 3, status = Order.Status.approved, complete = true))
        assertEquals(HttpStatusCode.OK, resp.status)
        val body = json.decodeFromString<Order>(resp.bodyAsText())
        assertEquals(7L, body.petId)
        assertEquals(3, body.quantity)
        assertEquals(Order.Status.approved, body.status)
        assertEquals(true, body.complete)
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
        val resp = client.get("/api/v3/store/order/${placed.id}")
        assertEquals(HttpStatusCode.OK, resp.status)
        val fetched = json.decodeFromString<Order>(resp.bodyAsText())
        assertEquals(placed.id, fetched.id)
        assertEquals(5L, fetched.petId)
        assertEquals(1, fetched.quantity)
        assertEquals(Order.Status.placed, fetched.status)
    }

    @Test
    fun `deleteOrder removes order`() = testApp {
        val placed = json.decodeFromString<Order>(
            placeOrder(Order(petId = 3L, quantity = 1)).bodyAsText()
        )
        assertEquals(HttpStatusCode.OK, client.delete("/api/v3/store/order/${placed.id}").status)
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/store/order/${placed.id}").status)
    }

    @Test
    fun `deleteOrder returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.delete("/api/v3/store/order/9999").status)
    }

    @Test
    fun `inventory count increases correctly`() = testApp {
        // First add - creates key with count 1
        postPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.available))
        val map1 = json.decodeFromString<Map<String, Int>>(
            client.get("/api/v3/store/inventory").bodyAsText()
        )
        assertEquals(1, map1["available"])

        // Second add - count should be 2, not some other value
        postPet(Pet(name = "B", photoUrls = listOf("u"), status = Pet.Status.available))
        val map2 = json.decodeFromString<Map<String, Int>>(
            client.get("/api/v3/store/inventory").bodyAsText()
        )
        assertEquals(2, map2["available"])
    }
}
