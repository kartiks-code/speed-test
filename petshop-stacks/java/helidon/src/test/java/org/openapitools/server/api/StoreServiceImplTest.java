package org.openapitools.server.api;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

import java.util.Map;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;
import org.openapitools.server.db.OrderRepository;
import org.openapitools.server.db.PetRepository;
import org.openapitools.server.model.Order;

/**
 * Unit tests for {@link StoreServiceImpl}, verifying delegation to the order and pet
 * repositories and propagation of repository errors.
 */
class StoreServiceImplTest {

    private OrderRepository orderRepo;
    private PetRepository petRepo;
    private StoreServiceImpl service;

    @BeforeEach
    void setUp() {
        orderRepo = Mockito.mock(OrderRepository.class);
        petRepo = Mockito.mock(PetRepository.class);
        service = new StoreServiceImpl();
        service.orderRepo = orderRepo;
        service.petRepo = petRepo;
    }

    @Test
    void deleteOrderDelegatesToOrderRepository() {
        Response response = service.deleteOrder(11L);
        assertNotNull(response);
        assertEquals(200, response.getStatus());
        verify(orderRepo).delete(11L);
        verifyNoInteractions(petRepo);
    }

    @Test
    void getInventoryDelegatesToPetRepository() {
        Map<String, Integer> inventory = Map.of("available", 3, "sold", 1);
        when(petRepo.getInventory()).thenReturn(inventory);

        assertSame(inventory, service.getInventory());
        verify(petRepo).getInventory();
        verifyNoInteractions(orderRepo);
    }

    @Test
    void getOrderByIdReturnsRepositoryResult() {
        Order order = new Order().id(4L).quantity(2);
        when(orderRepo.findById(4L)).thenReturn(order);

        assertSame(order, service.getOrderById(4L));
        verify(orderRepo).findById(4L);
    }

    @Test
    void getOrderByIdPropagatesNotFound() {
        when(orderRepo.findById(99L))
            .thenThrow(new WebApplicationException("Order not found", Response.Status.NOT_FOUND));

        WebApplicationException ex = assertThrows(WebApplicationException.class, () -> service.getOrderById(99L));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void placeOrderDelegatesToOrderRepository() {
        Order order = new Order().petId(7L).quantity(1);
        Order stored = new Order().id(20L).petId(7L).quantity(1);
        when(orderRepo.place(order)).thenReturn(stored);

        assertSame(stored, service.placeOrder(order));
        verify(orderRepo).place(order);
    }
}
