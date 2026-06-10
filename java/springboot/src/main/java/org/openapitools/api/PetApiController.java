package org.openapitools.api;

import org.openapitools.model.ModelApiResponse;
import org.openapitools.model.Pet;
import org.openapitools.persistence.PetStore;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.context.request.NativeWebRequest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import org.springframework.lang.Nullable;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;

@Controller
@RequestMapping("${openapi.swaggerPetstoreOpenAPI31.base-path:/api/v3}")
public class PetApiController implements PetApi {

    private final NativeWebRequest request;
    private final PetStore petStore;

    public PetApiController(NativeWebRequest request, PetStore petStore) {
        this.request = request;
        this.petStore = petStore;
    }

    @Override
    public Optional<NativeWebRequest> getRequest() {
        return Optional.ofNullable(request);
    }

    @Override
    public ResponseEntity<Pet> addPet(@Valid Pet pet) {
        return ResponseEntity.ok(petStore.createPet(pet));
    }

    @Override
    public ResponseEntity<Pet> updatePet(@Valid Pet pet) {
        return ResponseEntity.ok(petStore.updatePet(pet));
    }

    @Override
    public ResponseEntity<Pet> getPetById(Long petId) {
        return ResponseEntity.ok(petStore.getPetById(petId));
    }

    @Override
    public ResponseEntity<Void> deletePet(Long petId, @Nullable String apiKey) {
        petStore.deletePet(petId);
        return ResponseEntity.ok().build();
    }

    @Override
    public ResponseEntity<List<Pet>> findPetsByStatus(@Valid String status) {
        List<String> statuses = status != null ? Arrays.asList(status.split(",")) : List.of();
        return ResponseEntity.ok(petStore.findPetsByStatus(statuses));
    }

    @Override
    public ResponseEntity<List<Pet>> findPetsByTags(@Valid @Nullable List<String> tags) {
        return ResponseEntity.ok(petStore.findPetsByTags(tags != null ? tags : List.of()));
    }

    @Override
    public ResponseEntity<Void> updatePetWithForm(Long petId, @Nullable String name, @Nullable String status) {
        petStore.updatePetFields(petId, name, status);
        return ResponseEntity.ok().build();
    }

    @Override
    public ResponseEntity<ModelApiResponse> uploadFile(
            Long petId,
            @Nullable String additionalMetadata,
            @Nullable org.springframework.core.io.Resource body) {
        int size = 0;
        if (body != null) {
            try {
                size = (int) body.contentLength();
            } catch (Exception ignored) {
                // size remains 0
            }
        }
        String msg = buildUploadMessage(petId, additionalMetadata, size);
        ModelApiResponse response = new ModelApiResponse()
                .code(200)
                .type("unknown")
                .message(msg);
        return ResponseEntity.ok(response);
    }

    private static String buildUploadMessage(long petId, String additionalMetadata, int size) {
        StringBuilder sb = new StringBuilder();
        sb.append("petId: ").append(petId);
        sb.append(", bytes: ").append(size);
        if (additionalMetadata != null && !additionalMetadata.isBlank()) {
            sb.append(", additionalMetadata: ").append(additionalMetadata);
        }
        return sb.toString();
    }
}
