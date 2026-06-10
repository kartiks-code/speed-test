package org.openapitools.persistence;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.openapitools.model.Category;
import org.openapitools.model.Order;
import org.openapitools.model.Pet;
import org.openapitools.model.Tag;
import org.openapitools.model.User;
import org.postgresql.util.PGobject;
import org.springframework.dao.EmptyResultDataAccessException;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.core.RowMapper;

import java.sql.ResultSet;
import java.sql.Timestamp;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.startsWith;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Unit tests for {@link PetStore}. {@link JdbcTemplate} is mocked so these run
 * without a live database. They exercise validation, ID generation, the JSON /
 * enum {@link PGobject} write path, and the {@link RowMapper}s (captured from the
 * mocked template and invoked against a mocked {@link ResultSet}).
 */
@ExtendWith(MockitoExtension.class)
class PetStoreTest {

    @Mock
    private JdbcTemplate jdbc;

    // A real ObjectMapper so JSON serialization in pgJson / fromJson is genuinely exercised.
    private final ObjectMapper objectMapper = new ObjectMapper();

    private PetStore store() {
        return new PetStore(jdbc, objectMapper);
    }

    // ---------------------------------------------------------------------
    // Validation
    // ---------------------------------------------------------------------

    @Test
    void createPetRejectsMissingName() {
        Pet pet = new Pet().photoUrls(List.of("http://x/1.jpg"));
        assertThatThrownBy(() -> store().createPet(pet))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("name is required");
    }

    @Test
    void createPetRejectsMissingPhotoUrls() {
        Pet pet = new Pet().name("Fido");
        pet.setPhotoUrls(List.of());
        assertThatThrownBy(() -> store().createPet(pet))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("photoUrls is required");
    }

    @Test
    void updatePetRejectsMissingId() {
        Pet pet = new Pet().name("Fido").photoUrls(List.of("http://x/1.jpg"));
        assertThatThrownBy(() -> store().updatePet(pet))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("id is required");
    }

    @Test
    void updatePetFieldsRejectsInvalidStatus() {
        assertThatThrownBy(() -> store().updatePetFields(1L, null, "teleported"))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("invalid pet status");
    }

    @Test
    void createUserRejectsMissingUsername() {
        assertThatThrownBy(() -> store().createUser(new User()))
                .isInstanceOf(InvalidInputException.class)
                .hasMessageContaining("username is required");
    }

    // ---------------------------------------------------------------------
    // ID generation + JSON / enum write path
    // ---------------------------------------------------------------------

    @Test
    void createPetGeneratesIdAndSerializesJsonAndEnum() {
        Pet pet = new Pet()
                .name("Fido")
                .photoUrls(List.of("http://example.com/fido.jpg"))
                .category(new Category().id(2L).name("Dogs"))
                .status(Pet.StatusEnum.AVAILABLE);

        when(jdbc.queryForObject("SELECT COALESCE(MAX(id), 0) + 1 FROM pet", Long.class))
                .thenReturn(5L);
        Pet stored = new Pet().id(5L).name("Fido");
        when(jdbc.queryForObject(startsWith("SELECT id, name, category"), any(RowMapper.class), eq(5L)))
                .thenReturn(stored);

        Pet result = store().createPet(pet);

        assertThat(result).isSameAs(stored);
        assertThat(pet.getId()).isEqualTo(5L);

        ArgumentCaptor<Object> args = ArgumentCaptor.forClass(Object.class);
        verify(jdbc).update(startsWith("INSERT INTO pet"),
                args.capture(), args.capture(), args.capture(),
                args.capture(), args.capture(), args.capture());
        List<Object> values = args.getAllValues();
        assertThat(values.get(0)).isEqualTo(5L);
        assertThat(values.get(1)).isEqualTo("Fido");

        PGobject category = (PGobject) values.get(2);
        assertThat(category.getType()).isEqualTo("json");
        assertThat(category.getValue()).isEqualTo("{\"id\":2,\"name\":\"Dogs\"}");

        PGobject photoUrls = (PGobject) values.get(3);
        assertThat(photoUrls.getType()).isEqualTo("json");
        assertThat(photoUrls.getValue()).isEqualTo("[\"http://example.com/fido.jpg\"]");

        PGobject status = (PGobject) values.get(5);
        assertThat(status.getType()).isEqualTo("pet_status");
        assertThat(status.getValue()).isEqualTo("available");
    }

    @Test
    void placeOrderGeneratesIdWhenMissing() {
        Order order = new Order().petId(1L).quantity(2).status(Order.StatusEnum.PLACED);
        when(jdbc.queryForObject("SELECT COALESCE(MAX(id), 0) + 1 FROM \"order\"", Long.class))
                .thenReturn(10L);

        store().placeOrder(order);

        assertThat(order.getId()).isEqualTo(10L);
        ArgumentCaptor<Object> args = ArgumentCaptor.forClass(Object.class);
        verify(jdbc).update(startsWith("INSERT INTO \"order\""),
                args.capture(), args.capture(), args.capture(),
                args.capture(), args.capture(), args.capture());
        assertThat(args.getAllValues().get(0)).isEqualTo(10L);
        PGobject status = (PGobject) args.getAllValues().get(4);
        assertThat(status.getType()).isEqualTo("order_status");
        assertThat(status.getValue()).isEqualTo("placed");
    }

    // ---------------------------------------------------------------------
    // Query behaviour
    // ---------------------------------------------------------------------

    @Test
    void getPetByIdMapsEmptyResultToNotFound() {
        when(jdbc.queryForObject(anyString(), any(RowMapper.class), eq(7L)))
                .thenThrow(new EmptyResultDataAccessException(1));

        assertThatThrownBy(() -> store().getPetById(7L))
                .isInstanceOf(NotFoundException.class)
                .hasMessageContaining("pet 7 not found");
    }

    @Test
    void getUserByUsernameMapsEmptyResultToNotFound() {
        when(jdbc.queryForObject(anyString(), any(RowMapper.class), eq("ghost")))
                .thenThrow(new EmptyResultDataAccessException(1));

        assertThatThrownBy(() -> store().getUserByUsername("ghost"))
                .isInstanceOf(NotFoundException.class)
                .hasMessageContaining("user ghost not found");
    }

    @Test
    void findPetsByStatusDefaultsToAvailableWhenEmpty() {
        ArgumentCaptor<String> sql = ArgumentCaptor.forClass(String.class);

        store().findPetsByStatus(List.of());

        verify(jdbc).query(sql.capture(), any(RowMapper.class), eq("available"));
        assertThat(sql.getValue()).contains("status::text IN (?)");
    }

    @Test
    void findPetsByTagsReturnsEmptyWithoutQueryingWhenNoTags() {
        List<Pet> result = store().findPetsByTags(List.of());

        assertThat(result).isEmpty();
        verify(jdbc, never()).query(anyString(), any(RowMapper.class), any(Object[].class));
    }

    @Test
    void updatePetFieldsBuildsUpdateForNameAndStatus() {
        when(jdbc.update(anyString(), any(Object[].class))).thenReturn(1);

        boolean updated = store().updatePetFields(3L, "Rex", "sold");

        assertThat(updated).isTrue();
        ArgumentCaptor<String> sql = ArgumentCaptor.forClass(String.class);
        ArgumentCaptor<Object> args = ArgumentCaptor.forClass(Object.class);
        verify(jdbc).update(sql.capture(), args.capture(), args.capture(), args.capture());
        assertThat(sql.getValue()).startsWith("UPDATE pet SET");
        assertThat(sql.getValue()).contains("name = ?").contains("status = ?").endsWith("WHERE id = ?");
        List<Object> values = args.getAllValues();
        assertThat(values.get(0)).isEqualTo("Rex");
        PGobject status = (PGobject) values.get(1);
        assertThat(status.getType()).isEqualTo("pet_status");
        assertThat(status.getValue()).isEqualTo("sold");
        assertThat(values.get(2)).isEqualTo(3L);
    }

    @Test
    void inventoryAggregatesCountsByStatus() {
        when(jdbc.queryForList(anyString())).thenReturn(List.of(
                Map.of("status", "available", "cnt", 3),
                Map.of("status", "sold", "cnt", 1)));

        Map<String, Integer> inventory = store().inventory();

        assertThat(inventory).containsEntry("available", 3).containsEntry("sold", 1);
    }

    @Test
    void authenticateUserReturnsTrueWhenCredentialsMatch() {
        when(jdbc.queryForObject(anyString(), eq(Boolean.class), eq("alice"), eq("secret")))
                .thenReturn(true);

        assertThat(store().authenticateUser("alice", "secret")).isTrue();
    }

    @Test
    void authenticateUserReturnsFalseWhenNoMatch() {
        when(jdbc.queryForObject(anyString(), eq(Boolean.class), eq("alice"), eq("nope")))
                .thenReturn(false);

        assertThat(store().authenticateUser("alice", "nope")).isFalse();
    }

    // ---------------------------------------------------------------------
    // RowMappers (captured from the mocked template, invoked on a mocked ResultSet)
    // ---------------------------------------------------------------------

    @Test
    @SuppressWarnings("unchecked")
    void petRowMapperDeserializesJsonColumnsAndEnum() throws Exception {
        ArgumentCaptor<RowMapper> mapperCaptor = ArgumentCaptor.forClass(RowMapper.class);
        when(jdbc.queryForObject(anyString(), mapperCaptor.capture(), eq(1L))).thenReturn(new Pet());

        store().getPetById(1L);
        RowMapper<Pet> mapper = mapperCaptor.getValue();

        ResultSet rs = mock(ResultSet.class);
        when(rs.getLong("id")).thenReturn(1L);
        when(rs.getString("name")).thenReturn("Fido");
        when(rs.getString("category")).thenReturn("{\"id\":2,\"name\":\"Dogs\"}");
        when(rs.getString("photo_urls")).thenReturn("[\"http://example.com/fido.jpg\"]");
        when(rs.getString("tags")).thenReturn("[{\"id\":3,\"name\":\"cute\"}]");
        when(rs.getString("status")).thenReturn("available");

        Pet pet = mapper.mapRow(rs, 1);

        assertThat(pet.getId()).isEqualTo(1L);
        assertThat(pet.getName()).isEqualTo("Fido");
        assertThat(pet.getCategory()).isEqualTo(new Category().id(2L).name("Dogs"));
        assertThat(pet.getPhotoUrls()).containsExactly("http://example.com/fido.jpg");
        assertThat(pet.getTags()).containsExactly(new Tag().id(3L).name("cute"));
        assertThat(pet.getStatus()).isEqualTo(Pet.StatusEnum.AVAILABLE);
    }

    @Test
    @SuppressWarnings("unchecked")
    void orderRowMapperMapsTimestampAndEnum() throws Exception {
        ArgumentCaptor<RowMapper> mapperCaptor = ArgumentCaptor.forClass(RowMapper.class);
        when(jdbc.queryForObject(anyString(), mapperCaptor.capture(), eq(10L))).thenReturn(new Order());

        store().getOrderById(10L);
        RowMapper<Order> mapper = mapperCaptor.getValue();

        OffsetDateTime when = OffsetDateTime.of(2026, 1, 2, 3, 4, 5, 0, ZoneOffset.UTC);
        ResultSet rs = mock(ResultSet.class);
        when(rs.getLong("id")).thenReturn(10L);
        when(rs.getLong("pet_id")).thenReturn(1L);
        when(rs.getInt("quantity")).thenReturn(2);
        when(rs.getTimestamp("ship_date")).thenReturn(Timestamp.from(when.toInstant()));
        when(rs.getString("status")).thenReturn("placed");
        when(rs.getBoolean("complete")).thenReturn(true);

        Order order = mapper.mapRow(rs, 1);

        assertThat(order.getId()).isEqualTo(10L);
        assertThat(order.getPetId()).isEqualTo(1L);
        assertThat(order.getQuantity()).isEqualTo(2);
        assertThat(order.getShipDate()).isEqualTo(when);
        assertThat(order.getStatus()).isEqualTo(Order.StatusEnum.PLACED);
        assertThat(order.getComplete()).isTrue();
    }

    @Test
    @SuppressWarnings("unchecked")
    void userRowMapperMapsAllColumns() throws Exception {
        ArgumentCaptor<RowMapper> mapperCaptor = ArgumentCaptor.forClass(RowMapper.class);
        when(jdbc.queryForObject(anyString(), mapperCaptor.capture(), eq("alice"))).thenReturn(new User());

        store().getUserByUsername("alice");
        RowMapper<User> mapper = mapperCaptor.getValue();

        ResultSet rs = mock(ResultSet.class);
        when(rs.getLong("id")).thenReturn(1L);
        when(rs.getString("username")).thenReturn("alice");
        when(rs.getString("first_name")).thenReturn("Alice");
        when(rs.getString("last_name")).thenReturn("Smith");
        when(rs.getString("email")).thenReturn("alice@example.com");
        when(rs.getString("password")).thenReturn("secret");
        when(rs.getString("phone")).thenReturn("555-0100");
        when(rs.getInt("user_status")).thenReturn(1);

        User user = mapper.mapRow(rs, 1);

        assertThat(user.getId()).isEqualTo(1L);
        assertThat(user.getUsername()).isEqualTo("alice");
        assertThat(user.getFirstName()).isEqualTo("Alice");
        assertThat(user.getLastName()).isEqualTo("Smith");
        assertThat(user.getEmail()).isEqualTo("alice@example.com");
        assertThat(user.getPassword()).isEqualTo("secret");
        assertThat(user.getPhone()).isEqualTo("555-0100");
        assertThat(user.getUserStatus()).isEqualTo(1);
    }
}
