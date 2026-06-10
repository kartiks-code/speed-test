package org.openapitools.api;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.openapitools.model.ModelApiResponse;
import org.openapitools.model.Pet;
import org.openapitools.persistence.PetStore;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.context.request.NativeWebRequest;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Unit tests for {@link PetApiController}. The controller is exercised in
 * isolation with a Mockito-mocked {@link PetStore}; no Spring context or DB.
 */
@ExtendWith(MockitoExtension.class)
class PetApiControllerTest {

    @Mock
    private NativeWebRequest request;

    @Mock
    private PetStore petStore;

    @InjectMocks
    private PetApiController controller;

    private static Pet samplePet() {
        return new Pet().id(1L).name("Fido").photoUrls(List.of("http://example.com/fido.jpg"))
                .status(Pet.StatusEnum.AVAILABLE);
    }

    @Test
    void addPetDelegatesAndReturnsOk() {
        Pet pet = samplePet();
        when(petStore.createPet(pet)).thenReturn(pet);

        ResponseEntity<Pet> response = controller.addPet(pet);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(pet);
        verify(petStore).createPet(pet);
    }

    @Test
    void updatePetDelegatesAndReturnsOk() {
        Pet pet = samplePet();
        when(petStore.updatePet(pet)).thenReturn(pet);

        ResponseEntity<Pet> response = controller.updatePet(pet);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(pet);
        verify(petStore).updatePet(pet);
    }

    @Test
    void getPetByIdDelegatesAndReturnsOk() {
        Pet pet = samplePet();
        when(petStore.getPetById(1L)).thenReturn(pet);

        ResponseEntity<Pet> response = controller.getPetById(1L);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(pet);
        verify(petStore).getPetById(1L);
    }

    @Test
    void deletePetDelegatesAndReturnsOkWithNoBody() {
        ResponseEntity<Void> response = controller.deletePet(1L, "secret-key");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isNull();
        verify(petStore).deletePet(1L);
    }

    @Test
    void findPetsByStatusSplitsCommaSeparatedStatuses() {
        List<Pet> pets = List.of(samplePet());
        when(petStore.findPetsByStatus(List.of("available", "pending"))).thenReturn(pets);

        ResponseEntity<List<Pet>> response = controller.findPetsByStatus("available,pending");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isEqualTo(pets);
        verify(petStore).findPetsByStatus(List.of("available", "pending"));
    }

    @Test
    void findPetsByStatusWithNullPassesEmptyList() {
        when(petStore.findPetsByStatus(List.of())).thenReturn(List.of());

        ResponseEntity<List<Pet>> response = controller.findPetsByStatus(null);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        verify(petStore).findPetsByStatus(List.of());
    }

    @Test
    void findPetsByTagsPassesTagsThrough() {
        List<Pet> pets = List.of(samplePet());
        when(petStore.findPetsByTags(List.of("cute"))).thenReturn(pets);

        ResponseEntity<List<Pet>> response = controller.findPetsByTags(List.of("cute"));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isEqualTo(pets);
        verify(petStore).findPetsByTags(List.of("cute"));
    }

    @Test
    void findPetsByTagsWithNullPassesEmptyList() {
        when(petStore.findPetsByTags(List.of())).thenReturn(List.of());

        controller.findPetsByTags(null);

        verify(petStore).findPetsByTags(List.of());
    }

    @Test
    void updatePetWithFormDelegatesFields() {
        ResponseEntity<Void> response = controller.updatePetWithForm(1L, "Rex", "sold");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        verify(petStore).updatePetFields(1L, "Rex", "sold");
    }

    @Test
    void uploadFilePersistsBytesAndReportsByteCountAndMetadata() throws Exception {
        byte[] data = "hello".getBytes();
        when(petStore.savePetPhoto(eq(1L), any(byte[].class), eq("application/octet-stream"), eq("note")))
                .thenReturn(data.length);

        ResponseEntity<ModelApiResponse> response = controller.uploadFile(
                1L, "note", new ByteArrayResource(data));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        ModelApiResponse body = response.getBody();
        assertThat(body).isNotNull();
        assertThat(body.getCode()).isEqualTo(200);
        assertThat(body.getType()).isEqualTo("application/octet-stream");
        assertThat(body.getMessage()).isEqualTo("petId: 1, bytes: 5, additionalMetadata: note");
        verify(petStore).savePetPhoto(eq(1L), any(byte[].class), eq("application/octet-stream"), eq("note"));
    }

    @Test
    void uploadFilePersistsEmptyContentWhenNoBody() {
        when(petStore.savePetPhoto(eq(2L), any(byte[].class), eq("application/octet-stream"), eq("  ")))
                .thenReturn(0);

        ResponseEntity<ModelApiResponse> response = controller.uploadFile(2L, "  ", null);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().getMessage()).isEqualTo("petId: 2, bytes: 0");
    }

    @Test
    void getRequestExposesInjectedRequest() {
        assertThat(controller.getRequest()).contains(request);
    }
}
