package petstore

import (
	"context"
	"errors"
	"net/http"
	"testing"
)

// mockStore is a fully programmable Store implementation. Each method delegates
// to the matching function field when set, otherwise returns zero values. This
// lets table-driven handler tests inject success, not-found, invalid-input, and
// arbitrary-error behavior on a per-call basis without a database.
type mockStore struct {
	createPet        func(context.Context, Pet) (Pet, error)
	updatePet        func(context.Context, Pet) (Pet, error)
	getPetByID       func(context.Context, int64) (Pet, error)
	deletePet        func(context.Context, int64) (bool, error)
	findPetsByStatus func(context.Context, []string) ([]Pet, error)
	findPetsByTags   func(context.Context, []string) ([]Pet, error)
	updatePetFields  func(context.Context, int64, *string, *string) (bool, error)
	inventory        func(context.Context) (map[string]int32, error)
	createOrder      func(context.Context, Order) (Order, error)
	getOrderByID     func(context.Context, int64) (Order, error)
	deleteOrder      func(context.Context, int64) (bool, error)
	createUser       func(context.Context, User) (User, error)
	createUsers      func(context.Context, []User) (User, error)
	getUserByName    func(context.Context, string) (User, error)
	updateUser       func(context.Context, string, User) (bool, error)
	deleteUser       func(context.Context, string) (bool, error)
	authenticateUser func(context.Context, string, string) (bool, error)
}

func (m *mockStore) Close() error { return nil }

func (m *mockStore) CreatePet(ctx context.Context, pet Pet) (Pet, error) {
	if m.createPet != nil {
		return m.createPet(ctx, pet)
	}
	return Pet{}, nil
}

func (m *mockStore) UpdatePet(ctx context.Context, pet Pet) (Pet, error) {
	if m.updatePet != nil {
		return m.updatePet(ctx, pet)
	}
	return Pet{}, nil
}

func (m *mockStore) GetPetByID(ctx context.Context, id int64) (Pet, error) {
	if m.getPetByID != nil {
		return m.getPetByID(ctx, id)
	}
	return Pet{}, nil
}

func (m *mockStore) DeletePet(ctx context.Context, id int64) (bool, error) {
	if m.deletePet != nil {
		return m.deletePet(ctx, id)
	}
	return true, nil
}

func (m *mockStore) FindPetsByStatus(ctx context.Context, statuses []string) ([]Pet, error) {
	if m.findPetsByStatus != nil {
		return m.findPetsByStatus(ctx, statuses)
	}
	return nil, nil
}

func (m *mockStore) FindPetsByTags(ctx context.Context, tags []string) ([]Pet, error) {
	if m.findPetsByTags != nil {
		return m.findPetsByTags(ctx, tags)
	}
	return nil, nil
}

func (m *mockStore) UpdatePetFields(ctx context.Context, id int64, name *string, status *string) (bool, error) {
	if m.updatePetFields != nil {
		return m.updatePetFields(ctx, id, name, status)
	}
	return true, nil
}

func (m *mockStore) Inventory(ctx context.Context) (map[string]int32, error) {
	if m.inventory != nil {
		return m.inventory(ctx)
	}
	return map[string]int32{}, nil
}

func (m *mockStore) CreateOrder(ctx context.Context, order Order) (Order, error) {
	if m.createOrder != nil {
		return m.createOrder(ctx, order)
	}
	return Order{}, nil
}

func (m *mockStore) GetOrderByID(ctx context.Context, id int64) (Order, error) {
	if m.getOrderByID != nil {
		return m.getOrderByID(ctx, id)
	}
	return Order{}, nil
}

func (m *mockStore) DeleteOrder(ctx context.Context, id int64) (bool, error) {
	if m.deleteOrder != nil {
		return m.deleteOrder(ctx, id)
	}
	return true, nil
}

func (m *mockStore) CreateUser(ctx context.Context, user User) (User, error) {
	if m.createUser != nil {
		return m.createUser(ctx, user)
	}
	return User{}, nil
}

func (m *mockStore) CreateUsers(ctx context.Context, users []User) (User, error) {
	if m.createUsers != nil {
		return m.createUsers(ctx, users)
	}
	return User{}, nil
}

func (m *mockStore) GetUserByUsername(ctx context.Context, username string) (User, error) {
	if m.getUserByName != nil {
		return m.getUserByName(ctx, username)
	}
	return User{}, nil
}

func (m *mockStore) UpdateUser(ctx context.Context, username string, user User) (bool, error) {
	if m.updateUser != nil {
		return m.updateUser(ctx, username, user)
	}
	return true, nil
}

func (m *mockStore) DeleteUser(ctx context.Context, username string) (bool, error) {
	if m.deleteUser != nil {
		return m.deleteUser(ctx, username)
	}
	return true, nil
}

func (m *mockStore) AuthenticateUser(ctx context.Context, username, password string) (bool, error) {
	if m.authenticateUser != nil {
		return m.authenticateUser(ctx, username, password)
	}
	return true, nil
}

var _ Store = (*mockStore)(nil)

var errBoom = errors.New("boom")

func TestPetAPIHandlers(t *testing.T) {
	validPet := Pet{Id: 1, Name: "fido", PhotoUrls: []string{"a"}, Status: "available"}

	tests := []struct {
		name     string
		store    *mockStore
		invoke   func(*PetAPI) testRequest
		wantCode int
	}{
		{
			name:  "AddPet success",
			store: &mockStore{createPet: func(_ context.Context, p Pet) (Pet, error) { return p, nil }},
			invoke: func(api *PetAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/pet", validPet, nil, api.AddPet)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "AddPet malformed json -> 400",
			store: &mockStore{},
			invoke: func(api *PetAPI) testRequest {
				return rawRequest(http.MethodPost, "/api/v3/pet", `{"name":`, nil, api.AddPet)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "AddPet invalid input -> 400",
			store: &mockStore{createPet: func(context.Context, Pet) (Pet, error) { return Pet{}, ErrInvalidInput }},
			invoke: func(api *PetAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/pet", validPet, nil, api.AddPet)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "AddPet store error -> 500",
			store: &mockStore{createPet: func(context.Context, Pet) (Pet, error) { return Pet{}, errBoom }},
			invoke: func(api *PetAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/pet", validPet, nil, api.AddPet)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "GetPetById success",
			store: &mockStore{getPetByID: func(_ context.Context, id int64) (Pet, error) { return validPet, nil }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/1", map[string]string{"petId": "1"}, api.GetPetById)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "GetPetById not found -> 404",
			store: &mockStore{getPetByID: func(context.Context, int64) (Pet, error) { return Pet{}, ErrNotFound }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/9", map[string]string{"petId": "9"}, api.GetPetById)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "GetPetById invalid id -> 400",
			store: &mockStore{},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/abc", map[string]string{"petId": "abc"}, api.GetPetById)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "DeletePet success",
			store: &mockStore{deletePet: func(context.Context, int64) (bool, error) { return true, nil }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/pet/1", map[string]string{"petId": "1"}, api.DeletePet)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "DeletePet store error -> 500",
			store: &mockStore{deletePet: func(context.Context, int64) (bool, error) { return false, errBoom }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/pet/1", map[string]string{"petId": "1"}, api.DeletePet)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "FindPetsByStatus invalid status -> 400",
			store: &mockStore{},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/findByStatus?status=bogus", nil, api.FindPetsByStatus)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name: "FindPetsByStatus default status success",
			store: &mockStore{findPetsByStatus: func(_ context.Context, statuses []string) ([]Pet, error) {
				if len(statuses) != 1 || statuses[0] != "available" {
					t.Errorf("expected default status [available], got %#v", statuses)
				}
				return []Pet{validPet}, nil
			}},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/findByStatus", nil, api.FindPetsByStatus)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "FindPetsByStatus store error -> 500",
			store: &mockStore{findPetsByStatus: func(context.Context, []string) ([]Pet, error) { return nil, errBoom }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/findByStatus?status=available", nil, api.FindPetsByStatus)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "FindPetsByTags success",
			store: &mockStore{findPetsByTags: func(context.Context, []string) ([]Pet, error) { return []Pet{validPet}, nil }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/pet/findByTags?tags=friendly", nil, api.FindPetsByTags)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "UpdatePet not found -> 404",
			store: &mockStore{updatePet: func(context.Context, Pet) (Pet, error) { return Pet{}, ErrNotFound }},
			invoke: func(api *PetAPI) testRequest {
				return jsonRequest(http.MethodPut, "/api/v3/pet", validPet, nil, api.UpdatePet)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "UpdatePetWithForm success",
			store: &mockStore{updatePetFields: func(context.Context, int64, *string, *string) (bool, error) { return true, nil }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodPost, "/api/v3/pet/1?name=rex&status=sold", map[string]string{"petId": "1"}, api.UpdatePetWithForm)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "UpdatePetWithForm missing pet -> 404",
			store: &mockStore{updatePetFields: func(context.Context, int64, *string, *string) (bool, error) { return false, nil }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodPost, "/api/v3/pet/9?name=rex", map[string]string{"petId": "9"}, api.UpdatePetWithForm)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "UpdatePetWithForm invalid input -> 400",
			store: &mockStore{updatePetFields: func(context.Context, int64, *string, *string) (bool, error) { return false, ErrInvalidInput }},
			invoke: func(api *PetAPI) testRequest {
				return paramRequest(http.MethodPost, "/api/v3/pet/1?status=bogus", map[string]string{"petId": "1"}, api.UpdatePetWithForm)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "UploadFile success",
			store: &mockStore{getPetByID: func(context.Context, int64) (Pet, error) { return validPet, nil }},
			invoke: func(api *PetAPI) testRequest {
				return rawRequest(http.MethodPost, "/api/v3/pet/1/uploadImage", "binary-data", map[string]string{"petId": "1"}, api.UploadFile)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "UploadFile missing pet -> 404",
			store: &mockStore{getPetByID: func(context.Context, int64) (Pet, error) { return Pet{}, ErrNotFound }},
			invoke: func(api *PetAPI) testRequest {
				return rawRequest(http.MethodPost, "/api/v3/pet/9/uploadImage", "data", map[string]string{"petId": "9"}, api.UploadFile)
			},
			wantCode: http.StatusNotFound,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			api := &PetAPI{Store: tc.store}
			rec := tc.invoke(api).run()
			if rec.Code != tc.wantCode {
				t.Fatalf("status = %d, want %d: %s", rec.Code, tc.wantCode, rec.Body.String())
			}
		})
	}
}

func TestStoreAPIHandlers(t *testing.T) {
	validOrder := Order{Id: 1, PetId: 1, Quantity: 1, Status: "placed"}

	tests := []struct {
		name     string
		store    *mockStore
		invoke   func(*StoreAPI) testRequest
		wantCode int
	}{
		{
			name:  "PlaceOrder success",
			store: &mockStore{createOrder: func(_ context.Context, o Order) (Order, error) { return o, nil }},
			invoke: func(api *StoreAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/store/order", validOrder, nil, api.PlaceOrder)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "PlaceOrder invalid status -> 400",
			store: &mockStore{},
			invoke: func(api *StoreAPI) testRequest {
				bad := Order{Id: 2, Status: "bogus"}
				return jsonRequest(http.MethodPost, "/api/v3/store/order", bad, nil, api.PlaceOrder)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "PlaceOrder malformed json -> 400",
			store: &mockStore{},
			invoke: func(api *StoreAPI) testRequest {
				return rawRequest(http.MethodPost, "/api/v3/store/order", `{`, nil, api.PlaceOrder)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "PlaceOrder store error -> 500",
			store: &mockStore{createOrder: func(context.Context, Order) (Order, error) { return Order{}, errBoom }},
			invoke: func(api *StoreAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/store/order", validOrder, nil, api.PlaceOrder)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "GetInventory success",
			store: &mockStore{inventory: func(context.Context) (map[string]int32, error) { return map[string]int32{"available": 2}, nil }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/store/inventory", nil, api.GetInventory)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "GetInventory store error -> 500",
			store: &mockStore{inventory: func(context.Context) (map[string]int32, error) { return nil, errBoom }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/store/inventory", nil, api.GetInventory)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "GetOrderById success",
			store: &mockStore{getOrderByID: func(context.Context, int64) (Order, error) { return validOrder, nil }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/store/order/1", map[string]string{"orderId": "1"}, api.GetOrderById)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "GetOrderById not found -> 404",
			store: &mockStore{getOrderByID: func(context.Context, int64) (Order, error) { return Order{}, ErrNotFound }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/store/order/9", map[string]string{"orderId": "9"}, api.GetOrderById)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "GetOrderById invalid id -> 400",
			store: &mockStore{},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/store/order/x", map[string]string{"orderId": "x"}, api.GetOrderById)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "DeleteOrder success",
			store: &mockStore{deleteOrder: func(context.Context, int64) (bool, error) { return true, nil }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/store/order/1", map[string]string{"orderId": "1"}, api.DeleteOrder)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "DeleteOrder missing -> 404",
			store: &mockStore{deleteOrder: func(context.Context, int64) (bool, error) { return false, nil }},
			invoke: func(api *StoreAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/store/order/9", map[string]string{"orderId": "9"}, api.DeleteOrder)
			},
			wantCode: http.StatusNotFound,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			api := &StoreAPI{Store: tc.store}
			rec := tc.invoke(api).run()
			if rec.Code != tc.wantCode {
				t.Fatalf("status = %d, want %d: %s", rec.Code, tc.wantCode, rec.Body.String())
			}
		})
	}
}

func TestUserAPIHandlers(t *testing.T) {
	validUser := User{Id: 1, Username: "user1", Password: "secret"}

	tests := []struct {
		name     string
		store    *mockStore
		invoke   func(*UserAPI) testRequest
		wantCode int
	}{
		{
			name:  "CreateUser success",
			store: &mockStore{createUser: func(_ context.Context, u User) (User, error) { return u, nil }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/user", validUser, nil, api.CreateUser)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "CreateUser invalid input -> 400",
			store: &mockStore{createUser: func(context.Context, User) (User, error) { return User{}, ErrInvalidInput }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/user", User{}, nil, api.CreateUser)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "CreateUser malformed json -> 400",
			store: &mockStore{},
			invoke: func(api *UserAPI) testRequest {
				return rawRequest(http.MethodPost, "/api/v3/user", `{`, nil, api.CreateUser)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "CreateUsersWithListInput success",
			store: &mockStore{createUsers: func(_ context.Context, u []User) (User, error) { return u[len(u)-1], nil }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/user/createWithList", []User{validUser}, nil, api.CreateUsersWithListInput)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "CreateUsersWithListInput store error -> 500",
			store: &mockStore{createUsers: func(context.Context, []User) (User, error) { return User{}, errBoom }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPost, "/api/v3/user/createWithList", []User{validUser}, nil, api.CreateUsersWithListInput)
			},
			wantCode: http.StatusInternalServerError,
		},
		{
			name:  "GetUserByName success",
			store: &mockStore{getUserByName: func(context.Context, string) (User, error) { return validUser, nil }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/user/user1", map[string]string{"username": "user1"}, api.GetUserByName)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "GetUserByName not found -> 404",
			store: &mockStore{getUserByName: func(context.Context, string) (User, error) { return User{}, ErrNotFound }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/user/missing", map[string]string{"username": "missing"}, api.GetUserByName)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "LoginUser success",
			store: &mockStore{authenticateUser: func(context.Context, string, string) (bool, error) { return true, nil }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/user/login?username=user1&password=secret", nil, api.LoginUser)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "LoginUser missing credentials -> 400",
			store: &mockStore{},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/user/login?username=user1", nil, api.LoginUser)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "LoginUser bad credentials -> 400",
			store: &mockStore{authenticateUser: func(context.Context, string, string) (bool, error) { return false, nil }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodGet, "/api/v3/user/login?username=user1&password=wrong", nil, api.LoginUser)
			},
			wantCode: http.StatusBadRequest,
		},
		{
			name:  "UpdateUser success",
			store: &mockStore{updateUser: func(context.Context, string, User) (bool, error) { return true, nil }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPut, "/api/v3/user/user1", validUser, map[string]string{"username": "user1"}, api.UpdateUser)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "UpdateUser missing -> 404",
			store: &mockStore{updateUser: func(context.Context, string, User) (bool, error) { return false, nil }},
			invoke: func(api *UserAPI) testRequest {
				return jsonRequest(http.MethodPut, "/api/v3/user/missing", validUser, map[string]string{"username": "missing"}, api.UpdateUser)
			},
			wantCode: http.StatusNotFound,
		},
		{
			name:  "DeleteUser success",
			store: &mockStore{deleteUser: func(context.Context, string) (bool, error) { return true, nil }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/user/user1", map[string]string{"username": "user1"}, api.DeleteUser)
			},
			wantCode: http.StatusOK,
		},
		{
			name:  "DeleteUser missing -> 404",
			store: &mockStore{deleteUser: func(context.Context, string) (bool, error) { return false, nil }},
			invoke: func(api *UserAPI) testRequest {
				return paramRequest(http.MethodDelete, "/api/v3/user/missing", map[string]string{"username": "missing"}, api.DeleteUser)
			},
			wantCode: http.StatusNotFound,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			api := &UserAPI{Store: tc.store}
			rec := tc.invoke(api).run()
			if rec.Code != tc.wantCode {
				t.Fatalf("status = %d, want %d: %s", rec.Code, tc.wantCode, rec.Body.String())
			}
		})
	}
}

func TestLogoutUserAlwaysOK(t *testing.T) {
	api := &UserAPI{}
	rec := paramRequest(http.MethodGet, "/api/v3/user/logout", nil, api.LogoutUser).run()
	if rec.Code != http.StatusOK {
		t.Fatalf("LogoutUser status = %d, want %d", rec.Code, http.StatusOK)
	}
}
