package org.openapitools.server.db;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.openapitools.server.model.Order;

import javax.sql.DataSource;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.Timestamp;
import java.sql.Types;
import java.time.ZoneOffset;
import java.util.concurrent.atomic.AtomicLong;

@ApplicationScoped
public class OrderRepository {

    @Inject
    DataSource dataSource;

    private static final AtomicLong idGen = new AtomicLong(System.currentTimeMillis());

    public Order place(Order order) {
        try (Connection conn = dataSource.getConnection()) {
            long id = order.getId() != null ? order.getId() : idGen.getAndIncrement();
            String sql =
                "INSERT INTO \"order\" (\"id\", pet_id, quantity, ship_date, status, complete) " +
                "VALUES (?, ?, ?, ?, cast(? as order_status), ?)";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setLong(1, id);
                if (order.getPetId() != null) ps.setLong(2, order.getPetId());
                else ps.setNull(2, Types.BIGINT);
                if (order.getQuantity() != null) ps.setInt(3, order.getQuantity());
                else ps.setNull(3, Types.INTEGER);
                if (order.getShipDate() != null)
                    ps.setTimestamp(4, Timestamp.from(order.getShipDate().toInstant()));
                else ps.setNull(4, Types.TIMESTAMP);
                ps.setString(5, order.getStatus() != null ? order.getStatus().value() : null);
                if (order.getComplete() != null) ps.setBoolean(6, order.getComplete());
                else ps.setNull(6, Types.BOOLEAN);
                ps.executeUpdate();
            }
            order.setId(id);
            return order;
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to place order: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public Order findById(long orderId) {
        try (Connection conn = dataSource.getConnection()) {
            String sql =
                "SELECT \"id\", pet_id, quantity, ship_date, status::text, complete " +
                "FROM \"order\" WHERE \"id\" = ?";
            try (PreparedStatement ps = conn.prepareStatement(sql)) {
                ps.setLong(1, orderId);
                try (ResultSet rs = ps.executeQuery()) {
                    if (rs.next()) {
                        return mapRow(rs);
                    }
                }
            }
            throw new WebApplicationException("Order not found", Response.Status.NOT_FOUND);
        } catch (WebApplicationException e) {
            throw e;
        } catch (Exception e) {
            throw new WebApplicationException("Failed to get order: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    public void delete(long orderId) {
        try (Connection conn = dataSource.getConnection()) {
            try (PreparedStatement ps = conn.prepareStatement("DELETE FROM \"order\" WHERE \"id\" = ?")) {
                ps.setLong(1, orderId);
                ps.executeUpdate();
            }
        } catch (Exception e) {
            throw new WebApplicationException("Failed to delete order: " + e.getMessage(),
                Response.Status.INTERNAL_SERVER_ERROR);
        }
    }

    private Order mapRow(ResultSet rs) throws Exception {
        Order order = new Order();
        order.setId(rs.getObject("id", Long.class));
        order.setPetId(rs.getObject("pet_id", Long.class));
        order.setQuantity(rs.getObject("quantity", Integer.class));
        Timestamp ts = rs.getTimestamp("ship_date");
        if (ts != null) {
            order.setShipDate(ts.toInstant().atOffset(ZoneOffset.UTC));
        }
        String statusStr = rs.getString("status");
        if (statusStr != null) {
            order.setStatus(Order.StatusEnum.fromValue(statusStr));
        }
        order.setComplete(rs.getObject("complete", Boolean.class));
        return order;
    }
}
