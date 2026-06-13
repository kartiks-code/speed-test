package com.example.petstore.repository

import com.example.petstore.models.*

interface PetstoreRepository {

    // --- Pet ---
    suspend fun addPet(pet: Pet): Pet
    suspend fun updatePet(pet: Pet): Pet
    suspend fun findPetsByStatus(status: String): List<Pet>
    suspend fun findPetsByTags(tags: List<String>): List<Pet>
    suspend fun getPetById(petId: Long): Pet?
    suspend fun updatePetWithForm(petId: Long, name: String?, status: String?): Pet
    suspend fun deletePet(petId: Long)
    suspend fun uploadFile(petId: Long, additionalMetadata: String?, bytes: ByteArray): ModelApiResponse

    // --- Store ---
    suspend fun getInventory(): Map<String, Int>
    suspend fun placeOrder(order: Order): Order
    suspend fun getOrderById(orderId: Long): Order?
    suspend fun deleteOrder(orderId: Long)

    // --- User ---
    suspend fun createUser(user: User): User
    suspend fun createUsersWithList(users: List<User>): User
    suspend fun loginUser(username: String, password: String): String
    suspend fun logoutUser()
    suspend fun getUserByName(username: String): User?
    suspend fun updateUser(username: String, user: User)
    suspend fun deleteUser(username: String)
}
