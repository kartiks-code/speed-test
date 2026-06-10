package org.openapitools.server.api;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.validation.Valid;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import org.openapitools.server.db.OrderRepository;
import org.openapitools.server.db.PetRepository;
import org.openapitools.server.model.Order;

import java.util.Map;

@Path("/store")
@ApplicationScoped
@jakarta.annotation.Generated(value = "org.openapitools.codegen.languages.JavaHelidonServerCodegen", comments = "Generator version: 7.23.0")
public class StoreServiceImpl implements StoreService {

    @Inject
    OrderRepository orderRepo;

    @Inject
    PetRepository petRepo;

    @DELETE
    @Path("/order/{orderId}")
    @Produces({ "application/json" })
    public void deleteOrder(@PathParam("orderId") Long orderId) {
        orderRepo.delete(orderId);
    }

    @GET
    @Path("/inventory")
    @Produces({ "application/json" })
    public Map<String, Integer> getInventory() {
        return petRepo.getInventory();
    }

    @GET
    @Path("/order/{orderId}")
    @Produces({ "application/json", "application/xml" })
    public Order getOrderById(@PathParam("orderId") Long orderId) {
        return orderRepo.findById(orderId);
    }

    @POST
    @Path("/order")
    @Consumes({ "application/json", "application/xml", "application/x-www-form-urlencoded" })
    @Produces({ "application/json" })
    public Order placeOrder(@Valid Order order) {
        return orderRepo.place(order);
    }
}
