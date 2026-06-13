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

class UserApiTest {

    private fun testApp(block: suspend ApplicationTestBuilder.() -> Unit) =
        testApplication {
            application { configureModule(InMemoryPetstoreRepository()) }
            block()
        }

    private suspend fun ApplicationTestBuilder.postUser(user: User) =
        client.post("/api/v3/user") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(user))
        }

    @Test
    fun `createUser returns user with id`() = testApp {
        val resp = postUser(User(username = "alice", firstName = "Alice", lastName = "Smith", email = "alice@example.com", password = "secret"))
        assertEquals(HttpStatusCode.OK, resp.status)
        val body = json.decodeFromString<User>(resp.bodyAsText())
        assertNotNull(body.id)
        assertEquals("alice", body.username)
        assertEquals("Alice", body.firstName)
        assertEquals("Smith", body.lastName)
        assertEquals("alice@example.com", body.email)
    }

    @Test
    fun `createUser assigns sequential ids`() = testApp {
        val u1 = json.decodeFromString<User>(postUser(User(username = "u1")).bodyAsText())
        val u2 = json.decodeFromString<User>(postUser(User(username = "u2")).bodyAsText())
        assertTrue(u2.id!! > u1.id!!)
    }

    @Test
    fun `getUserByName returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/user/nobody").status)
    }

    @Test
    fun `getUserByName returns user after create`() = testApp {
        postUser(User(username = "bob", firstName = "Bob", email = "bob@example.com"))
        val resp = client.get("/api/v3/user/bob")
        assertEquals(HttpStatusCode.OK, resp.status)
        val fetched = json.decodeFromString<User>(resp.bodyAsText())
        assertEquals("bob", fetched.username)
        assertEquals("Bob", fetched.firstName)
        assertEquals("bob@example.com", fetched.email)
    }

    @Test
    fun `createUsersWithListInput creates all users`() = testApp {
        val users = listOf(User(username = "user1", firstName = "One"), User(username = "user2", firstName = "Two"))
        val resp = client.post("/api/v3/user/createWithList") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(users))
        }
        assertEquals(HttpStatusCode.OK, resp.status)
        // returns first user
        val body = json.decodeFromString<User>(resp.bodyAsText())
        assertEquals("user1", body.username)
        assertEquals("One", body.firstName)
        // verify second user also created
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/user1").status)
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/user2").status)
    }

    @Test
    fun `createUsersWithListInput returns 400 for empty list`() = testApp {
        val resp = client.post("/api/v3/user/createWithList") {
            contentType(ContentType.Application.Json)
            setBody("[]")
        }
        assertEquals(HttpStatusCode.BadRequest, resp.status)
    }

    @Test
    fun `updateUser changes user fields`() = testApp {
        postUser(User(username = "carol", firstName = "Carol"))
        val updateResp = client.put("/api/v3/user/carol") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(User(username = "carol", firstName = "Caroline", email = "c@example.com")))
        }
        assertEquals(HttpStatusCode.NoContent, updateResp.status)
        val fetched = json.decodeFromString<User>(client.get("/api/v3/user/carol").bodyAsText())
        assertEquals("Caroline", fetched.firstName)
        assertEquals("c@example.com", fetched.email)
        assertEquals("carol", fetched.username)
    }

    @Test
    fun `updateUser returns 404 for missing user`() = testApp {
        val resp = client.put("/api/v3/user/nobody") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(User(username = "nobody", firstName = "Ghost")))
        }
        assertEquals(HttpStatusCode.NotFound, resp.status)
    }

    @Test
    fun `deleteUser removes user`() = testApp {
        postUser(User(username = "dave"))
        assertEquals(HttpStatusCode.NoContent, client.delete("/api/v3/user/dave").status)
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/user/dave").status)
    }

    @Test
    fun `deleteUser returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.delete("/api/v3/user/nobody").status)
    }

    @Test
    fun `loginUser returns token`() = testApp {
        postUser(User(username = "eve", password = "pass"))
        val resp = client.get("/api/v3/user/login?username=eve&password=pass")
        assertEquals(HttpStatusCode.OK, resp.status)
        val token = resp.bodyAsText().trim('"')
        assertEquals("logged-in-eve", token)
    }

    @Test
    fun `loginUser returns 404 for unknown user`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/user/login?username=unknown&password=x").status)
    }

    @Test
    fun `loginUser returns 400 when username missing`() = testApp {
        val resp = client.get("/api/v3/user/login?password=pass")
        assertEquals(HttpStatusCode.BadRequest, resp.status)
    }

    @Test
    fun `loginUser returns 400 when password missing`() = testApp {
        val resp = client.get("/api/v3/user/login?username=eve")
        assertEquals(HttpStatusCode.BadRequest, resp.status)
    }

    @Test
    fun `loginUser returns 400 when both params missing`() = testApp {
        assertEquals(HttpStatusCode.BadRequest, client.get("/api/v3/user/login").status)
    }

    @Test
    fun `logoutUser returns 200`() = testApp {
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/logout").status)
    }

    @Test
    fun `createUser then update then delete lifecycle`() = testApp {
        // Create
        postUser(User(username = "lifecycle", firstName = "Original"))
        // Update
        client.put("/api/v3/user/lifecycle") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(User(username = "lifecycle", firstName = "Updated")))
        }
        val updated = json.decodeFromString<User>(client.get("/api/v3/user/lifecycle").bodyAsText())
        assertEquals("Updated", updated.firstName)
        // Delete
        assertEquals(HttpStatusCode.NoContent, client.delete("/api/v3/user/lifecycle").status)
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/user/lifecycle").status)
    }
}
