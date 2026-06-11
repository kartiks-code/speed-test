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
    }

    @Test
    fun `getUserByName returns 404 for missing`() = testApp {
        assertEquals(HttpStatusCode.NotFound, client.get("/api/v3/user/nobody").status)
    }

    @Test
    fun `getUserByName returns user after create`() = testApp {
        postUser(User(username = "bob", firstName = "Bob"))
        val fetched = json.decodeFromString<User>(client.get("/api/v3/user/bob").bodyAsText())
        assertEquals("bob", fetched.username)
        assertEquals("Bob", fetched.firstName)
    }

    @Test
    fun `createUsersWithListInput creates all users`() = testApp {
        val users = listOf(User(username = "user1", firstName = "One"), User(username = "user2", firstName = "Two"))
        val resp = client.post("/api/v3/user/createWithList") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(users))
        }
        assertEquals(HttpStatusCode.OK, resp.status)
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/user1").status)
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/user2").status)
    }

    @Test
    fun `updateUser changes user fields`() = testApp {
        postUser(User(username = "carol", firstName = "Carol"))
        client.put("/api/v3/user/carol") {
            contentType(ContentType.Application.Json)
            setBody(json.encodeToString(User(username = "carol", firstName = "Caroline", email = "c@example.com")))
        }
        val fetched = json.decodeFromString<User>(client.get("/api/v3/user/carol").bodyAsText())
        assertEquals("Caroline", fetched.firstName)
        assertEquals("c@example.com", fetched.email)
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
    fun `logoutUser returns 200`() = testApp {
        assertEquals(HttpStatusCode.OK, client.get("/api/v3/user/logout").status)
    }
}
