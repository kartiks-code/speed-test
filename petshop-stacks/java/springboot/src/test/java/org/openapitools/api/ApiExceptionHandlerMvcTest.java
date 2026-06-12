package org.openapitools.api;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.openapitools.model.Pet;
import org.openapitools.persistence.InvalidInputException;
import org.openapitools.persistence.NotFoundException;
import org.openapitools.persistence.PetStore;
import org.springframework.http.MediaType;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.request.NativeWebRequest;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.when;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * Verifies that {@link ApiExceptionHandler} maps persistence exceptions to the
 * correct HTTP status and JSON error body when wired into the MVC pipeline.
 *
 * Uses a standalone MockMvc setup (no Spring context, no DB) registering the
 * real {@link PetApiController} with the exception handler as advice.
 */
@ExtendWith(MockitoExtension.class)
class ApiExceptionHandlerMvcTest {

    @Mock
    private NativeWebRequest request;

    @Mock
    private PetStore petStore;

    private MockMvc mockMvc;

    @BeforeEach
    void setUp() {
        mockMvc = MockMvcBuilders.standaloneSetup(new PetApiController(request, petStore))
                .setControllerAdvice(new ApiExceptionHandler())
                .addPlaceholderValue("openapi.swaggerPetstoreOpenAPI31.base-path", "/api/v3")
                .build();
    }

    @Test
    void notFoundExceptionBecomes404() throws Exception {
        when(petStore.getPetById(anyLong())).thenThrow(new NotFoundException("pet 7 not found"));

        mockMvc.perform(get("/api/v3/pet/7"))
                .andExpect(status().isNotFound())
                .andExpect(jsonPath("$.code").value("not_found"))
                .andExpect(jsonPath("$.message").value("pet 7 not found"));
    }

    @Test
    void invalidInputExceptionBecomes400() throws Exception {
        when(petStore.createPet(any(Pet.class)))
                .thenThrow(new InvalidInputException("pet name is required"));

        mockMvc.perform(post("/api/v3/pet")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"name\":\"\",\"photoUrls\":[\"http://x/1.jpg\"]}"))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.code").value("invalid_input"))
                .andExpect(jsonPath("$.message").value("pet name is required"));
    }
}
