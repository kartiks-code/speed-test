package org.openapitools.api;

import org.openapitools.model.User;
import org.openapitools.persistence.InvalidInputException;
import org.openapitools.persistence.PetStore;
import org.springframework.http.HttpHeaders;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.context.request.NativeWebRequest;

import jakarta.validation.Valid;
import org.springframework.lang.Nullable;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.List;
import java.util.Optional;

@Controller
@RequestMapping("${openapi.swaggerPetstoreOpenAPI31.base-path:/api/v3}")
public class UserApiController implements UserApi {

    private final NativeWebRequest request;
    private final PetStore petStore;

    public UserApiController(NativeWebRequest request, PetStore petStore) {
        this.request = request;
        this.petStore = petStore;
    }

    @Override
    public Optional<NativeWebRequest> getRequest() {
        return Optional.ofNullable(request);
    }

    @Override
    public ResponseEntity<User> createUser(@Valid @Nullable User user) {
        if (user == null) throw new InvalidInputException("user body is required");
        return ResponseEntity.ok(petStore.createUser(user));
    }

    @Override
    public ResponseEntity<User> createUsersWithListInput(@Valid @Nullable List<@Valid User> user) {
        if (user == null || user.isEmpty()) throw new InvalidInputException("user list is required");
        return ResponseEntity.ok(petStore.createUsers(user));
    }

    @Override
    public ResponseEntity<User> getUserByName(String username) {
        return ResponseEntity.ok(petStore.getUserByUsername(username));
    }

    @Override
    public ResponseEntity<String> loginUser(@Nullable String username, @Nullable String password) {
        if (username == null || username.isBlank() || password == null || password.isBlank()) {
            throw new InvalidInputException("username and password are required");
        }
        boolean ok = petStore.authenticateUser(username, password);
        if (!ok) {
            throw new InvalidInputException("invalid username or password");
        }
        String expires = OffsetDateTime.now(ZoneOffset.UTC).plusHours(1)
                .format(DateTimeFormatter.ISO_OFFSET_DATE_TIME);
        return ResponseEntity.ok()
                .header("X-Rate-Limit", "5000")
                .header("X-Expires-After", expires)
                .body("logged in user session: " + username);
    }

    @Override
    public ResponseEntity<Void> logoutUser() {
        return ResponseEntity.ok().build();
    }

    @Override
    public ResponseEntity<Void> updateUser(String username, @Valid @Nullable User user) {
        if (user == null) throw new InvalidInputException("user body is required");
        petStore.updateUser(username, user);
        return ResponseEntity.ok().build();
    }

    @Override
    public ResponseEntity<Void> deleteUser(String username) {
        petStore.deleteUser(username);
        return ResponseEntity.ok().build();
    }
}
