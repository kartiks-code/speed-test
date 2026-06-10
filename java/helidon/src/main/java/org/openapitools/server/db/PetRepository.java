package org.openapitools.server.db;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.openapitools.server.model.Category;
import org.openapitools.server.model.Pet;
import org.openapitools.server.model.Tag;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Types;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicLong;

@ApplicationScoped
public class PetRepository {

    @Inject
    DataSourceProvider dsProvider;

    private static final AtomicLong idGen = new AtomicLong(System.currentTimeMillis());
    private final ObjectMapper mapper = new ObjectMapper();

    public Pet add(Pet pet) {
        try (Connection conn = dsProvider.get().getConnection()) {
            long id = pet.getId() != null ? pet.getId() : idGen.getAndIncrement();
            String sql =
                "INSERT INTO pet (\"id\", \"name\", category, photo_urls, tags, status) " +
                "VALUES (?, ?, ?, cast(? as json), cast(? as json), cast(? as pet_status))";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setLong(1, id);
                ps.setString(2, pet.getName());
                ps.setString(3, pet.getCategory() != null ? mapper.writeValueAsString(pet.getCategory()) : null);
                ps.setString(4, mapper.writeValueAsString(pet.getPhotoUrls()));
                ps.setString(5, pet.getTags() != null ? mapper.writeValueAsString(pet.getTags()) : null);
                ps.setString(6, pet.getStatus() != null ? pet.getStatus().value() : null);
                ps.executeUpdate();
            }
            pet.setId(id);
            return pet;
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to add pet: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public Pet update(Pet pet) {
        if (pet.getId() == null) {
            throw new WebApplicationException("Pet ID is required for update", Response.Status.BAD_REQUEST);
        }
        try (Connection conn = dsProvider.get().getConnection()) {
            String sql =
                "UPDATE pet SET \"name\" = ?, category = ?, " +
                "photo_urls = cast(? as json), tags = cast(? as json), status = cast(? as pet_status) " +
                "WHERE \"id\" = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setString(1, pet.getName());
                ps.setString(2, pet.getCategory() != null ? mapper.writeValueAsString(pet.getCategory()) : null);
                ps.setString(3, mapper.writeValueAsString(pet.getPhotoUrls()));
                ps.setString(4, pet.getTags() != null ? mapper.writeValueAsString(pet.getTags()) : null);
                ps.setString(5, pet.getStatus() != null ? pet.getStatus().value() : null);
                ps.setLong(6, pet.getId());
                if (ps.executeUpdate() == 0) {
                    throw new WebApplicationException("Pet not found", Response.Status.NOT_FOUND);
                }
            }
            return pet;
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to update pet: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public Pet findById(long petId) {
        try (Connection conn = dsProvider.get().getConnection()) {
            String sql =
                "SELECT \"id\", \"name\", category, photo_urls, tags, status::text " +
                "FROM pet WHERE \"id\" = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setLong(1, petId);
                try (ResultSet rs = ps.executeQuery()) {
                    if (rs.next()) {
                        return mapRow(rs);
                    }
                }
            }
            throw new WebApplicationException("Pet not found", Response.Status.NOT_FOUND);
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to get pet: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public void delete(long petId) {
        try (Connection conn = dsProvider.get().getConnection()) {
            try (PreparedStatement ps = conn.prepareStatement("DELETE FROM pet WHERE \"id\" = ?")) {
                ps.setLong(1, petId);
                ps.executeUpdate();
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to delete pet: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public List<Pet> findByStatus(String status) {
        try (Connection conn = dsProvider.get().getConnection()) {
            String sql =
                "SELECT \"id\", \"name\", category, photo_urls, tags, status::text " +
                "FROM pet WHERE status = cast(? as pet_status)";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setString(1, status);
                try (ResultSet rs = ps.executeQuery()) {
                    List<Pet> pets = new ArrayList<>();
                    while (rs.next()) {
                        pets.add(mapRow(rs));
                    }
                    return pets;
                }
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to find pets by status: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public List<Pet> findByTags(List<String> tags) {
        if (tags == null || tags.isEmpty()) {
            return new ArrayList<>();
        }
        try (Connection conn = dsProvider.get().getConnection()) {
            StringBuilder sql = new StringBuilder(
                "SELECT \"id\", \"name\", category, photo_urls, tags, status::text FROM pet WHERE false");
            for (int i = 0; i < tags.size(); i++) {
                sql.append(" OR tags::jsonb @> cast(? as jsonb)");
            }
            try (PreparedStatement ps = conn.prepareStatement(sql.toString())) {
                for (int i = 0; i < tags.size(); i++) {
                    ObjectNode node = mapper.createObjectNode().put("name", tags.get(i));
                    ArrayNode arr = mapper.createArrayNode().add(node);
                    ps.setString(i + 1, mapper.writeValueAsString(arr));
                }
                try (ResultSet rs = ps.executeQuery()) {
                    List<Pet> pets = new ArrayList<>();
                    while (rs.next()) {
                        pets.add(mapRow(rs));
                    }
                    return pets;
                }
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to find pets by tags: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public void updateWithForm(long petId, String name, String status) {
        if (name == null && status == null) {
            return;
        }
        try (Connection conn = dsProvider.get().getConnection()) {
            StringBuilder sql = new StringBuilder("UPDATE pet SET ");
            List<Object> params = new ArrayList<>();
            if (name != null) {
                sql.append("\"name\" = ?");
                params.add(name);
            }
            if (status != null) {
                if (!params.isEmpty()) sql.append(", ");
                sql.append("status = cast(? as pet_status)");
                params.add(status);
            }
            sql.append(" WHERE \"id\" = ?");
            params.add(petId);
            try (PreparedStatement ps = conn.prepareStatement(sql.toString())) {
                for (int i = 0; i < params.size(); i++) {
                    ps.setObject(i + 1, params.get(i));
                }
                ps.executeUpdate();
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to update pet with form: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public Map<String, Integer> getInventory() {
        try (Connection conn = dsProvider.get().getConnection()) {
            String sql = "SELECT status::text, cast(COUNT(*) as int) FROM pet GROUP BY status";
            try (PreparedStatement ps = conn.prepareStatement(sql);
                 ResultSet rs = ps.executeQuery()) {
                Map<String, Integer> inventory = new HashMap<>();
                while (rs.next()) {
                    String s = rs.getString(1);
                    if (s != null) {
                        inventory.put(s, rs.getInt(2));
                    }
                }
                return inventory;
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to get inventory: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    private Pet mapRow(ResultSet rs) throws Exception {
        Pet pet = new Pet();
        pet.setId(rs.getObject("id", Long.class));
        pet.setName(rs.getString("name"));

        String categoryJson = rs.getString("category");
        if (categoryJson != null) {
            pet.setCategory(mapper.readValue(categoryJson, Category.class));
        }

        String photoUrlsJson = rs.getString("photo_urls");
        if (photoUrlsJson != null) {
            pet.setPhotoUrls(mapper.readValue(photoUrlsJson, new TypeReference<List<String>>() {}));
        }

        String tagsJson = rs.getString("tags");
        if (tagsJson != null) {
            pet.setTags(mapper.readValue(tagsJson, new TypeReference<List<Tag>>() {}));
        }

        String statusStr = rs.getString("status");
        if (statusStr != null) {
            pet.setStatus(Pet.StatusEnum.fromValue(statusStr));
        }

        return pet;
    }
}
