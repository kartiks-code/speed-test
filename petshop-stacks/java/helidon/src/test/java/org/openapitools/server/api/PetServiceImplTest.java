package org.openapitools.server.api;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.util.List;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;
import org.openapitools.server.db.PetRepository;
import org.openapitools.server.model.ModelApiResponse;
import org.openapitools.server.model.Pet;

/**
 * Unit tests for {@link PetServiceImpl}. The JAX-RS resource is a thin delegate, so these
 * tests verify that each endpoint forwards to the injected {@link PetRepository} and that
 * repository-thrown {@link WebApplicationException}s propagate untouched.
 */
class PetServiceImplTest {

    private PetRepository repo;
    private PetServiceImpl service;

    @BeforeEach
    void setUp() {
        repo = Mockito.mock(PetRepository.class);
        service = new PetServiceImpl();
        service.repo = repo;
    }

    @Test
    void addPetDelegatesToRepository() {
        Pet pet = new Pet().name("Fido");
        Pet stored = new Pet().id(1L).name("Fido");
        when(repo.add(pet)).thenReturn(stored);

        assertSame(stored, service.addPet(pet));
        verify(repo).add(pet);
    }

    @Test
    void deletePetDelegatesToRepository() {
        Response response = service.deletePet(7L, "api-key");
        assertNotNull(response);
        assertEquals(200, response.getStatus());
        verify(repo).delete(7L);
    }

    @Test
    void findPetsByStatusDelegatesToRepository() {
        List<Pet> expected = List.of(new Pet().id(1L));
        when(repo.findByStatus("available")).thenReturn(expected);

        assertSame(expected, service.findPetsByStatus("available"));
        verify(repo).findByStatus("available");
    }

    @Test
    void findPetsByTagsDelegatesToRepository() {
        List<String> tags = List.of("cute", "fluffy");
        List<Pet> expected = List.of(new Pet().id(2L));
        when(repo.findByTags(tags)).thenReturn(expected);

        assertSame(expected, service.findPetsByTags(tags));
        verify(repo).findByTags(tags);
    }

    @Test
    void getPetByIdReturnsRepositoryResult() {
        Pet pet = new Pet().id(5L).name("Rex");
        when(repo.findById(5L)).thenReturn(pet);

        assertSame(pet, service.getPetById(5L));
        verify(repo).findById(5L);
    }

    @Test
    void getPetByIdPropagatesNotFound() {
        when(repo.findById(404L))
            .thenThrow(new WebApplicationException("Pet not found", Response.Status.NOT_FOUND));

        WebApplicationException ex = assertThrows(WebApplicationException.class, () -> service.getPetById(404L));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void updatePetDelegatesToRepository() {
        Pet pet = new Pet().id(3L).name("Milo");
        when(repo.update(pet)).thenReturn(pet);

        assertSame(pet, service.updatePet(pet));
        verify(repo).update(pet);
    }

    @Test
    void updatePetPropagatesBadRequest() {
        Pet pet = new Pet().name("no-id");
        when(repo.update(pet))
            .thenThrow(new WebApplicationException("Pet ID is required for update", Response.Status.BAD_REQUEST));

        WebApplicationException ex = assertThrows(WebApplicationException.class, () -> service.updatePet(pet));
        assertEquals(Response.Status.BAD_REQUEST.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void updatePetWithFormDelegatesToRepository() {
        service.updatePetWithForm(9L, "Buddy", "sold");
        verify(repo).updateWithForm(9L, "Buddy", "sold");
    }

    @Test
    void uploadFilePersistsPhotoAndReportsByteCount() {
        when(repo.savePhoto(1L, new byte[0], "application/octet-stream", "meta-data")).thenReturn(0);

        ModelApiResponse response = service.uploadFile(1L, "meta-data", null);

        assertEquals(200, response.getCode());
        assertEquals("application/octet-stream", response.getType());
        assertEquals("petId: 1, bytes: 0, additionalMetadata: meta-data", response.getMessage());
        verify(repo).savePhoto(1L, new byte[0], "application/octet-stream", "meta-data");
    }
}
