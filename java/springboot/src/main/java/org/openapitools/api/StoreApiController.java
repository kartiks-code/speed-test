package org.openapitools.api;

import org.openapitools.model.Order;
import org.openapitools.persistence.PetStore;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.context.request.NativeWebRequest;

import jakarta.validation.Valid;
import org.springframework.lang.Nullable;
import java.util.Map;
import java.util.Optional;

@Controller
@RequestMapping("${openapi.swaggerPetstoreOpenAPI31.base-path:/api/v3}")
public class StoreApiController implements StoreApi {

    private final NativeWebRequest request;
    private final PetStore petStore;

    public StoreApiController(NativeWebRequest request, PetStore petStore) {
        this.request = request;
        this.petStore = petStore;
    }

    @Override
    public Optional<NativeWebRequest> getRequest() {
        return Optional.ofNullable(request);
    }

    @Override
    public ResponseEntity<Map<String, Integer>> getInventory() {
        return ResponseEntity.ok(petStore.inventory());
    }

    @Override
    public ResponseEntity<Order> placeOrder(@Valid @Nullable Order order) {
        if (order == null) {
            order = new Order();
        }
        return ResponseEntity.ok(petStore.placeOrder(order));
    }

    @Override
    public ResponseEntity<Order> getOrderById(Long orderId) {
        return ResponseEntity.ok(petStore.getOrderById(orderId));
    }

    @Override
    public ResponseEntity<Void> deleteOrder(Long orderId) {
        petStore.deleteOrder(orderId);
        return ResponseEntity.ok().build();
    }
}
