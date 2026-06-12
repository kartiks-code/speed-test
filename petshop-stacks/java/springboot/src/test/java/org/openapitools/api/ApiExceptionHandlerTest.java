package org.openapitools.api;

import org.junit.jupiter.api.Test;
import org.openapitools.model.Error;
import org.openapitools.persistence.InvalidInputException;
import org.openapitools.persistence.NotFoundException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Unit tests for {@link ApiExceptionHandler}. Pure unit tests: the handler is
 * exercised directly with exception instances, no Spring context or DB required.
 */
class ApiExceptionHandlerTest {

    private final ApiExceptionHandler handler = new ApiExceptionHandler();

    @Test
    void notFoundMapsTo404WithNotFoundCode() {
        ResponseEntity<Error> response = handler.handleNotFound(new NotFoundException("pet 7 not found"));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.NOT_FOUND);
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().getCode()).isEqualTo("not_found");
        assertThat(response.getBody().getMessage()).isEqualTo("pet 7 not found");
    }

    @Test
    void invalidInputMapsTo400WithInvalidInputCode() {
        ResponseEntity<Error> response = handler.handleInvalidInput(new InvalidInputException("username is required"));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.BAD_REQUEST);
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().getCode()).isEqualTo("invalid_input");
        assertThat(response.getBody().getMessage()).isEqualTo("username is required");
    }

    @Test
    void genericExceptionMapsTo500WithErrorCode() {
        ResponseEntity<Error> response = handler.handleGeneric(new RuntimeException("boom"));

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.INTERNAL_SERVER_ERROR);
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().getCode()).isEqualTo("error");
        assertThat(response.getBody().getMessage()).isEqualTo("boom");
    }
}
