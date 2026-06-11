package com.example.petstore.repository

import com.example.petstore.models.*
import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.postgresql.util.PGobject
import java.sql.Connection
import java.sql.ResultSet
import javax.sql.DataSource

class PostgresPetstoreRepository(dataSource: DataSource? = null) : PetstoreRepository {

    private val ds: DataSource = dataSource ?: createDataSource()

    private fun conn(): Connection = ds.connection

    // -------------------------------------------------------------------------
    // Pet
    // -------------------------------------------------------------------------

    override fun addPet(pet: Pet): Pet {
        conn().use { c ->
            val id = pet.id ?: nextId(c, "pet")
            val saved = pet.copy(id = id)
            upsertPet(c, saved)
            return getPetById(id)!!
        }
    }

    override fun updatePet(pet: Pet): Pet {
        val id = pet.id ?: throw InvalidInputException("Pet id required for update")
        conn().use { c ->
            val existing = getPetById(id) ?: throw NotFoundException("Pet not found: $id")
            upsertPet(c, existing.copy(
                name = pet.name,
                photoUrls = pet.photoUrls,
                category = pet.category,
                tags = pet.tags,
                status = pet.status
            ))
            return getPetById(id)!!
        }
    }

    override fun findPetsByStatus(status: String): List<Pet> {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, name, photo_urls, category, tags, status::text FROM pet WHERE status::text = ?"
            ).use { ps ->
                ps.setString(1, status)
                return ps.executeQuery().use { rs -> rs.toPetList() }
            }
        }
    }

    override fun findPetsByTags(tags: List<String>): List<Pet> {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, name, photo_urls, category, tags, status::text FROM pet"
            ).use { ps ->
                val allPets = ps.executeQuery().use { rs -> rs.toPetList() }
                return allPets.filter { pet ->
                    pet.tags?.any { tag -> tags.contains(tag.name) } == true
                }
            }
        }
    }

    override fun getPetById(petId: Long): Pet? {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, name, photo_urls, category, tags, status::text FROM pet WHERE id = ?"
            ).use { ps ->
                ps.setLong(1, petId)
                return ps.executeQuery().use { rs -> if (rs.next()) rs.toPet() else null }
            }
        }
    }

    override fun updatePetWithForm(petId: Long, name: String?, status: String?): Pet {
        val existing = getPetById(petId) ?: throw NotFoundException("Pet not found: $petId")
        val updated = existing.copy(
            name = name ?: existing.name,
            status = status?.let { s -> Pet.Status.entries.find { it.value == s } } ?: existing.status
        )
        conn().use { c -> upsertPet(c, updated) }
        return getPetById(petId)!!
    }

    override fun deletePet(petId: Long) {
        getPetById(petId) ?: throw NotFoundException("Pet not found: $petId")
        conn().use { c ->
            c.prepareStatement("DELETE FROM pet WHERE id = ?").use { ps ->
                ps.setLong(1, petId)
                ps.executeUpdate()
            }
        }
    }

    override fun uploadFile(petId: Long, additionalMetadata: String?, bytes: ByteArray): ModelApiResponse {
        getPetById(petId) ?: throw NotFoundException("Pet not found: $petId")
        conn().use { c ->
            val photoId = nextId(c, "pet_photo")
            c.prepareStatement(
                "INSERT INTO pet_photo (id, pet_id, metadata, content) VALUES (?, ?, ?, ?) " +
                    "ON CONFLICT (id) DO UPDATE SET content = excluded.content"
            ).use { ps ->
                ps.setLong(1, photoId)
                ps.setLong(2, petId)
                ps.setString(3, additionalMetadata)
                ps.setBytes(4, bytes)
                ps.executeUpdate()
            }
        }
        return ModelApiResponse(code = 200, type = "unknown", message = "File uploaded: ${bytes.size} bytes")
    }

    private fun upsertPet(c: Connection, pet: Pet) {
        c.prepareStatement(
            """INSERT INTO pet (id, name, photo_urls, category, tags, status)
               VALUES (?, ?, ?, ?, ?, ?::pet_status)
               ON CONFLICT (id) DO UPDATE SET
                 name = excluded.name,
                 photo_urls = excluded.photo_urls,
                 category = excluded.category,
                 tags = excluded.tags,
                 status = excluded.status"""
        ).use { ps ->
            ps.setLong(1, pet.id!!)
            ps.setString(2, pet.name)
            ps.setObject(3, jsonObj(Json.encodeToString(pet.photoUrls)))
            ps.setString(4, pet.category?.let { Json.encodeToString(it) })
            ps.setObject(5, jsonObj(Json.encodeToString(pet.tags ?: emptyList<Tag>())))
            ps.setString(6, pet.status?.value)
            ps.executeUpdate()
        }
    }

    // -------------------------------------------------------------------------
    // Store
    // -------------------------------------------------------------------------

    override fun getInventory(): Map<String, Int> {
        conn().use { c ->
            c.prepareStatement(
                "SELECT status::text, COUNT(*) AS cnt FROM pet GROUP BY status"
            ).use { ps ->
                val result = mutableMapOf<String, Int>()
                ps.executeQuery().use { rs ->
                    while (rs.next()) {
                        result[rs.getString("status")] = rs.getInt("cnt")
                    }
                }
                return result
            }
        }
    }

    override fun placeOrder(order: Order): Order {
        conn().use { c ->
            val id = order.id ?: nextId(c, "\"order\"")
            val saved = order.copy(id = id)
            upsertOrder(c, saved)
            return getOrderById(id)!!
        }
    }

    override fun getOrderById(orderId: Long): Order? {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, pet_id, quantity, ship_date, status::text, complete FROM \"order\" WHERE id = ?"
            ).use { ps ->
                ps.setLong(1, orderId)
                return ps.executeQuery().use { rs -> if (rs.next()) rs.toOrder() else null }
            }
        }
    }

    override fun deleteOrder(orderId: Long) {
        getOrderById(orderId) ?: throw NotFoundException("Order not found: $orderId")
        conn().use { c ->
            c.prepareStatement("DELETE FROM \"order\" WHERE id = ?").use { ps ->
                ps.setLong(1, orderId)
                ps.executeUpdate()
            }
        }
    }

    private fun upsertOrder(c: Connection, order: Order) {
        c.prepareStatement(
            """INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
               VALUES (?, ?, ?, ?::timestamp, ?::order_status, ?)
               ON CONFLICT (id) DO UPDATE SET
                 pet_id = excluded.pet_id,
                 quantity = excluded.quantity,
                 ship_date = excluded.ship_date,
                 status = excluded.status,
                 complete = excluded.complete"""
        ).use { ps ->
            ps.setLong(1, order.id!!)
            order.petId?.let { ps.setLong(2, it) } ?: ps.setNull(2, java.sql.Types.BIGINT)
            order.quantity?.let { ps.setInt(3, it) } ?: ps.setNull(3, java.sql.Types.INTEGER)
            ps.setString(4, order.shipDate)
            ps.setString(5, order.status?.value)
            order.complete?.let { ps.setBoolean(6, it) } ?: ps.setNull(6, java.sql.Types.BOOLEAN)
            ps.executeUpdate()
        }
    }

    // -------------------------------------------------------------------------
    // User
    // -------------------------------------------------------------------------

    override fun createUser(user: User): User {
        conn().use { c ->
            val id = user.id ?: nextId(c, "\"user\"", "id")
            upsertUser(c, user.copy(id = id))
            return getUserByName(user.username ?: "")!!
        }
    }

    override fun createUsersWithList(users: List<User>): User {
        conn().use { c ->
            for (user in users) {
                val id = user.id ?: nextId(c, "\"user\"", "id")
                upsertUser(c, user.copy(id = id))
            }
        }
        return getUserByName(users.first().username ?: "")!!
    }

    override fun loginUser(username: String, password: String): String {
        getUserByName(username) ?: throw NotFoundException("User not found: $username")
        return "logged-in-$username"
    }

    override fun logoutUser() { /* stateless no-op */ }

    override fun getUserByName(username: String): User? {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, username, first_name, last_name, email, password, phone, user_status FROM \"user\" WHERE username = ?"
            ).use { ps ->
                ps.setString(1, username)
                return ps.executeQuery().use { rs -> if (rs.next()) rs.toUser() else null }
            }
        }
    }

    override fun updateUser(username: String, user: User) {
        getUserByName(username) ?: throw NotFoundException("User not found: $username")
        conn().use { c -> upsertUser(c, user.copy(username = username)) }
    }

    override fun deleteUser(username: String) {
        getUserByName(username) ?: throw NotFoundException("User not found: $username")
        conn().use { c ->
            c.prepareStatement("DELETE FROM \"user\" WHERE username = ?").use { ps ->
                ps.setString(1, username)
                ps.executeUpdate()
            }
        }
    }

    private fun upsertUser(c: Connection, user: User) {
        val uname = user.username ?: throw InvalidInputException("Username required")
        c.prepareStatement(
            """INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT (username) DO UPDATE SET
                 id = excluded.id,
                 first_name = excluded.first_name,
                 last_name = excluded.last_name,
                 email = excluded.email,
                 password = excluded.password,
                 phone = excluded.phone,
                 user_status = excluded.user_status"""
        ).use { ps ->
            user.id?.let { ps.setLong(1, it) } ?: ps.setNull(1, java.sql.Types.BIGINT)
            ps.setString(2, uname)
            ps.setString(3, user.firstName)
            ps.setString(4, user.lastName)
            ps.setString(5, user.email)
            ps.setString(6, user.password)
            ps.setString(7, user.phone)
            user.userStatus?.let { ps.setInt(8, it) } ?: ps.setNull(8, java.sql.Types.INTEGER)
            ps.executeUpdate()
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    private fun nextId(c: Connection, table: String, col: String = "id"): Long {
        c.prepareStatement("SELECT COALESCE(MAX($col), 0) + 1 FROM $table").use { ps ->
            return ps.executeQuery().use { rs ->
                rs.next()
                rs.getLong(1)
            }
        }
    }

    private fun jsonObj(value: String): PGobject {
        val obj = PGobject()
        obj.type = "json"
        obj.value = value
        return obj
    }

    private fun ResultSet.toPet(): Pet {
        val categoryJson = getString("category")
        val tagsJson = getString("tags")
        val photoUrlsJson = getString("photo_urls")
        return Pet(
            id = getLong("id"),
            name = getString("name"),
            photoUrls = if (photoUrlsJson != null) Json.decodeFromString(photoUrlsJson) else emptyList(),
            category = if (categoryJson != null) Json.decodeFromString(categoryJson) else null,
            tags = if (tagsJson != null) Json.decodeFromString(tagsJson) else null,
            status = getString("status")?.let { s -> Pet.Status.entries.find { it.value == s } }
        )
    }

    private fun ResultSet.toPetList(): List<Pet> {
        val list = mutableListOf<Pet>()
        while (next()) list.add(toPet())
        return list
    }

    private fun ResultSet.toOrder(): Order {
        return Order(
            id = getLong("id"),
            petId = getLong("pet_id").takeIf { it != 0L },
            quantity = getInt("quantity").takeIf { !wasNull() },
            shipDate = getString("ship_date"),
            status = getString("status")?.let { s -> Order.Status.entries.find { it.value == s } },
            complete = getBoolean("complete").takeIf { !wasNull() }
        )
    }

    private fun ResultSet.toUser(): User {
        return User(
            id = getLong("id").takeIf { it != 0L },
            username = getString("username"),
            firstName = getString("first_name"),
            lastName = getString("last_name"),
            email = getString("email"),
            password = getString("password"),
            phone = getString("phone"),
            userStatus = getInt("user_status").takeIf { !wasNull() }
        )
    }
}

// -------------------------------------------------------------------------
// Domain exceptions
// -------------------------------------------------------------------------

class NotFoundException(message: String) : RuntimeException(message)
class InvalidInputException(message: String) : RuntimeException(message)

// -------------------------------------------------------------------------
// DataSource factory
// -------------------------------------------------------------------------

fun createDataSource(): HikariDataSource {
    val host = System.getenv("POSTGRES_HOST") ?: "localhost"
    val port = System.getenv("POSTGRES_PORT")?.toInt() ?: 5434
    val db = System.getenv("POSTGRES_DB") ?: "kotlin-ktor"
    val user = System.getenv("POSTGRES_USER") ?: "myuser"
    val pass = System.getenv("POSTGRES_PASSWORD") ?: "mypassword"
    val url = System.getenv("DATABASE_URL") ?: "jdbc:postgresql://$host:$port/$db"

    return HikariDataSource(HikariConfig().apply {
        jdbcUrl = url
        username = user
        password = pass
        driverClassName = "org.postgresql.Driver"
        maximumPoolSize = 10
    })
}
