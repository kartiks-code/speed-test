package com.example.petstore.repository

import com.example.petstore.models.*
import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.postgresql.util.PGobject
import java.sql.Connection
import java.sql.ResultSet
import javax.sql.DataSource

/**
 * JDBC-backed repository. All public methods are suspend functions that shift
 * the blocking JDBC work onto [Dispatchers.IO], so Ktor's Netty event-loop
 * threads are never pinned during database I/O. Each method's JDBC work runs
 * entirely within a single withContext(Dispatchers.IO) block.
 */
class PostgresPetstoreRepository(dataSource: DataSource? = null) : PetstoreRepository {

    private val ds: DataSource = dataSource ?: createDataSource()

    private fun conn(): Connection = ds.connection

    // -------------------------------------------------------------------------
    // Pet
    // -------------------------------------------------------------------------

    override suspend fun addPet(pet: Pet): Pet = withContext(Dispatchers.IO) {
        conn().use { c ->
            val id = pet.id ?: nextId(c, "pet")
            val saved = pet.copy(id = id)
            upsertPet(c, saved)
            fetchPetById(c, id)!!
        }
    }

    override suspend fun updatePet(pet: Pet): Pet {
        val id = pet.id ?: throw InvalidInputException("Pet id required for update")
        return withContext(Dispatchers.IO) {
            conn().use { c ->
                val existing = fetchPetById(c, id) ?: throw NotFoundException("Pet not found: $id")
                upsertPet(c, existing.copy(
                    name = pet.name,
                    photoUrls = pet.photoUrls,
                    category = pet.category,
                    tags = pet.tags,
                    status = pet.status
                ))
                fetchPetById(c, id)!!
            }
        }
    }

    override suspend fun findPetsByStatus(status: String): List<Pet> = withContext(Dispatchers.IO) {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, name, photo_urls, category, tags, status::text FROM pet WHERE status::text = ?"
            ).use { ps ->
                ps.setString(1, status)
                ps.executeQuery().use { rs -> rs.toPetList() }
            }
        }
    }

    override suspend fun findPetsByTags(tags: List<String>): List<Pet> = withContext(Dispatchers.IO) {
        conn().use { c ->
            c.prepareStatement(
                "SELECT id, name, photo_urls, category, tags, status::text FROM pet"
            ).use { ps ->
                val allPets = ps.executeQuery().use { rs -> rs.toPetList() }
                allPets.filter { pet ->
                    pet.tags?.any { tag -> tags.contains(tag.name) } == true
                }
            }
        }
    }

    override suspend fun getPetById(petId: Long): Pet? = withContext(Dispatchers.IO) {
        fetchPetById(petId)
    }

    override suspend fun updatePetWithForm(petId: Long, name: String?, status: String?): Pet =
        withContext(Dispatchers.IO) {
            conn().use { c ->
                val existing = fetchPetById(c, petId) ?: throw NotFoundException("Pet not found: $petId")
                val updated = existing.copy(
                    name = name ?: existing.name,
                    status = status?.let { s -> Pet.Status.entries.find { it.value == s } } ?: existing.status
                )
                upsertPet(c, updated)
                fetchPetById(c, petId)!!
            }
        }

    override suspend fun deletePet(petId: Long): Unit = withContext(Dispatchers.IO) {
        conn().use { c ->
            fetchPetById(c, petId) ?: throw NotFoundException("Pet not found: $petId")
            c.prepareStatement("DELETE FROM pet WHERE id = ?").use { ps ->
                ps.setLong(1, petId)
                ps.executeUpdate()
            }
        }
    }

    override suspend fun uploadFile(petId: Long, additionalMetadata: String?, bytes: ByteArray): ModelApiResponse =
        withContext(Dispatchers.IO) {
            conn().use { c ->
                fetchPetById(c, petId) ?: throw NotFoundException("Pet not found: $petId")
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
            ModelApiResponse(code = 200, type = "unknown", message = "File uploaded: ${bytes.size} bytes")
        }

    private fun fetchPetById(petId: Long): Pet? = conn().use { fetchPetById(it, petId) }

    /** Uses an existing connection so composite operations never hold one pool slot while acquiring another. */
    private fun fetchPetById(c: Connection, petId: Long): Pet? {
        c.prepareStatement(
            "SELECT id, name, photo_urls, category, tags, status::text FROM pet WHERE id = ?"
        ).use { ps ->
            ps.setLong(1, petId)
            return ps.executeQuery().use { rs -> if (rs.next()) rs.toPet() else null }
        }
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

    override suspend fun getInventory(): Map<String, Int> = withContext(Dispatchers.IO) {
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
                result
            }
        }
    }

    override suspend fun placeOrder(order: Order): Order = withContext(Dispatchers.IO) {
        conn().use { c ->
            val id = order.id ?: nextId(c, "\"order\"")
            val saved = order.copy(id = id)
            upsertOrder(c, saved)
            fetchOrderById(c, id)!!
        }
    }

    override suspend fun getOrderById(orderId: Long): Order? = withContext(Dispatchers.IO) {
        fetchOrderById(orderId)
    }

    override suspend fun deleteOrder(orderId: Long): Unit = withContext(Dispatchers.IO) {
        conn().use { c ->
            fetchOrderById(c, orderId) ?: throw NotFoundException("Order not found: $orderId")
            c.prepareStatement("DELETE FROM \"order\" WHERE id = ?").use { ps ->
                ps.setLong(1, orderId)
                ps.executeUpdate()
            }
        }
    }

    private fun fetchOrderById(orderId: Long): Order? = conn().use { fetchOrderById(it, orderId) }

    private fun fetchOrderById(c: Connection, orderId: Long): Order? {
        c.prepareStatement(
            "SELECT id, pet_id, quantity, ship_date, status::text, complete FROM \"order\" WHERE id = ?"
        ).use { ps ->
            ps.setLong(1, orderId)
            return ps.executeQuery().use { rs -> if (rs.next()) rs.toOrder() else null }
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

    override suspend fun createUser(user: User): User = withContext(Dispatchers.IO) {
        conn().use { c ->
            val id = user.id ?: nextId(c, "\"user\"")
            upsertUser(c, user.copy(id = id))
            fetchUserByName(c, user.username ?: "")!!
        }
    }

    override suspend fun createUsersWithList(users: List<User>): User = withContext(Dispatchers.IO) {
        conn().use { c ->
            for (user in users) {
                val id = user.id ?: nextId(c, "\"user\"")
                upsertUser(c, user.copy(id = id))
            }
            fetchUserByName(c, users.first().username ?: "")!!
        }
    }

    override suspend fun loginUser(username: String, password: String): String = withContext(Dispatchers.IO) {
        fetchUserByName(username) ?: throw NotFoundException("User not found: $username")
        "logged-in-$username"
    }

    override suspend fun logoutUser() { /* stateless no-op */ }

    override suspend fun getUserByName(username: String): User? = withContext(Dispatchers.IO) {
        fetchUserByName(username)
    }

    override suspend fun updateUser(username: String, user: User): Unit = withContext(Dispatchers.IO) {
        conn().use { c ->
            fetchUserByName(c, username) ?: throw NotFoundException("User not found: $username")
            upsertUser(c, user.copy(username = username))
        }
    }

    override suspend fun deleteUser(username: String): Unit = withContext(Dispatchers.IO) {
        conn().use { c ->
            fetchUserByName(c, username) ?: throw NotFoundException("User not found: $username")
            c.prepareStatement("DELETE FROM \"user\" WHERE username = ?").use { ps ->
                ps.setString(1, username)
                ps.executeUpdate()
            }
        }
    }

    private fun fetchUserByName(username: String): User? = conn().use { fetchUserByName(it, username) }

    private fun fetchUserByName(c: Connection, username: String): User? {
        c.prepareStatement(
            "SELECT id, username, first_name, last_name, email, password, phone, user_status FROM \"user\" WHERE username = ?"
        ).use { ps ->
            ps.setString(1, username)
            return ps.executeQuery().use { rs -> if (rs.next()) rs.toUser() else null }
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

    private fun nextId(c: Connection, table: String): Long {
        val seqName = "${table.replace("\"", "")}_id_seq"
        c.prepareStatement("SELECT nextval('$seqName')").use { ps ->
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
        maximumPoolSize = System.getenv("HIKARI_MAXIMUM_POOL_SIZE")?.toInt() ?: 10
        System.getenv("HIKARI_MINIMUM_IDLE")?.let { minimumIdle = it.toInt() }
    })
}
