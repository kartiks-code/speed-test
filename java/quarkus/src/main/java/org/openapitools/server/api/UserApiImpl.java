package org.openapitools.server.api;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.validation.Valid;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.openapitools.server.db.UserRepository;
import org.openapitools.server.model.User;

import java.util.List;

@Path("/user")
@ApplicationScoped
public class UserApiImpl implements UserApi {

    @Inject
    UserRepository repo;

    @POST
    @Consumes({ "application/json", "application/xml", "application/x-www-form-urlencoded" })
    @Produces({ "application/json", "application/xml" })
    public User createUser(@Valid User user) {
        return repo.create(user);
    }

    @POST
    @Path("/createWithList")
    @Consumes({ "application/json" })
    @Produces({ "application/json", "application/xml" })
    public User createUsersWithListInput(@Valid List<@Valid User> user) {
        if (user == null || user.isEmpty()) {
            throw new WebApplicationException("User list must not be empty", Response.Status.BAD_REQUEST);
        }
        for (User u : user) {
            repo.create(u);
        }
        return user.get(0);
    }

    @DELETE
    @Path("/{username}")
    @Produces({ "application/json" })
    public void deleteUser(@PathParam("username") String username) {
        repo.delete(username);
    }

    @GET
    @Path("/{username}")
    @Produces({ "application/json", "application/xml" })
    public User getUserByName(@PathParam("username") String username) {
        return repo.findByUsername(username);
    }

    @GET
    @Path("/login")
    @Produces({ "application/xml", "application/json" })
    public String loginUser(@QueryParam("username") String username,
                            @QueryParam("password") String password) {
        if (!repo.authenticate(username, password)) {
            throw new WebApplicationException("Invalid username or password", Response.Status.BAD_REQUEST);
        }
        return "logged in user session: " + username + "/" + System.currentTimeMillis();
    }

    @GET
    @Path("/logout")
    public void logoutUser() {
        // stateless — nothing to invalidate
    }

    @PUT
    @Path("/{username}")
    @Consumes({ "application/json", "application/xml", "application/x-www-form-urlencoded" })
    @Produces({ "application/json" })
    public void updateUser(@PathParam("username") String username, @Valid User user) {
        repo.update(username, user);
    }
}
