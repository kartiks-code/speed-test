package org.openapitools.server.api;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

import java.util.List;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;
import org.openapitools.server.db.UserRepository;
import org.openapitools.server.model.User;

/**
 * Unit tests for {@link UserServiceImpl}, covering delegation plus the bits of logic that
 * live in the resource itself: list validation, login authentication, and the no-op logout.
 */
class UserServiceImplTest {

    private UserRepository repo;
    private UserServiceImpl service;

    @BeforeEach
    void setUp() {
        repo = Mockito.mock(UserRepository.class);
        service = new UserServiceImpl();
        service.repo = repo;
    }

    @Test
    void createUserDelegatesToRepository() {
        User user = new User().username("alice");
        when(repo.create(user)).thenReturn(user);

        assertSame(user, service.createUser(user));
        verify(repo).create(user);
    }

    @Test
    void createUsersWithListInputCreatesEachAndReturnsFirst() {
        User first = new User().username("alice");
        User second = new User().username("bob");

        User result = service.createUsersWithListInput(List.of(first, second));

        assertSame(first, result);
        verify(repo).create(first);
        verify(repo).create(second);
    }

    @Test
    void createUsersWithListInputRejectsNullList() {
        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> service.createUsersWithListInput(null));
        assertEquals(Response.Status.BAD_REQUEST.getStatusCode(), ex.getResponse().getStatus());
        verify(repo, never()).create(Mockito.any());
    }

    @Test
    void createUsersWithListInputRejectsEmptyList() {
        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> service.createUsersWithListInput(List.of()));
        assertEquals(Response.Status.BAD_REQUEST.getStatusCode(), ex.getResponse().getStatus());
        verify(repo, never()).create(Mockito.any());
    }

    @Test
    void deleteUserDelegatesToRepository() {
        service.deleteUser("carol");
        verify(repo).delete("carol");
    }

    @Test
    void getUserByNameReturnsRepositoryResult() {
        User user = new User().username("dave");
        when(repo.findByUsername("dave")).thenReturn(user);

        assertSame(user, service.getUserByName("dave"));
        verify(repo).findByUsername("dave");
    }

    @Test
    void getUserByNamePropagatesNotFound() {
        when(repo.findByUsername("ghost"))
            .thenThrow(new WebApplicationException("User not found", Response.Status.NOT_FOUND));

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> service.getUserByName("ghost"));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void loginUserReturnsSessionStringWhenAuthenticated() {
        when(repo.authenticate("erin", "secret")).thenReturn(true);

        String session = service.loginUser("erin", "secret");

        assertTrue(session.startsWith("logged in user session: erin/"));
        verify(repo).authenticate("erin", "secret");
    }

    @Test
    void loginUserRejectsInvalidCredentials() {
        when(repo.authenticate("erin", "wrong")).thenReturn(false);

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> service.loginUser("erin", "wrong"));
        assertEquals(Response.Status.BAD_REQUEST.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void logoutUserIsNoOp() {
        service.logoutUser();
        verifyNoInteractions(repo);
    }

    @Test
    void updateUserDelegatesToRepository() {
        User user = new User().username("frank").email("frank@example.com");
        service.updateUser("frank", user);
        verify(repo).update("frank", user);
    }
}
