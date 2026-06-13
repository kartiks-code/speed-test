package petstore

import (
	"context"
	"errors"
	"io"
	"net/http"
	"testing"
	"time"
)

type fakeStore struct {
	pets   map[int64]Pet
	orders map[int64]Order
	users  map[string]User
	photos map[int64][][]byte
}

func newFakeStore() *fakeStore {
	return &fakeStore{
		pets:   map[int64]Pet{},
		orders: map[int64]Order{},
		users:  map[string]User{},
		photos: map[int64][][]byte{},
	}
}

func (s *fakeStore) SavePetPhoto(ctx context.Context, petID int64, content []byte, contentType string, metadata string) error {
	s.photos[petID] = append(s.photos[petID], content)
	return nil
}

func (s *fakeStore) Close() error { return nil }

func (s *fakeStore) CreatePet(ctx context.Context, pet Pet) (Pet, error) {
	s.pets[pet.Id] = pet
	return pet, nil
}

func (s *fakeStore) UpdatePet(ctx context.Context, pet Pet) (Pet, error) {
	if _, ok := s.pets[pet.Id]; !ok {
		return Pet{}, ErrNotFound
	}
	s.pets[pet.Id] = pet
	return pet, nil
}

func (s *fakeStore) GetPetByID(ctx context.Context, id int64) (Pet, error) {
	pet, ok := s.pets[id]
	if !ok {
		return Pet{}, ErrNotFound
	}
	return pet, nil
}

func (s *fakeStore) DeletePet(ctx context.Context, id int64) (bool, error) {
	if _, ok := s.pets[id]; !ok {
		return false, nil
	}
	delete(s.pets, id)
	return true, nil
}

func (s *fakeStore) FindPetsByStatus(ctx context.Context, statuses []string) ([]Pet, error) {
	var pets []Pet
	allowed := make(map[string]bool, len(statuses))
	for _, status := range statuses {
		allowed[status] = true
	}
	for _, pet := range s.pets {
		if allowed[pet.Status] {
			pets = append(pets, pet)
		}
	}
	return pets, nil
}

func (s *fakeStore) FindPetsByTags(ctx context.Context, tags []string) ([]Pet, error) {
	var pets []Pet
	allowed := make(map[string]bool, len(tags))
	for _, tag := range tags {
		allowed[tag] = true
	}
	for _, pet := range s.pets {
		for _, tag := range pet.Tags {
			if allowed[tag.Name] {
				pets = append(pets, pet)
				break
			}
		}
	}
	return pets, nil
}

func (s *fakeStore) UpdatePetFields(ctx context.Context, id int64, name *string, status *string) (bool, error) {
	pet, ok := s.pets[id]
	if !ok {
		return false, nil
	}
	if name != nil {
		pet.Name = *name
	}
	if status != nil {
		pet.Status = *status
	}
	s.pets[id] = pet
	return true, nil
}

func (s *fakeStore) Inventory(ctx context.Context) (map[string]int32, error) {
	inventory := map[string]int32{}
	for _, pet := range s.pets {
		inventory[pet.Status]++
	}
	return inventory, nil
}

func (s *fakeStore) CreateOrder(ctx context.Context, order Order) (Order, error) {
	s.orders[order.Id] = order
	return order, nil
}

func (s *fakeStore) GetOrderByID(ctx context.Context, id int64) (Order, error) {
	order, ok := s.orders[id]
	if !ok {
		return Order{}, ErrNotFound
	}
	return order, nil
}

func (s *fakeStore) DeleteOrder(ctx context.Context, id int64) (bool, error) {
	if _, ok := s.orders[id]; !ok {
		return false, nil
	}
	delete(s.orders, id)
	return true, nil
}

func (s *fakeStore) CreateUser(ctx context.Context, user User) (User, error) {
	s.users[user.Username] = user
	return user, nil
}

func (s *fakeStore) CreateUsers(ctx context.Context, users []User) (User, error) {
	var last User
	for _, user := range users {
		s.users[user.Username] = user
		last = user
	}
	return last, nil
}

func (s *fakeStore) GetUserByUsername(ctx context.Context, username string) (User, error) {
	user, ok := s.users[username]
	if !ok {
		return User{}, ErrNotFound
	}
	return user, nil
}

func (s *fakeStore) UpdateUser(ctx context.Context, username string, user User) (bool, error) {
	if _, ok := s.users[username]; !ok {
		return false, nil
	}
	if user.Username == "" {
		user.Username = username
	}
	delete(s.users, username)
	s.users[user.Username] = user
	return true, nil
}

func (s *fakeStore) DeleteUser(ctx context.Context, username string) (bool, error) {
	if _, ok := s.users[username]; !ok {
		return false, nil
	}
	delete(s.users, username)
	return true, nil
}

func (s *fakeStore) AuthenticateUser(ctx context.Context, username string, password string) (bool, error) {
	user, ok := s.users[username]
	return ok && user.Password == password, nil
}

func TestPetAPIUsesStoreForCRUD(t *testing.T) {
	store := newFakeStore()
	app := testApp(t, store)

	pet := Pet{Id: 10, Name: "doggie", PhotoUrls: []string{"https://example.test/dog.png"}, Status: "available"}
	resp := doRequest(t, app, http.MethodPost, "/api/v3/pet", pet)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("AddPet status = %d, want %d", resp.StatusCode, http.StatusOK)
	}

	var created Pet
	decodeBody(t, resp, &created)
	if created.Id != pet.Id || created.Name != pet.Name {
		t.Fatalf("created pet = %#v, want %#v", created, pet)
	}

	resp = doRequest(t, app, http.MethodGet, "/api/v3/pet/10", nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("GetPetById status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	resp.Body.Close()

	updatedName := "doggie-updated"
	resp = doRequest(t, app, http.MethodPost, "/api/v3/pet/10?name="+updatedName+"&status=sold", nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("UpdatePetWithForm status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	resp.Body.Close()
	if store.pets[10].Name != updatedName || store.pets[10].Status != "sold" {
		t.Fatalf("updated pet = %#v", store.pets[10])
	}

	resp = doRequest(t, app, http.MethodDelete, "/api/v3/pet/10", nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("DeletePet status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	resp.Body.Close()

	resp = doRequest(t, app, http.MethodGet, "/api/v3/pet/10", nil)
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("GetPetById missing status = %d, want %d", resp.StatusCode, http.StatusNotFound)
	}
	resp.Body.Close()
}

func TestStoreAndUserAPIsUseStore(t *testing.T) {
	store := newFakeStore()
	store.pets[1] = Pet{Id: 1, Name: "cat", PhotoUrls: []string{"cat"}, Status: "available"}
	store.pets[2] = Pet{Id: 2, Name: "dog", PhotoUrls: []string{"dog"}, Status: "sold"}
	app := testApp(t, store)

	order := Order{Id: 99, PetId: 1, Quantity: 2, ShipDate: time.Date(2026, 6, 9, 0, 0, 0, 0, time.UTC), Status: "placed", Complete: false}
	resp := doRequest(t, app, http.MethodPost, "/api/v3/store/order", order)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("PlaceOrder status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	resp.Body.Close()

	resp = doRequest(t, app, http.MethodGet, "/api/v3/store/inventory", nil)
	var inventory map[string]int32
	decodeBody(t, resp, &inventory)
	if inventory["available"] != 1 || inventory["sold"] != 1 {
		t.Fatalf("inventory = %#v", inventory)
	}

	user := User{Id: 7, Username: "user1", FirstName: "First", LastName: "Last", Password: "secret"}
	resp = doRequest(t, app, http.MethodPost, "/api/v3/user", user)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("CreateUser status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	resp.Body.Close()

	resp = doRequest(t, app, http.MethodGet, "/api/v3/user/login?username=user1&password=secret", nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("LoginUser status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	body, _ := io.ReadAll(resp.Body)
	resp.Body.Close()
	if len(body) == 0 {
		t.Fatal("LoginUser body is empty, want session token")
	}

	resp = doRequest(t, app, http.MethodGet, "/api/v3/user/missing", nil)
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("GetUserByName missing status = %d, want %d", resp.StatusCode, http.StatusNotFound)
	}
	resp.Body.Close()
}

func TestHandlersReturnServerErrorWhenStoreIsMissing(t *testing.T) {
	app := testApp(t, nil)
	resp := doRequest(t, app, http.MethodPost, "/api/v3/pet", Pet{Id: 1, Name: "dog", PhotoUrls: []string{"dog"}})
	if resp.StatusCode != http.StatusInternalServerError {
		t.Fatalf("AddPet without store status = %d, want %d", resp.StatusCode, http.StatusInternalServerError)
	}
	resp.Body.Close()
}

func TestErrorStatusMapping(t *testing.T) {
	if statusForError(ErrNotFound) != http.StatusNotFound {
		t.Fatalf("ErrNotFound status = %d, want %d", statusForError(ErrNotFound), http.StatusNotFound)
	}
	if statusForError(errors.New("boom")) != http.StatusInternalServerError {
		t.Fatalf("unexpected error status = %d, want %d", statusForError(errors.New("boom")), http.StatusInternalServerError)
	}
}
