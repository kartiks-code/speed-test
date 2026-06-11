package com.example.petstore.repository

import com.example.petstore.models.*

interface PetstoreRepository {

    // --- Pet ---
    fun addPet(pet: Pet): Pet
    fun updatePet(pet: Pet): Pet
    fun findPetsByStatus(status: String): List<Pet>
    fun findPetsByTags(tags: List<String>): List<Pet>
    fun getPetById(petId: Long): Pet?
    fun updatePetWithForm(petId: Long, name: String?, status: String?): Pet
    fun deletePet(petId: Long)
    fun uploadFile(petId: Long, additionalMetadata: String?, bytes: ByteArray): ModelApiResponse

    // --- Store ---
    fun getInventory(): Map<String, Int>
    fun placeOrder(order: Order): Order
    fun getOrderById(orderId: Long): Order?
    fun deleteOrder(orderId: Long)

    // --- User ---
    fun createUser(user: User): User
    fun createUsersWithList(users: List<User>): User
    fun loginUser(username: String, password: String): String
    fun logoutUser()
    fun getUserByName(username: String): User?
    fun updateUser(username: String, user: User)
    fun deleteUser(username: String)
}
