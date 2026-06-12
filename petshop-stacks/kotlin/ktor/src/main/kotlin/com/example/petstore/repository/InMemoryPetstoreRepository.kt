package com.example.petstore.repository

import com.example.petstore.models.*
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

class InMemoryPetstoreRepository : PetstoreRepository {

    private val pets = ConcurrentHashMap<Long, Pet>()
    private val orders = ConcurrentHashMap<Long, Order>()
    private val users = ConcurrentHashMap<String, User>()
    private val photos = ConcurrentHashMap<Long, ByteArray>()

    private val petIdSeq = AtomicLong(0)
    private val orderIdSeq = AtomicLong(0)
    private val photoIdSeq = AtomicLong(0)

    private fun nextPetId() = (pets.keys.maxOrNull() ?: 0L) + 1L
    private fun nextOrderId() = (orders.keys.maxOrNull() ?: 0L) + 1L
    private fun nextPhotoId() = (photos.keys.maxOrNull() ?: 0L) + 1L

    // -------------------------------------------------------------------------
    // Pet
    // -------------------------------------------------------------------------

    override fun addPet(pet: Pet): Pet {
        val id = pet.id ?: nextPetId()
        val saved = pet.copy(id = id)
        pets[id] = saved
        return saved
    }

    override fun updatePet(pet: Pet): Pet {
        val id = pet.id ?: throw InvalidInputException("Pet id required for update")
        pets[id] ?: throw NotFoundException("Pet not found: $id")
        val updated = pet.copy(id = id)
        pets[id] = updated
        return updated
    }

    override fun findPetsByStatus(status: String): List<Pet> =
        pets.values.filter { it.status?.value == status }

    override fun findPetsByTags(tags: List<String>): List<Pet> =
        pets.values.filter { pet -> pet.tags?.any { tag -> tags.contains(tag.name) } == true }

    override fun getPetById(petId: Long): Pet? = pets[petId]

    override fun updatePetWithForm(petId: Long, name: String?, status: String?): Pet {
        val existing = pets[petId] ?: throw NotFoundException("Pet not found: $petId")
        val updated = existing.copy(
            name = name ?: existing.name,
            status = status?.let { s -> Pet.Status.entries.find { it.value == s } } ?: existing.status
        )
        pets[petId] = updated
        return updated
    }

    override fun deletePet(petId: Long) {
        pets.remove(petId) ?: throw NotFoundException("Pet not found: $petId")
    }

    override fun uploadFile(petId: Long, additionalMetadata: String?, bytes: ByteArray): ModelApiResponse {
        pets[petId] ?: throw NotFoundException("Pet not found: $petId")
        val id = nextPhotoId()
        photos[id] = bytes
        return ModelApiResponse(code = 200, type = "unknown", message = "File uploaded: ${bytes.size} bytes")
    }

    // -------------------------------------------------------------------------
    // Store
    // -------------------------------------------------------------------------

    override fun getInventory(): Map<String, Int> {
        val result = mutableMapOf<String, Int>()
        for (pet in pets.values) {
            val s = pet.status?.value ?: continue
            result[s] = (result[s] ?: 0) + 1
        }
        return result
    }

    override fun placeOrder(order: Order): Order {
        val id = order.id ?: nextOrderId()
        val saved = order.copy(id = id)
        orders[id] = saved
        return saved
    }

    override fun getOrderById(orderId: Long): Order? = orders[orderId]

    override fun deleteOrder(orderId: Long) {
        orders.remove(orderId) ?: throw NotFoundException("Order not found: $orderId")
    }

    // -------------------------------------------------------------------------
    // User
    // -------------------------------------------------------------------------

    override fun createUser(user: User): User {
        val uname = user.username ?: throw InvalidInputException("Username required")
        val id = user.id ?: ((users.values.mapNotNull { it.id }.maxOrNull() ?: 0L) + 1L)
        val saved = user.copy(id = id)
        users[uname] = saved
        return saved
    }

    override fun createUsersWithList(users: List<User>): User {
        for (user in users) createUser(user)
        return getUserByName(users.first().username ?: "")!!
    }

    override fun loginUser(username: String, password: String): String {
        users[username] ?: throw NotFoundException("User not found: $username")
        return "logged-in-$username"
    }

    override fun logoutUser() { /* stateless no-op */ }

    override fun getUserByName(username: String): User? = users[username]

    override fun updateUser(username: String, user: User) {
        users[username] ?: throw NotFoundException("User not found: $username")
        users[username] = user.copy(username = username)
    }

    override fun deleteUser(username: String) {
        users.remove(username) ?: throw NotFoundException("User not found: $username")
    }
}
