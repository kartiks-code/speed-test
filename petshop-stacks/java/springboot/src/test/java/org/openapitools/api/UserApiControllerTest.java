package org.openapitools.api;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.openapitools.model.User;
import org.openapitools.persistence.InvalidInputException;
import org.openapitools.persistence.PetStore;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.context.request.NativeWebRequest;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

/**
 * Unit tests for {@link UserApiController} with a Mockito-mocked {@link PetStore}.
 * Covers the success path plus the invalid-input branches the controller enforces
 * before delegating to persistence.
 */
@ExtendWith(MockitoExtension.class)
class UserApiControllerTest {

    @Mock
    private NativeWebRequest request;

    @Mock
    private PetStore petStore;

    @InjectMocks
    private UserApiController controller;

    private static User sampleUser() {
        return new User().id(1L).username("alice").password("secret");
    }

    @Test
    void createUserDelegatesAndReturnsOk() {
        User user = sampleUser();
        when(petStore.createUser(user)).thenReturn(user);

        ResponseEntity<User> response = controller.createUser(user);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(user);
        verify(petStore).createUser(user);
    }

    @Test
    void createUserWithNullBodyThrowsInvalidInput() {
        assertThatThrownBy(() -> controller.createUser(null))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("user body is required");
        verifyNoInteractions(petStore);
    }

    @Test
    void createUsersWithListDelegatesAndReturnsOk() {
        List<User> users = List.of(sampleUser());
        User last = sampleUser();
        when(petStore.createUsers(users)).thenReturn(last);

        ResponseEntity<User> response = controller.createUsersWithListInput(users);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(last);
        verify(petStore).createUsers(users);
    }

    @Test
    void createUsersWithNullListThrowsInvalidInput() {
        assertThatThrownBy(() -> controller.createUsersWithListInput(null))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("user list is required");
        verifyNoInteractions(petStore);
    }

    @Test
    void createUsersWithEmptyListThrowsInvalidInput() {
        assertThatThrownBy(() -> controller.createUsersWithListInput(List.of()))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("user list is required");
        verifyNoInteractions(petStore);
    }

    @Test
    void getUserByNameDelegatesAndReturnsOk() {
        User user = sampleUser();
        when(petStore.getUserByUsername("alice")).thenReturn(user);

        ResponseEntity<User> response = controller.getUserByName("alice");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isSameAs(user);
        verify(petStore).getUserByUsername("alice");
    }

    @Test
    void loginUserReturnsSessionWithRateLimitHeadersOnSuccess() {
        when(petStore.authenticateUser("alice", "secret")).thenReturn(true);

        ResponseEntity<String> response = controller.loginUser("alice", "secret");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isEqualTo("logged in user session: alice");
        assertThat(response.getHeaders().getFirst("X-Rate-Limit")).isEqualTo("5000");
        assertThat(response.getHeaders().getFirst("X-Expires-After")).isNotBlank();
        verify(petStore).authenticateUser("alice", "secret");
    }

    @Test
    void loginUserWithBlankCredentialsThrowsInvalidInput() {
        assertThatThrownBy(() -> controller.loginUser("", "secret"))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("username and password are required");
        verifyNoInteractions(petStore);
    }

    @Test
    void loginUserWithWrongCredentialsThrowsInvalidInput() {
        when(petStore.authenticateUser("alice", "nope")).thenReturn(false);

        assertThatThrownBy(() -> controller.loginUser("alice", "nope"))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("invalid username or password");
    }

    @Test
    void logoutUserIsStatelessNoOp() {
        ResponseEntity<Void> response = controller.logoutUser();

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        assertThat(response.getBody()).isNull();
        verifyNoInteractions(petStore);
    }

    @Test
    void updateUserDelegatesAndReturnsOk() {
        User user = sampleUser();

        ResponseEntity<Void> response = controller.updateUser("alice", user);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        verify(petStore).updateUser("alice", user);
    }

    @Test
    void updateUserWithNullBodyThrowsInvalidInput() {
        assertThatThrownBy(() -> controller.updateUser("alice", null))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("user body is required");
        verifyNoInteractions(petStore);
    }

    @Test
    void deleteUserDelegatesAndReturnsOk() {
        ResponseEntity<Void> response = controller.deleteUser("alice");

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.OK);
        verify(petStore).deleteUser("alice");
    }
}
