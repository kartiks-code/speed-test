package petstore

import (
	"context"
	"os"
	"testing"
	"time"
)

func TestPostgresStorePetOrderAndUserCRUD(t *testing.T) {
	dsn := os.Getenv("TEST_DATABASE_DSN")
	if dsn == "" {
		t.Skip("set TEST_DATABASE_DSN to run PostgreSQL integration tests")
	}

	ctx := context.Background()
	store, err := NewPostgresStore(ctx, dsn)
	if err != nil {
		t.Fatalf("NewPostgresStore: %v", err)
	}
	defer store.Close()

	cleanupPostgresStore(t, store)

	pet := Pet{
		Id:        101,
		Name:      "doggie",
		Category:  Category{Id: 1, Name: "dogs"},
		PhotoUrls: []string{"https://example.test/doggie.png"},
		Tags:      []Tag{{Id: 2, Name: "friendly"}},
		Status:    "available",
	}
	createdPet, err := store.CreatePet(ctx, pet)
	if err != nil {
		t.Fatalf("CreatePet: %v", err)
	}
	if createdPet.Id != pet.Id || createdPet.Category.Name != "dogs" || len(createdPet.PhotoUrls) != 1 {
		t.Fatalf("createdPet = %#v", createdPet)
	}

	pet.Name = "doggie-updated"
	updatedPet, err := store.UpdatePet(ctx, pet)
	if err != nil {
		t.Fatalf("UpdatePet: %v", err)
	}
	if updatedPet.Name != "doggie-updated" {
		t.Fatalf("updatedPet.Name = %q", updatedPet.Name)
	}

	pets, err := store.FindPetsByTags(ctx, []string{"friendly"})
	if err != nil {
		t.Fatalf("FindPetsByTags: %v", err)
	}
	if len(pets) != 1 || pets[0].Id != pet.Id {
		t.Fatalf("pets by tag = %#v", pets)
	}

	order := Order{Id: 501, PetId: pet.Id, Quantity: 3, ShipDate: time.Now().UTC().Truncate(time.Second), Status: "placed", Complete: true}
	createdOrder, err := store.CreateOrder(ctx, order)
	if err != nil {
		t.Fatalf("CreateOrder: %v", err)
	}
	if createdOrder.Id != order.Id || createdOrder.PetId != pet.Id {
		t.Fatalf("createdOrder = %#v", createdOrder)
	}

	user := User{Id: 701, Username: "user1", FirstName: "First", LastName: "Last", Password: "secret", UserStatus: 1}
	createdUser, err := store.CreateUser(ctx, user)
	if err != nil {
		t.Fatalf("CreateUser: %v", err)
	}
	if createdUser.Username != user.Username {
		t.Fatalf("createdUser = %#v", createdUser)
	}

	ok, err := store.AuthenticateUser(ctx, user.Username, user.Password)
	if err != nil {
		t.Fatalf("AuthenticateUser: %v", err)
	}
	if !ok {
		t.Fatalf("AuthenticateUser returned false")
	}

	deleted, err := store.DeletePet(ctx, pet.Id)
	if err != nil {
		t.Fatalf("DeletePet: %v", err)
	}
	if !deleted {
		t.Fatalf("DeletePet returned false")
	}
}

func cleanupPostgresStore(t *testing.T, store *PostgresStore) {
	t.Helper()
	statements := []string{
		`DELETE FROM pet`,
		`DELETE FROM "order"`,
		`DELETE FROM "user"`,
	}
	for _, statement := range statements {
		if _, err := store.db.Exec(statement); err != nil {
			t.Fatalf("%s: %v", statement, err)
		}
	}
}
