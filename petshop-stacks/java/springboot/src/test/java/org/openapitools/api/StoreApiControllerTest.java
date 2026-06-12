package org.openapitools.api;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.openapitools.model.Order;
import org.openapitools.persistence.PetStore;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.context.request.NativeWebRequest;

import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Unit tests for {@link StoreApiController} with a Mockito-mocked {@link PetStore}.
 */
@ExtendWith(MockitoExtension.class)
class StoreApiControllerTest {

    @Mock
    private NativeWebRequest request;

    @Mock
    private PetStore petStore;

    @InjectMocks
    private StoreApiController controller;

    @Test
    void getInventoryDelegatesAndReturnsOk() {
        Map<String, Integer> inventory = Map.of("available", 3, "sold", 1);
        when(petStore.inventory()).thenReturn(inventory);

        ResponseEntity<Map<String, Integer>> response = controller.getInventory();

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isEqualTo(inventory);
        verify(petStore).inventory();
    }

    @Test
    void placeOrderDelegatesAndReturnsOk() {
        Order order = new Order().id(10L).petId(1L).quantity(2);
        when(petStore.placeOrder(order)).thenReturn(order);

        ResponseEntity<Order> response = controller.placeOrder(order);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(order);
        verify(petStore).placeOrder(order);
    }

    @Test
    void placeOrderWithNullBodyUsesEmptyOrder() {
        ArgumentCaptor<Order> captor = ArgumentCaptor.forClass(Order.class);
        when(petStore.placeOrder(captor.capture())).thenAnswer(inv -> inv.getArgument(0));

        ResponseEntity<Order> response = controller.placeOrder(null);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(captor.getValue()).isNotNull();
        assertThat(captor.getValue().getId()).isNull();
    }

    @Test
    void getOrderByIdDelegatesAndReturnsOk() {
        Order order = new Order().id(10L);
        when(petStore.getOrderById(10L)).thenReturn(order);

        ResponseEntity<Order> response = controller.getOrderById(10L);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(order);
        verify(petStore).getOrderById(10L);
    }

    @Test
    void deleteOrderDelegatesAndReturnsOkWithNoBody() {
        ResponseEntity<Void> response = controller.deleteOrder(10L);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isNull();
        verify(petStore).deleteOrder(10L);
    }
}
