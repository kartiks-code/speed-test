package org.openapitools.server.db;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.Timestamp;
import java.sql.Types;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;

import javax.sql.DataSource;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.openapitools.server.model.Order;

/**
 * Unit tests for {@link OrderRepository} with the JDBC chain mocked. Covers parameter binding
 * (including null handling and the {@code order_status} enum cast), timestamp conversion, and
 * ResultSet → {@link Order} mapping.
 */
class OrderRepositoryTest {

    private DataSource dataSource;
    private Connection connection;
    private PreparedStatement statement;
    private ResultSet resultSet;
    private OrderRepository repository;

    @BeforeEach
    void setUp() throws Exception {
        dataSource = mock(DataSource.class);
        connection = mock(Connection.class);
        statement = mock(PreparedStatement.class);
        resultSet = mock(ResultSet.class);

        when(dataSource.getConnection()).thenReturn(connection);
        when(connection.prepareStatement(anyString())).thenReturn(statement);
        when(statement.executeQuery()).thenReturn(resultSet);

        repository = new OrderRepository();
        repository.dataSource = dataSource;
    }

    private String capturedSql() throws Exception {
        ArgumentCaptor<String> sql = ArgumentCaptor.forClass(String.class);
        verify(connection).prepareStatement(sql.capture());
        return sql.getValue();
    }

    @Test
    void placeBindsAllFieldsAndCastsStatus() throws Exception {
        OffsetDateTime shipDate = OffsetDateTime.of(2024, 1, 2, 3, 4, 5, 0, ZoneOffset.UTC);
        Order order = new Order()
            .id(30L)
            .petId(7L)
            .quantity(2)
            .shipDate(shipDate)
            .status(Order.StatusEnum.PLACED)
            .complete(true);

        Order result = repository.place(order);

        assertSame(order, result);
        assertEquals(30L, result.getId());

        String sql = capturedSql();
        assertTrue(sql.contains("INSERT INTO \"order\""));
        assertTrue(sql.contains("cast(? as order_status)"));

        verify(statement).setLong(1, 30L);
        verify(statement).setLong(2, 7L);
        verify(statement).setInt(3, 2);
        verify(statement).setTimestamp(4, Timestamp.from(shipDate.toInstant()));
        verify(statement).setString(5, "placed");
        verify(statement).setBoolean(6, true);
        verify(statement).executeUpdate();
    }

    @Test
    void placeGeneratesIdAndSetsNullsForMissingOptionalFields() throws Exception {
        Order order = new Order();

        Order result = repository.place(order);

        assertNotNull(result.getId());
        verify(statement).setNull(2, Types.BIGINT);
        verify(statement).setNull(3, Types.INTEGER);
        verify(statement).setNull(4, Types.TIMESTAMP);
        verify(statement).setString(eq(5), eq((String) null));
        verify(statement).setNull(6, Types.BOOLEAN);
    }

    @Test
    void findByIdMapsRow() throws Exception {
        Timestamp ts = Timestamp.from(OffsetDateTime.of(2023, 6, 1, 0, 0, 0, 0, ZoneOffset.UTC).toInstant());
        when(resultSet.next()).thenReturn(true);
        when(resultSet.getObject("id", Long.class)).thenReturn(4L);
        when(resultSet.getObject("pet_id", Long.class)).thenReturn(7L);
        when(resultSet.getObject("quantity", Integer.class)).thenReturn(3);
        when(resultSet.getTimestamp("ship_date")).thenReturn(ts);
        when(resultSet.getString("status")).thenReturn("approved");
        when(resultSet.getObject("complete", Boolean.class)).thenReturn(true);

        Order order = repository.findById(4L);

        assertEquals(4L, order.getId());
        assertEquals(7L, order.getPetId());
        assertEquals(3, order.getQuantity());
        assertNotNull(order.getShipDate());
        assertEquals(Order.StatusEnum.APPROVED, order.getStatus());
        assertEquals(true, order.getComplete());
        assertTrue(capturedSql().contains("status::text"));
        verify(statement).setLong(1, 4L);
    }

    @Test
    void findByIdHandlesNullShipDateAndStatus() throws Exception {
        when(resultSet.next()).thenReturn(true);
        when(resultSet.getObject("id", Long.class)).thenReturn(1L);
        when(resultSet.getTimestamp("ship_date")).thenReturn(null);
        when(resultSet.getString("status")).thenReturn(null);

        Order order = repository.findById(1L);

        assertNull(order.getShipDate());
        assertNull(order.getStatus());
    }

    @Test
    void findByIdThrowsNotFoundWhenNoRow() throws Exception {
        when(resultSet.next()).thenReturn(false);

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findById(99L));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void deleteBindsIdAndExecutes() throws Exception {
        repository.delete(12L);
        assertTrue(capturedSql().contains("DELETE FROM \"order\""));
        verify(statement).setLong(1, 12L);
        verify(statement).executeUpdate();
    }

    @Test
    void wrapsSqlFailuresAsInternalServerError() throws Exception {
        when(dataSource.getConnection()).thenThrow(new java.sql.SQLException("boom"));

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findById(1L));
        assertEquals(Response.Status.INTERNAL_SERVER_ERROR.getStatusCode(), ex.getResponse().getStatus());
    }
}
