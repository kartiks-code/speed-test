package org.openapitools.server.db;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.openapitools.server.model.User;

import javax.sql.DataSource;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.Types;

@ApplicationScoped
public class UserRepository {

    @Inject
    DataSource dataSource;

    public User create(User user) {
        try (Connection conn = dataSource.getConnection()) {
            String sql =
                "INSERT INTO \"user\" (\"id\", username, first_name, last_name, email, \"password\", phone, user_status) " +
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                if (user.getId() != null) ps.setLong(1, user.getId());
                else ps.setNull(1, Types.BIGINT);
                ps.setString(2, user.getUsername());
                ps.setString(3, user.getFirstName());
                ps.setString(4, user.getLastName());
                ps.setString(5, user.getEmail());
                ps.setString(6, user.getPassword());
                ps.setString(7, user.getPhone());
                if (user.getUserStatus() != null) ps.setInt(8, user.getUserStatus());
                else ps.setNull(8, Types.INTEGER);
                ps.executeUpdate();
            }
            return user;
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to create user: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public User findByUsername(String username) {
        try (Connection conn = dataSource.getConnection()) {
            String sql =
                "SELECT \"id\", username, first_name, last_name, email, \"password\", phone, user_status " +
                "FROM \"user\" WHERE username = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setString(1, username);
                try (ResultSet rs = ps.executeQuery()) {
                    if (rs.next()) {
                        return mapRow(rs);
                    }
                }
            }
            throw new WebApplicationException("User not found", Response.Status.NOT_FOUND);
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to get user: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public void update(String username, User user) {
        try (Connection conn = dataSource.getConnection()) {
            String sql =
                "UPDATE \"user\" SET \"id\" = ?, first_name = ?, last_name = ?, email = ?, " +
                "\"password\" = ?, phone = ?, user_status = ? WHERE username = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                if (user.getId() != null) ps.setLong(1, user.getId());
                else ps.setNull(1, Types.BIGINT);
                ps.setString(2, user.getFirstName());
                ps.setString(3, user.getLastName());
                ps.setString(4, user.getEmail());
                ps.setString(5, user.getPassword());
                ps.setString(6, user.getPhone());
                if (user.getUserStatus() != null) ps.setInt(7, user.getUserStatus());
                else ps.setNull(7, Types.INTEGER);
                ps.setString(8, username);
                ps.executeUpdate();
            }
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to update user: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public void delete(String username) {
        try (Connection conn = dataSource.getConnection()) {
            try (PreparedStatement ps = conn.prepareStatement("DELETE FROM \"user\" WHERE username = ?")) {
                ps.setString(1, username);
                ps.executeUpdate();
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to delete user: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public boolean authenticate(String username, String password) {
        try (Connection conn = dataSource.getConnection()) {
            String sql = "SELECT 1 FROM \"user\" WHERE username = ? AND \"password\" = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setString(1, username);
                ps.setString(2, password);
                try (ResultSet rs = ps.executeQuery()) {
                    return rs.next();
                }
            }
        } catch (Exception e) {
            throw new WebApplicationException("Authentication failed: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    private User mapRow(ResultSet rs) throws Exception {
        User user = new User();
        user.setId(rs.getObject("id", Long.class));
        user.setUsername(rs.getString("username"));
        user.setFirstName(rs.getString("first_name"));
        user.setLastName(rs.getString("last_name"));
        user.setEmail(rs.getString("email"));
        user.setPassword(rs.getString("password"));
        user.setPhone(rs.getString("phone"));
        user.setUserStatus(rs.getObject("user_status", Integer.class));
        return user;
    }
}
