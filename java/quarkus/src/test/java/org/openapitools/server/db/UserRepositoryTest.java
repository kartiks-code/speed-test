package org.openapitools.server.db;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.Types;

import javax.sql.DataSource;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.openapitools.server.model.User;

/**
 * Unit tests for {@link UserRepository} with the JDBC chain mocked. Covers parameter binding,
 * null handling, authentication result interpretation, and ResultSet → {@link User} mapping.
 */
class UserRepositoryTest {

    private DataSource dataSource;
    private Connection connection;
    private PreparedStatement statement;
    private ResultSet resultSet;
    private UserRepository repository;

    @BeforeEach
    void setUp() throws Exception {
        dataSource = mock(DataSource.class);
        connection = mock(Connection.class);
        statement = mock(PreparedStatement.class);
        resultSet = mock(ResultSet.class);

        when(dataSource.getConnection()).thenReturn(connection);
        when(connection.prepareStatement(anyString())).thenReturn(statement);
        when(statement.executeQuery()).thenReturn(resultSet);

        repository = new UserRepository();
        repository.dataSource = dataSource;
    }

    private String capturedSql() throws Exception {
        ArgumentCaptor<String> sql = ArgumentCaptor.forClass(String.class);
        verify(connection).prepareStatement(sql.capture());
        return sql.getValue();
    }

    @Test
    void createBindsAllColumns() throws Exception {
        User user = new User()
            .id(10L)
            .username("alice")
            .firstName("Alice")
            .lastName("Smith")
            .email("alice@example.com")
            .password("pw")
            .phone("555")
            .userStatus(1);

        User result = repository.create(user);

        assertSame(user, result);
        assertTrue(capturedSql().contains("INSERT INTO \"user\""));
        verify(statement).setLong(1, 10L);
        verify(statement).setString(2, "alice");
        verify(statement).setString(3, "Alice");
        verify(statement).setString(4, "Smith");
        verify(statement).setString(5, "alice@example.com");
        verify(statement).setString(6, "pw");
        verify(statement).setString(7, "555");
        verify(statement).setInt(8, 1);
        verify(statement).executeUpdate();
    }

    @Test
    void createSetsNullsForMissingIdAndStatus() throws Exception {
        User user = new User().username("bob");

        repository.create(user);

        verify(statement).setNull(1, Types.BIGINT);
        verify(statement).setNull(8, Types.INTEGER);
    }

    @Test
    void findByUsernameMapsRow() throws Exception {
        when(resultSet.next()).thenReturn(true);
        when(resultSet.getObject("id", Long.class)).thenReturn(10L);
        when(resultSet.getString("username")).thenReturn("alice");
        when(resultSet.getString("first_name")).thenReturn("Alice");
        when(resultSet.getString("last_name")).thenReturn("Smith");
        when(resultSet.getString("email")).thenReturn("alice@example.com");
        when(resultSet.getString("password")).thenReturn("pw");
        when(resultSet.getString("phone")).thenReturn("555");
        when(resultSet.getObject("user_status", Integer.class)).thenReturn(2);

        User user = repository.findByUsername("alice");

        assertEquals(10L, user.getId());
        assertEquals("alice", user.getUsername());
        assertEquals("Alice", user.getFirstName());
        assertEquals("Smith", user.getLastName());
        assertEquals("alice@example.com", user.getEmail());
        assertEquals("pw", user.getPassword());
        assertEquals("555", user.getPhone());
        assertEquals(2, user.getUserStatus());
        verify(statement).setString(1, "alice");
    }

    @Test
    void findByUsernameThrowsNotFoundWhenNoRow() throws Exception {
        when(resultSet.next()).thenReturn(false);

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findByUsername("ghost"));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void updateBindsColumnsAndUsernamePredicate() throws Exception {
        User user = new User().id(5L).firstName("New").userStatus(3);

        repository.update("alice", user);

        String sql = capturedSql();
        assertTrue(sql.contains("UPDATE \"user\" SET"));
        assertTrue(sql.contains("WHERE username = ?"));
        verify(statement).setLong(1, 5L);
        verify(statement).setString(2, "New");
        verify(statement).setInt(7, 3);
        verify(statement).setString(8, "alice");
        verify(statement).executeUpdate();
    }

    @Test
    void authenticateReturnsTrueWhenRowPresent() throws Exception {
        when(resultSet.next()).thenReturn(true);

        assertTrue(repository.authenticate("alice", "pw"));
        verify(statement).setString(1, "alice");
        verify(statement).setString(2, "pw");
    }

    @Test
    void authenticateReturnsFalseWhenNoRow() throws Exception {
        when(resultSet.next()).thenReturn(false);

        assertFalse(repository.authenticate("alice", "wrong"));
    }

    @Test
    void deleteBindsUsernameAndExecutes() throws Exception {
        repository.delete("alice");
        assertTrue(capturedSql().contains("DELETE FROM \"user\""));
        verify(statement).setString(1, "alice");
        verify(statement).executeUpdate();
    }

    @Test
    void wrapsSqlFailuresAsInternalServerError() throws Exception {
        when(dataSource.getConnection()).thenThrow(new java.sql.SQLException("boom"));

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findByUsername("x"));
        assertEquals(Response.Status.INTERNAL_SERVER_ERROR.getStatusCode(), ex.getResponse().getStatus());
    }
}
