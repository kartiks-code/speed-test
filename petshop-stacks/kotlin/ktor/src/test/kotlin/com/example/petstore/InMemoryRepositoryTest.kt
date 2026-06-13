package com.example.petstore

import com.example.petstore.models.*
import com.example.petstore.repository.InMemoryPetstoreRepository
import com.example.petstore.repository.InvalidInputException
import com.example.petstore.repository.NotFoundException
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class InMemoryRepositoryTest {

    private lateinit var repo: InMemoryPetstoreRepository

    @BeforeEach
    fun setUp() {
        repo = InMemoryPetstoreRepository()
    }

    // -------------------------------------------------------------------------
    // Pet
    // -------------------------------------------------------------------------

    @Test
    fun `addPet assigns id when none provided`() = runBlocking {
        val pet = repo.addPet(Pet(name = "Fido", photoUrls = listOf("url")))
        assertNotNull(pet.id)
        assertEquals("Fido", pet.name)
    }

    @Test
    fun `addPet uses provided id`() = runBlocking {
        val pet = repo.addPet(Pet(id = 42L, name = "Rex", photoUrls = listOf("url")))
        assertEquals(42L, pet.id)
    }

    @Test
    fun `addPet stores pet retrievable by id`() = runBlocking {
        val added = repo.addPet(Pet(name = "Spot", photoUrls = listOf("u")))
        val found = repo.getPetById(added.id!!)
        assertNotNull(found)
        assertEquals("Spot", found.name)
    }

    @Test
    fun `getPetById returns null for missing pet`() = runBlocking {
        assertNull(repo.getPetById(999L))
    }

    @Test
    fun `updatePet changes pet fields`() = runBlocking {
        val created = repo.addPet(Pet(name = "Old", photoUrls = listOf("u"), status = Pet.Status.available))
        val updated = repo.updatePet(created.copy(name = "New", status = Pet.Status.sold))
        assertEquals("New", updated.name)
        assertEquals(Pet.Status.sold, updated.status)
    }

    @Test
    fun `updatePet throws NotFoundException for missing pet`() = runBlocking {
        assertThrows<NotFoundException> {
            runBlocking { repo.updatePet(Pet(id = 999L, name = "Ghost", photoUrls = listOf("u"))) }
        }
    }

    @Test
    fun `updatePet throws InvalidInputException when id is null`() = runBlocking {
        assertThrows<InvalidInputException> {
            runBlocking { repo.updatePet(Pet(name = "NoId", photoUrls = listOf("u"))) }
        }
    }

    @Test
    fun `findPetsByStatus returns only matching pets`() = runBlocking {
        repo.addPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.available))
        repo.addPet(Pet(name = "B", photoUrls = listOf("u"), status = Pet.Status.sold))
        repo.addPet(Pet(name = "C", photoUrls = listOf("u"), status = Pet.Status.pending))

        val available = repo.findPetsByStatus("available")
        assertEquals(1, available.size)
        assertEquals("A", available[0].name)

        val sold = repo.findPetsByStatus("sold")
        assertEquals(1, sold.size)
        assertEquals("B", sold[0].name)
    }

    @Test
    fun `findPetsByStatus returns empty for no match`() = runBlocking {
        repo.addPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.sold))
        val result = repo.findPetsByStatus("available")
        assertTrue(result.isEmpty())
    }

    @Test
    fun `findPetsByTags returns pets with matching tag`() = runBlocking {
        repo.addPet(Pet(name = "Tagged", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "alpha"))))
        repo.addPet(Pet(name = "Untagged", photoUrls = listOf("u"), tags = listOf(Tag(id = 2L, name = "beta"))))
        repo.addPet(Pet(name = "NoTags", photoUrls = listOf("u")))

        val result = repo.findPetsByTags(listOf("alpha"))
        assertEquals(1, result.size)
        assertEquals("Tagged", result[0].name)
    }

    @Test
    fun `findPetsByTags returns empty for no match`() = runBlocking {
        repo.addPet(Pet(name = "Foo", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "beta"))))
        val result = repo.findPetsByTags(listOf("alpha"))
        assertTrue(result.isEmpty())
    }

    @Test
    fun `findPetsByTags ignores pets with null tags`() = runBlocking {
        repo.addPet(Pet(name = "NoTags", photoUrls = listOf("u"), tags = null))
        val result = repo.findPetsByTags(listOf("alpha"))
        assertTrue(result.isEmpty())
    }

    @Test
    fun `updatePetWithForm updates name and status`() = runBlocking {
        val created = repo.addPet(Pet(name = "Original", photoUrls = listOf("u"), status = Pet.Status.available))
        val updated = repo.updatePetWithForm(created.id!!, "Renamed", "sold")
        assertEquals("Renamed", updated.name)
        assertEquals(Pet.Status.sold, updated.status)
    }

    @Test
    fun `updatePetWithForm keeps existing name when null`() = runBlocking {
        val created = repo.addPet(Pet(name = "KeepMe", photoUrls = listOf("u"), status = Pet.Status.available))
        val updated = repo.updatePetWithForm(created.id!!, null, "sold")
        assertEquals("KeepMe", updated.name)
        assertEquals(Pet.Status.sold, updated.status)
    }

    @Test
    fun `updatePetWithForm keeps existing status when null`() = runBlocking {
        val created = repo.addPet(Pet(name = "KeepStatus", photoUrls = listOf("u"), status = Pet.Status.pending))
        val updated = repo.updatePetWithForm(created.id!!, "NewName", null)
        assertEquals("NewName", updated.name)
        assertEquals(Pet.Status.pending, updated.status)
    }

    @Test
    fun `updatePetWithForm throws NotFoundException for missing pet`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.updatePetWithForm(999L, "X", null) }
        }
    }

    @Test
    fun `deletePet removes the pet`() = runBlocking {
        val created = repo.addPet(Pet(name = "ToDelete", photoUrls = listOf("u")))
        repo.deletePet(created.id!!)
        assertNull(repo.getPetById(created.id!!))
    }

    @Test
    fun `deletePet throws NotFoundException for missing pet`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.deletePet(999L) }
        }
    }

    @Test
    fun `uploadFile stores bytes and returns response`() = runBlocking {
        val pet = repo.addPet(Pet(name = "PicPet", photoUrls = listOf("u")))
        val bytes = "image-data".toByteArray()
        val response = repo.uploadFile(pet.id!!, "meta", bytes)
        assertEquals(200, response.code)
        assertTrue(response.message!!.contains("${bytes.size}"))
        assertTrue(response.message!!.contains("bytes"))
    }

    @Test
    fun `uploadFile throws NotFoundException for missing pet`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.uploadFile(999L, null, byteArrayOf()) }
        }
    }

    @Test
    fun `uploadFile increments photo id for each upload`() = runBlocking {
        val pet = repo.addPet(Pet(name = "P", photoUrls = listOf("u")))
        repo.uploadFile(pet.id!!, null, "a".toByteArray())
        repo.uploadFile(pet.id!!, null, "bb".toByteArray())
        // Both uploads succeed without error
    }

    // -------------------------------------------------------------------------
    // Store
    // -------------------------------------------------------------------------

    @Test
    fun `getInventory returns empty when no pets`() = runBlocking {
        val inv = repo.getInventory()
        assertTrue(inv.isEmpty())
    }

    @Test
    fun `getInventory counts pets by status`() = runBlocking {
        repo.addPet(Pet(name = "A", photoUrls = listOf("u"), status = Pet.Status.available))
        repo.addPet(Pet(name = "B", photoUrls = listOf("u"), status = Pet.Status.available))
        repo.addPet(Pet(name = "C", photoUrls = listOf("u"), status = Pet.Status.sold))
        repo.addPet(Pet(name = "D", photoUrls = listOf("u"), status = null))

        val inv = repo.getInventory()
        assertEquals(2, inv["available"])
        assertEquals(1, inv["sold"])
        assertEquals(null, inv["pending"])
    }

    @Test
    fun `getInventory excludes pets with null status`() = runBlocking {
        repo.addPet(Pet(name = "NullStatus", photoUrls = listOf("u"), status = null))
        val inv = repo.getInventory()
        assertTrue(inv.isEmpty())
    }

    @Test
    fun `placeOrder assigns id and stores order`() = runBlocking {
        val order = repo.placeOrder(Order(petId = 1L, quantity = 2, status = Order.Status.placed))
        assertNotNull(order.id)
        assertEquals(1L, order.petId)
        assertEquals(2, order.quantity)
        assertEquals(Order.Status.placed, order.status)
    }

    @Test
    fun `placeOrder uses provided id`() = runBlocking {
        val order = repo.placeOrder(Order(id = 77L, petId = 1L, quantity = 1))
        assertEquals(77L, order.id)
    }

    @Test
    fun `getOrderById returns order after place`() = runBlocking {
        val placed = repo.placeOrder(Order(petId = 5L, quantity = 3, status = Order.Status.approved))
        val found = repo.getOrderById(placed.id!!)
        assertNotNull(found)
        assertEquals(5L, found.petId)
        assertEquals(3, found.quantity)
        assertEquals(Order.Status.approved, found.status)
    }

    @Test
    fun `getOrderById returns null for missing order`() = runBlocking {
        assertNull(repo.getOrderById(999L))
    }

    @Test
    fun `deleteOrder removes order`() = runBlocking {
        val placed = repo.placeOrder(Order(petId = 1L, quantity = 1))
        repo.deleteOrder(placed.id!!)
        assertNull(repo.getOrderById(placed.id!!))
    }

    @Test
    fun `deleteOrder throws NotFoundException for missing order`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.deleteOrder(999L) }
        }
    }

    // -------------------------------------------------------------------------
    // User
    // -------------------------------------------------------------------------

    @Test
    fun `createUser assigns id and stores user`() = runBlocking {
        val user = repo.createUser(User(username = "alice", firstName = "Alice", email = "a@example.com"))
        assertNotNull(user.id)
        assertEquals("alice", user.username)
        assertEquals("Alice", user.firstName)
        assertEquals("a@example.com", user.email)
    }

    @Test
    fun `createUser uses provided id`() = runBlocking {
        val user = repo.createUser(User(id = 55L, username = "bob"))
        assertEquals(55L, user.id)
    }

    @Test
    fun `createUser throws InvalidInputException when username is null`() {
        assertThrows<InvalidInputException> {
            runBlocking { repo.createUser(User(username = null)) }
        }
    }

    @Test
    fun `getUserByName returns user after create`() = runBlocking {
        repo.createUser(User(username = "carol", firstName = "Carol"))
        val found = repo.getUserByName("carol")
        assertNotNull(found)
        assertEquals("Carol", found.firstName)
    }

    @Test
    fun `getUserByName returns null for missing user`() = runBlocking {
        assertNull(repo.getUserByName("nobody"))
    }

    @Test
    fun `createUsersWithList creates all and returns first`() = runBlocking {
        val users = listOf(
            User(username = "u1", firstName = "One"),
            User(username = "u2", firstName = "Two")
        )
        val first = repo.createUsersWithList(users)
        assertEquals("u1", first.username)
        assertNotNull(repo.getUserByName("u2"))
    }

    @Test
    fun `loginUser returns token for known user`() = runBlocking {
        repo.createUser(User(username = "dave", password = "secret"))
        val token = repo.loginUser("dave", "secret")
        assertEquals("logged-in-dave", token)
    }

    @Test
    fun `loginUser throws NotFoundException for unknown user`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.loginUser("nobody", "x") }
        }
    }

    @Test
    fun `logoutUser completes without error`() = runBlocking {
        repo.logoutUser()
    }

    @Test
    fun `updateUser changes user fields`() = runBlocking {
        repo.createUser(User(username = "eve", firstName = "Eve"))
        repo.updateUser("eve", User(username = "eve", firstName = "Evelyn", email = "e@example.com"))
        val updated = repo.getUserByName("eve")
        assertNotNull(updated)
        assertEquals("Evelyn", updated.firstName)
        assertEquals("e@example.com", updated.email)
    }

    @Test
    fun `updateUser preserves username`() = runBlocking {
        repo.createUser(User(username = "frank", firstName = "Frank"))
        repo.updateUser("frank", User(username = "frank", firstName = "Franklin"))
        val found = repo.getUserByName("frank")
        assertNotNull(found)
        assertEquals("frank", found.username)
    }

    @Test
    fun `updateUser throws NotFoundException for missing user`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.updateUser("nobody", User(username = "nobody")) }
        }
    }

    @Test
    fun `deleteUser removes user`() = runBlocking {
        repo.createUser(User(username = "grace"))
        repo.deleteUser("grace")
        assertNull(repo.getUserByName("grace"))
    }

    @Test
    fun `deleteUser throws NotFoundException for missing user`() {
        assertThrows<NotFoundException> {
            runBlocking { repo.deleteUser("nobody") }
        }
    }

    @Test
    fun `sequential pet ids increment`() = runBlocking {
        val p1 = repo.addPet(Pet(name = "P1", photoUrls = listOf("u")))
        val p2 = repo.addPet(Pet(name = "P2", photoUrls = listOf("u")))
        assertTrue(p2.id!! > p1.id!!)
    }

    @Test
    fun `sequential order ids increment`() = runBlocking {
        val o1 = repo.placeOrder(Order(petId = 1L, quantity = 1))
        val o2 = repo.placeOrder(Order(petId = 2L, quantity = 1))
        assertTrue(o2.id!! > o1.id!!)
    }

    @Test
    fun `findPetsByTags with multiple tags returns all matches`() = runBlocking {
        repo.addPet(Pet(name = "Alpha", photoUrls = listOf("u"), tags = listOf(Tag(id = 1L, name = "alpha"))))
        repo.addPet(Pet(name = "Beta", photoUrls = listOf("u"), tags = listOf(Tag(id = 2L, name = "beta"))))
        repo.addPet(Pet(name = "Gamma", photoUrls = listOf("u"), tags = listOf(Tag(id = 3L, name = "gamma"))))

        val result = repo.findPetsByTags(listOf("alpha", "beta"))
        assertEquals(2, result.size)
        assertTrue(result.any { it.name == "Alpha" })
        assertTrue(result.any { it.name == "Beta" })
    }

    @Test
    fun `createUser id increments from existing max`() = runBlocking {
        repo.createUser(User(id = 100L, username = "high"))
        val next = repo.createUser(User(username = "newuser"))
        assertTrue(next.id!! > 100L)
    }
}
