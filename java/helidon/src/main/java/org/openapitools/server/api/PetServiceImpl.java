package org.openapitools.server.api;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.DefaultValue;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.HeaderParam;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import org.openapitools.server.db.PetRepository;
import org.openapitools.server.model.ModelApiResponse;
import org.openapitools.server.model.Pet;

import java.io.File;
import java.util.List;

@Path("/pet")
@ApplicationScoped
@jakarta.annotation.Generated(value = "org.openapitools.codegen.languages.JavaHelidonServerCodegen", comments = "Generator version: 7.23.0")
public class PetServiceImpl implements PetService {

    @Inject
    PetRepository repo;

    @POST
    @Consumes({ "application/json", "application/xml", "application/x-www-form-urlencoded" })
    @Produces({ "application/json", "application/xml" })
    public Pet addPet(@Valid @NotNull Pet pet) {
        return repo.add(pet);
    }

    @DELETE
    @Path("/{petId}")
    @Produces({ "application/json" })
    public void deletePet(@PathParam("petId") Long petId, @HeaderParam("api_key") String apiKey) {
        repo.delete(petId);
    }

    @GET
    @Path("/findByStatus")
    @Produces({ "application/json", "application/xml" })
    public List<Pet> findPetsByStatus(@QueryParam("status") @DefaultValue("available") String status) {
        return repo.findByStatus(status);
    }

    @GET
    @Path("/findByTags")
    @Produces({ "application/json", "application/xml" })
    public List<Pet> findPetsByTags(@QueryParam("tags") List<String> tags) {
        return repo.findByTags(tags);
    }

    @GET
    @Path("/{petId}")
    @Produces({ "application/json", "application/xml" })
    public Pet getPetById(@PathParam("petId") Long petId) {
        return repo.findById(petId);
    }

    @PUT
    @Consumes({ "application/json", "application/xml", "application/x-www-form-urlencoded" })
    @Produces({ "application/json", "application/xml" })
    public Pet updatePet(@Valid @NotNull Pet pet) {
        return repo.update(pet);
    }

    @POST
    @Path("/{petId}")
    @Produces({ "application/json" })
    public void updatePetWithForm(@PathParam("petId") Long petId,
                                  @QueryParam("name") String name,
                                  @QueryParam("status") String status) {
        repo.updateWithForm(petId, name, status);
    }

    @POST
    @Path("/{petId}/uploadImage")
    @Consumes({ "application/octet-stream" })
    @Produces({ "application/json" })
    public ModelApiResponse uploadFile(@PathParam("petId") Long petId,
                                       @QueryParam("additionalMetadata") String additionalMetadata,
                                       @Valid File body) {
        byte[] content = new byte[0];
        if (body != null) {
            try {
                content = java.nio.file.Files.readAllBytes(body.toPath());
            } catch (java.io.IOException e) {
                throw new jakarta.ws.rs.WebApplicationException(
                    "Failed to read uploaded file: " + e.getMessage(),
                    jakarta.ws.rs.core.Response.Status.BAD_REQUEST);
            }
        }
        int size = repo.savePhoto(petId, content, "application/octet-stream", additionalMetadata);
        ModelApiResponse response = new ModelApiResponse();
        response.setCode(200);
        response.setType("application/octet-stream");
        response.setMessage("petId: " + petId + ", bytes: " + size
            + (additionalMetadata != null ? ", additionalMetadata: " + additionalMetadata : ""));
        return response;
    }
}
