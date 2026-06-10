package org.openapitools.persistence;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.openapitools.model.Category;
import org.openapitools.model.Order;
import org.openapitools.model.Pet;
import org.openapitools.model.Tag;
import org.openapitools.model.User;
import org.postgresql.util.PGobject;
import org.springframework.dao.EmptyResultDataAccessException;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.core.RowMapper;
import org.springframework.stereotype.Repository;
import org.springframework.transaction.annotation.Transactional;

import java.sql.SQLException;
import java.sql.Timestamp;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

@Repository
public class PetStore {

    private final JdbcTemplate jdbc;
    private final ObjectMapper objectMapper;

    public PetStore(JdbcTemplate jdbc, ObjectMapper objectMapper) {
        this.jdbc = jdbc;
        this.objectMapper = objectMapper;
    }

    // -----------------------------------------------------------------------
    // Pet
    // -----------------------------------------------------------------------

    public Pet createPet(Pet pet) {
        validatePet(pet);
        if (pet.getId() == null || pet.getId() == 0) {
            pet.setId(nextId("pet"));
        }
        return upsertPet(pet);
    }

    public Pet updatePet(Pet pet) {
        if (pet.getId() == null || pet.getId() == 0) {
            throw new InvalidInputException("pet id is required for update");
        }
        validatePet(pet);
        getPetById(pet.getId()); // throws NotFoundException if missing
        return upsertPet(pet);
    }

    public Pet getPetById(long id) {
        try {
            return jdbc.queryForObject(
                    "SELECT id, name, category, photo_urls, tags, status::text FROM pet WHERE id = ?",
                    PET_MAPPER, id);
        } catch (EmptyResultDataAccessException e) {
            throw new NotFoundException("pet " + id + " not found");
        }
    }

    public boolean deletePet(long id) {
        return jdbc.update("DELETE FROM pet WHERE id = ?", id) > 0;
    }

    public List<Pet> findPetsByStatus(List<String> statuses) {
        List<String> cleaned = compactStrings(statuses);
        if (cleaned.isEmpty()) {
            cleaned = List.of("available");
        }
        String sql = "SELECT id, name, category, photo_urls, tags, status::text FROM pet"
                + " WHERE status::text IN (" + placeholders(cleaned.size()) + ") ORDER BY id";
        return jdbc.query(sql, PET_MAPPER, cleaned.toArray());
    }

    public List<Pet> findPetsByTags(List<String> tags) {
        List<String> cleaned = compactStrings(tags);
        if (cleaned.isEmpty()) {
            return List.of();
        }
        String sql = "SELECT id, name, category, photo_urls, tags, status::text FROM pet"
                + " WHERE EXISTS (SELECT 1 FROM json_array_elements(COALESCE(tags, '[]'::json)) elem"
                + " WHERE elem->>'name' IN (" + placeholders(cleaned.size()) + ")) ORDER BY id";
        return jdbc.query(sql, PET_MAPPER, cleaned.toArray());
    }

    public boolean updatePetFields(long id, String name, String status) {
        if (name == null && status == null) {
            getPetById(id); // verify exists
            return true;
        }
        List<String> setClauses = new ArrayList<>();
        List<Object> args = new ArrayList<>();
        if (name != null) {
            setClauses.add("name = ?");
            args.add(name);
        }
        if (status != null) {
            if (!validPetStatus(status)) {
                throw new InvalidInputException("invalid pet status: " + status);
            }
            setClauses.add("status = ?");
            args.add(pgEnum("pet_status", status));
        }
        args.add(id);
        String sql = "UPDATE pet SET " + String.join(", ", setClauses) + " WHERE id = ?";
        return jdbc.update(sql, args.toArray()) > 0;
    }

    public int savePetPhoto(long petId, byte[] content, String contentType, String metadata) {
        getPetById(petId); // throws NotFoundException if missing
        byte[] data = content != null ? content : new byte[0];
        jdbc.update(
                "INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)"
                        + " VALUES ((SELECT COALESCE(MAX(id), 0) + 1 FROM pet_photo), ?, ?, ?, ?)",
                petId, contentType, metadata, data);
        return data.length;
    }

    public Map<String, Integer> inventory() {
        List<Map<String, Object>> rows = jdbc.queryForList(
                "SELECT status::text, COUNT(*) AS cnt FROM pet WHERE status IS NOT NULL GROUP BY status ORDER BY status");
        Map<String, Integer> result = new HashMap<>();
        for (Map<String, Object> row : rows) {
            result.put((String) row.get("status"), ((Number) row.get("cnt")).intValue());
        }
        return result;
    }

    // -----------------------------------------------------------------------
    // Order
    // -----------------------------------------------------------------------

    public Order placeOrder(Order order) {
        if (order.getId() == null || order.getId() == 0) {
            order.setId(nextId("\"order\""));
        }
        return upsertOrder(order);
    }

    public Order getOrderById(long id) {
        try {
            return jdbc.queryForObject(
                    "SELECT id, pet_id, quantity, ship_date, status::text, complete FROM \"order\" WHERE id = ?",
                    ORDER_MAPPER, id);
        } catch (EmptyResultDataAccessException e) {
            throw new NotFoundException("order " + id + " not found");
        }
    }

    public boolean deleteOrder(long id) {
        return jdbc.update("DELETE FROM \"order\" WHERE id = ?", id) > 0;
    }

    // -----------------------------------------------------------------------
    // User
    // -----------------------------------------------------------------------

    public User createUser(User user) {
        if (user.getUsername() == null || user.getUsername().isBlank()) {
            throw new InvalidInputException("username is required");
        }
        if (user.getId() == null || user.getId() == 0) {
            user.setId(nextId("\"user\""));
        }
        return upsertUser(user);
    }

    @Transactional
    public User createUsers(List<User> users) {
        User last = null;
        for (User user : users) {
            if (user.getUsername() == null || user.getUsername().isBlank()) {
                throw new InvalidInputException("username is required");
            }
            if (user.getId() == null || user.getId() == 0) {
                user.setId(nextId("\"user\""));
            }
            last = upsertUser(user);
        }
        return last;
    }

    public User getUserByUsername(String username) {
        try {
            return jdbc.queryForObject(
                    "SELECT id, username, first_name, last_name, email, password, phone, user_status"
                            + " FROM \"user\" WHERE username = ?",
                    USER_MAPPER, username);
        } catch (EmptyResultDataAccessException e) {
            throw new NotFoundException("user " + username + " not found");
        }
    }

    public boolean updateUser(String username, User user) {
        if (username == null || username.isBlank()) {
            throw new InvalidInputException("username is required");
        }
        User existing = getUserByUsername(username); // throws NotFoundException
        if (user.getUsername() == null || user.getUsername().isBlank()) {
            user.setUsername(username);
        }
        if (user.getId() == null || user.getId() == 0) {
            user.setId(existing.getId());
        }
        int rows = jdbc.update(
                "UPDATE \"user\" SET id = ?, username = ?, first_name = ?, last_name = ?,"
                        + " email = ?, password = ?, phone = ?, user_status = ? WHERE username = ?",
                user.getId(), user.getUsername(), user.getFirstName(), user.getLastName(),
                user.getEmail(), user.getPassword(), user.getPhone(), user.getUserStatus(),
                username);
        return rows > 0;
    }

    public boolean deleteUser(String username) {
        return jdbc.update("DELETE FROM \"user\" WHERE username = ?", username) > 0;
    }

    public boolean authenticateUser(String username, String password) {
        Boolean result = jdbc.queryForObject(
                "SELECT EXISTS (SELECT 1 FROM \"user\" WHERE username = ? AND password = ?)",
                Boolean.class, username, password);
        return Boolean.TRUE.equals(result);
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    private Pet upsertPet(Pet pet) {
        String statusValue = pet.getStatus() != null ? pet.getStatus().getValue() : null;
        jdbc.update(
                "INSERT INTO pet (id, name, category, photo_urls, tags, status)"
                        + " VALUES (?, ?, ?, ?, ?, ?)"
                        + " ON CONFLICT (id) DO UPDATE"
                        + " SET name = EXCLUDED.name,"
                        + "     category = EXCLUDED.category,"
                        + "     photo_urls = EXCLUDED.photo_urls,"
                        + "     tags = EXCLUDED.tags,"
                        + "     status = EXCLUDED.status",
                pet.getId(),
                pet.getName(),
                pgJson(pet.getCategory()),
                pgJson(pet.getPhotoUrls()),
                pgJson(pet.getTags()),
                pgEnum("pet_status", statusValue));
        return getPetById(pet.getId());
    }

    private Order upsertOrder(Order order) {
        String statusValue = order.getStatus() != null ? order.getStatus().getValue() : null;
        Timestamp shipDate = order.getShipDate() != null
                ? Timestamp.from(order.getShipDate().toInstant()) : null;
        jdbc.update(
                "INSERT INTO \"order\" (id, pet_id, quantity, ship_date, status, complete)"
                        + " VALUES (?, ?, ?, ?, ?, ?)"
                        + " ON CONFLICT (id) DO UPDATE"
                        + " SET pet_id = EXCLUDED.pet_id,"
                        + "     quantity = EXCLUDED.quantity,"
                        + "     ship_date = EXCLUDED.ship_date,"
                        + "     status = EXCLUDED.status,"
                        + "     complete = EXCLUDED.complete",
                order.getId(),
                order.getPetId(),
                order.getQuantity(),
                shipDate,
                pgEnum("order_status", statusValue),
                order.getComplete());
        return getOrderById(order.getId());
    }

    private User upsertUser(User user) {
        jdbc.update(
                "INSERT INTO \"user\" (id, username, first_name, last_name, email, password, phone, user_status)"
                        + " VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                        + " ON CONFLICT (username) DO UPDATE"
                        + " SET id = EXCLUDED.id,"
                        + "     first_name = EXCLUDED.first_name,"
                        + "     last_name = EXCLUDED.last_name,"
                        + "     email = EXCLUDED.email,"
                        + "     password = EXCLUDED.password,"
                        + "     phone = EXCLUDED.phone,"
                        + "     user_status = EXCLUDED.user_status",
                user.getId(), user.getUsername(), user.getFirstName(), user.getLastName(),
                user.getEmail(), user.getPassword(), user.getPhone(), user.getUserStatus());
        return getUserByUsername(user.getUsername());
    }

    private long nextId(String table) {
        Long id = jdbc.queryForObject("SELECT COALESCE(MAX(id), 0) + 1 FROM " + table, Long.class);
        return id != null ? id : 1L;
    }

    private void validatePet(Pet pet) {
        if (pet.getName() == null || pet.getName().isBlank()) {
            throw new InvalidInputException("pet name is required");
        }
        if (pet.getPhotoUrls() == null || pet.getPhotoUrls().isEmpty()) {
            throw new InvalidInputException("pet photoUrls is required");
        }
        if (pet.getStatus() != null && !validPetStatus(pet.getStatus().getValue())) {
            throw new InvalidInputException("invalid pet status: " + pet.getStatus().getValue());
        }
    }

    private static boolean validPetStatus(String status) {
        return "available".equals(status) || "pending".equals(status) || "sold".equals(status);
    }

    private static String placeholders(int count) {
        return IntStream.range(0, count).mapToObj(i -> "?").collect(Collectors.joining(", "));
    }

    private static List<String> compactStrings(List<String> values) {
        if (values == null) return List.of();
        List<String> result = new ArrayList<>();
        for (String value : values) {
            if (value == null) continue;
            for (String part : value.split(",")) {
                String trimmed = part.trim();
                if (!trimmed.isEmpty()) {
                    result.add(trimmed);
                }
            }
        }
        return result;
    }

    private PGobject pgJson(Object value) {
        try {
            PGobject obj = new PGobject();
            obj.setType("json");
            obj.setValue(value == null ? null : objectMapper.writeValueAsString(value));
            return obj;
        } catch (JsonProcessingException | SQLException e) {
            throw new RuntimeException("JSON serialization failed", e);
        }
    }

    private static PGobject pgEnum(String type, String value) {
        try {
            PGobject obj = new PGobject();
            obj.setType(type);
            obj.setValue(value == null || value.isBlank() ? null : value);
            return obj;
        } catch (SQLException e) {
            throw new RuntimeException("enum serialization failed", e);
        }
    }

    // -----------------------------------------------------------------------
    // RowMappers
    // -----------------------------------------------------------------------

    private final RowMapper<Pet> PET_MAPPER = (rs, rowNum) -> {
        Pet pet = new Pet();
        pet.setId(rs.getLong("id"));
        pet.setName(rs.getString("name"));

        String categoryJson = rs.getString("category");
        if (categoryJson != null && !categoryJson.isBlank()) {
            pet.setCategory(fromJson(categoryJson, Category.class));
        }

        String photoUrlsJson = rs.getString("photo_urls");
        pet.setPhotoUrls(photoUrlsJson != null
                ? fromJsonList(photoUrlsJson, String.class) : new ArrayList<>());

        String tagsJson = rs.getString("tags");
        if (tagsJson != null && !tagsJson.isBlank()) {
            pet.setTags(fromJsonList(tagsJson, Tag.class));
        }

        String statusStr = rs.getString("status");
        if (statusStr != null) {
            pet.setStatus(Pet.StatusEnum.fromValue(statusStr));
        }
        return pet;
    };

    private final RowMapper<Order> ORDER_MAPPER = (rs, rowNum) -> {
        Order order = new Order();
        order.setId(rs.getLong("id"));

        long petId = rs.getLong("pet_id");
        if (!rs.wasNull()) order.setPetId(petId);

        int quantity = rs.getInt("quantity");
        if (!rs.wasNull()) order.setQuantity(quantity);

        Timestamp shipDate = rs.getTimestamp("ship_date");
        if (shipDate != null) {
            order.setShipDate(shipDate.toInstant().atOffset(ZoneOffset.UTC));
        }

        String statusStr = rs.getString("status");
        if (statusStr != null) {
            order.setStatus(Order.StatusEnum.fromValue(statusStr));
        }

        boolean complete = rs.getBoolean("complete");
        if (!rs.wasNull()) order.setComplete(complete);

        return order;
    };

    private final RowMapper<User> USER_MAPPER = (rs, rowNum) -> {
        User user = new User();

        long id = rs.getLong("id");
        if (!rs.wasNull()) user.setId(id);

        user.setUsername(rs.getString("username"));
        user.setFirstName(rs.getString("first_name"));
        user.setLastName(rs.getString("last_name"));
        user.setEmail(rs.getString("email"));
        user.setPassword(rs.getString("password"));
        user.setPhone(rs.getString("phone"));

        int userStatus = rs.getInt("user_status");
        if (!rs.wasNull()) user.setUserStatus(userStatus);

        return user;
    };

    private <T> T fromJson(String json, Class<T> type) {
        if (json == null || json.isBlank() || "null".equals(json)) return null;
        try {
            return objectMapper.readValue(json, type);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("JSON deserialization failed", e);
        }
    }

    private <T> List<T> fromJsonList(String json, Class<T> elementType) {
        if (json == null || json.isBlank() || "null".equals(json)) return new ArrayList<>();
        try {
            return objectMapper.readValue(json,
                    objectMapper.getTypeFactory().constructCollectionType(List.class, elementType));
        } catch (JsonProcessingException e) {
            throw new RuntimeException("JSON deserialization failed", e);
        }
    }
}
