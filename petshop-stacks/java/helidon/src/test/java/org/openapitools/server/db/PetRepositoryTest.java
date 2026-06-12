package org.openapitools.server.db;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.util.List;
import java.util.Map;

import javax.sql.DataSource;

import com.fasterxml.jackson.databind.ObjectMapper;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.openapitools.server.model.Category;
import org.openapitools.server.model.Pet;
import org.openapitools.server.model.Tag;

/**
 * Unit tests for {@link PetRepository}. The full JDBC chain
 * (DataSourceProvider -> DataSource -> Connection -> PreparedStatement -> ResultSet) is mocked,
 * so these run without a live database. They assert SQL text, bound parameters (including the
 * Jackson JSON serialization for {@code category}/{@code photo_urls}/{@code tags} and the enum
 * casts), and ResultSet -> model mapping.
 */
class PetRepositoryTest {

    private final ObjectMapper mapper = new ObjectMapper();

    private DataSourceProvider dsProvider;
    private DataSource dataSource;
    private Connection connection;
    private PreparedStatement statement;
    private ResultSet resultSet;
    private PetRepository repository;

    @BeforeEach
    void setUp() throws Exception {
        dsProvider = mock(DataSourceProvider.class);
        dataSource = mock(DataSource.class);
        connection = mock(Connection.class);
        statement = mock(PreparedStatement.class);
        resultSet = mock(ResultSet.class);

        when(dsProvider.get()).thenReturn(dataSource);
        when(dataSource.getConnection()).thenReturn(connection);
        when(connection.prepareStatement(anyString())).thenReturn(statement);
        when(statement.executeQuery()).thenReturn(resultSet);

        repository = new PetRepository();
        repository.dsProvider = dsProvider;
    }

    private String capturedSql() throws Exception {
        ArgumentCaptor<String> sql = ArgumentCaptor.forClass(String.class);
        verify(connection).prepareStatement(sql.capture());
        return sql.getValue();
    }

    @Test
    void addUsesProvidedIdAndSerializesJsonColumns() throws Exception {
        Category category = new Category().id(2L).name("dogs");
        Tag tag = new Tag().id(8L).name("cute");
        Pet pet = new Pet()
            .id(42L)
            .name("Fido")
            .category(category)
            .photoUrls(List.of("http://x/1.jpg"))
            .tags(List.of(tag))
            .status(Pet.StatusEnum.AVAILABLE);

        Pet result = repository.add(pet);

        assertSame(pet, result);
        assertEquals(42L, result.getId());

        String sql = capturedSql();
        assertTrue(sql.contains("INSERT INTO pet"));
        assertTrue(sql.contains("cast(? as json)"));
        assertTrue(sql.contains("cast(? as pet_status)"));

        verify(statement).setLong(1, 42L);
        verify(statement).setString(2, "Fido");
        verify(statement).setString(3, mapper.writeValueAsString(category));
        verify(statement).setString(4, mapper.writeValueAsString(List.of("http://x/1.jpg")));
        verify(statement).setString(5, mapper.writeValueAsString(List.of(tag)));
        verify(statement).setString(6, "available");
        verify(statement).executeUpdate();
    }

    @Test
    void addGeneratesIdWhenMissingAndPassesNullForOptionalJson() throws Exception {
        Pet pet = new Pet().name("Anon").photoUrls(List.of());

        Pet result = repository.add(pet);

        assertNotNull(result.getId());
        verify(statement).setString(3, null);
        verify(statement).setString(5, null);
        verify(statement).setString(6, null);
    }

    @Test
    void updateRejectsMissingId() {
        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.update(new Pet().name("x")));
        assertEquals(Response.Status.BAD_REQUEST.getStatusCode(), ex.getResponse().getStatus());
        verifyNoInteractions(dsProvider);
    }

    @Test
    void updateReturnsPetWhenRowAffected() throws Exception {
        when(statement.executeUpdate()).thenReturn(1);
        Pet pet = new Pet().id(5L).name("Rex").photoUrls(List.of()).status(Pet.StatusEnum.SOLD);

        Pet result = repository.update(pet);

        assertSame(pet, result);
        String sql = capturedSql();
        assertTrue(sql.contains("UPDATE pet SET"));
        assertTrue(sql.contains("status = cast(? as pet_status)"));
        verify(statement).setLong(6, 5L);
        verify(statement).setString(5, "sold");
    }

    @Test
    void updateThrowsNotFoundWhenNoRowAffected() throws Exception {
        when(statement.executeUpdate()).thenReturn(0);
        Pet pet = new Pet().id(123L).name("Ghost").photoUrls(List.of());

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.update(pet));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void findByIdMapsRow() throws Exception {
        Category category = new Category().id(1L).name("dogs");
        Tag tag = new Tag().id(3L).name("fluffy");
        when(resultSet.next()).thenReturn(true);
        when(resultSet.getObject("id", Long.class)).thenReturn(77L);
        when(resultSet.getString("name")).thenReturn("Rex");
        when(resultSet.getString("category")).thenReturn(mapper.writeValueAsString(category));
        when(resultSet.getString("photo_urls")).thenReturn(mapper.writeValueAsString(List.of("u1")));
        when(resultSet.getString("tags")).thenReturn(mapper.writeValueAsString(List.of(tag)));
        when(resultSet.getString("status")).thenReturn("pending");

        Pet pet = repository.findById(77L);

        assertEquals(77L, pet.getId());
        assertEquals("Rex", pet.getName());
        assertEquals("dogs", pet.getCategory().getName());
        assertEquals(List.of("u1"), pet.getPhotoUrls());
        assertEquals(1, pet.getTags().size());
        assertEquals("fluffy", pet.getTags().get(0).getName());
        assertEquals(Pet.StatusEnum.PENDING, pet.getStatus());

        String sql = capturedSql();
        assertTrue(sql.contains("status::text"));
        verify(statement).setLong(1, 77L);
    }

    @Test
    void findByIdLeavesNullableColumnsUnsetWhenAbsent() throws Exception {
        when(resultSet.next()).thenReturn(true);
        when(resultSet.getObject("id", Long.class)).thenReturn(1L);
        when(resultSet.getString("name")).thenReturn("Bare");
        when(resultSet.getString("category")).thenReturn(null);
        when(resultSet.getString("photo_urls")).thenReturn(null);
        when(resultSet.getString("tags")).thenReturn(null);
        when(resultSet.getString("status")).thenReturn(null);

        Pet pet = repository.findById(1L);

        assertNull(pet.getCategory());
        assertNull(pet.getTags());
        assertNull(pet.getStatus());
    }

    @Test
    void findByIdThrowsNotFoundWhenNoRow() throws Exception {
        when(resultSet.next()).thenReturn(false);

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findById(9L));
        assertEquals(Response.Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
    }

    @Test
    void findByStatusBindsStatusAndCastsEnum() throws Exception {
        when(resultSet.next()).thenReturn(true, false);
        when(resultSet.getObject("id", Long.class)).thenReturn(1L);
        when(resultSet.getString("name")).thenReturn("A");
        when(resultSet.getString("status")).thenReturn("available");

        List<Pet> pets = repository.findByStatus("available");

        assertEquals(1, pets.size());
        String sql = capturedSql();
        assertTrue(sql.contains("status = cast(? as pet_status)"));
        verify(statement).setString(1, "available");
    }

    @Test
    void findByTagsReturnsEmptyWithoutDbForEmptyInput() {
        assertTrue(repository.findByTags(List.of()).isEmpty());
        assertTrue(repository.findByTags(null).isEmpty());
        verifyNoInteractions(dsProvider);
    }

    @Test
    void findByTagsBuildsOneContainmentClausePerTag() throws Exception {
        when(resultSet.next()).thenReturn(false);

        repository.findByTags(List.of("cute", "small"));

        String sql = capturedSql();
        int occurrences = sql.split("tags::jsonb @> cast\\(\\? as jsonb\\)", -1).length - 1;
        assertEquals(2, occurrences);
        verify(statement).setString(eq(1), anyString());
        verify(statement).setString(eq(2), anyString());
    }

    @Test
    void getInventoryAggregatesByStatus() throws Exception {
        when(resultSet.next()).thenReturn(true, true, false);
        when(resultSet.getString(1)).thenReturn("available", "sold");
        when(resultSet.getInt(2)).thenReturn(5, 2);

        Map<String, Integer> inventory = repository.getInventory();

        assertEquals(5, inventory.get("available"));
        assertEquals(2, inventory.get("sold"));
        assertTrue(capturedSql().contains("GROUP BY status"));
    }

    @Test
    void updateWithFormSkipsDbWhenNoFields() {
        repository.updateWithForm(1L, null, null);
        verifyNoInteractions(dsProvider);
    }

    @Test
    void updateWithFormBuildsNameAndStatusClauses() throws Exception {
        repository.updateWithForm(7L, "Buddy", "sold");

        String sql = capturedSql();
        assertTrue(sql.contains("\"name\" = ?"));
        assertTrue(sql.contains("status = cast(? as pet_status)"));
        verify(statement).setObject(1, "Buddy");
        verify(statement).setObject(2, "sold");
        verify(statement).setObject(3, 7L);
        verify(statement).executeUpdate();
    }

    @Test
    void deleteBindsIdAndExecutes() throws Exception {
        repository.delete(15L);
        assertTrue(capturedSql().contains("DELETE FROM pet"));
        verify(statement).setLong(1, 15L);
        verify(statement).executeUpdate();
    }

    @Test
    void wrapsSqlFailuresAsInternalServerError() throws Exception {
        when(dataSource.getConnection()).thenThrow(new java.sql.SQLException("boom"));

        WebApplicationException ex =
            assertThrows(WebApplicationException.class, () -> repository.findById(1L));
        assertEquals(Response.Status.INTERNAL_SERVER_ERROR.getStatusCode(), ex.getResponse().getStatus());
    }
}
