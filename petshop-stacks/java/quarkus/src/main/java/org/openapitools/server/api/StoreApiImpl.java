package org.openapitools.server.api;

import io.smallrye.common.annotation.RunOnVirtualThread;
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
@RunOnVirtualThread
public class StoreApiImpl implements StoreApi {

    @Inject
    OrderRepository orderRepo;

    @Inject
    PetRepository petRepo;

    @DELETE
    @Path("/order/{orderId}")
    @Produces({ "application/json" })
    public jakarta.ws.rs.core.Response deleteOrder(@PathParam("orderId") Long orderId) {
        orderRepo.delete(orderId);
        return jakarta.ws.rs.core.Response.ok().build();
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
