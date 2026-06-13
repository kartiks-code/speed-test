package petstore

import (
	"fmt"

	"github.com/gofiber/fiber/v2"
)

type UserAPI struct {
	Store Store
}

// Post /api/v3/user
// Create user.
func (api *UserAPI) CreateUser(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	var user User
	if !bindJSON(c, &user) {
		return nil
	}
	created, err := api.Store.CreateUser(c.Context(), user)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(created)
}

// Post /api/v3/user/createWithList
// Creates list of users with given input array.
func (api *UserAPI) CreateUsersWithListInput(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	var users []User
	if !bindJSON(c, &users) {
		return nil
	}
	created, err := api.Store.CreateUsers(c.Context(), users)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(created)
}

// Delete /api/v3/user/:username
// Delete user.
func (api *UserAPI) DeleteUser(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	username := c.Params("username")
	if username == "" {
		return writeError(c, fiber.StatusBadRequest, "invalid_username", "username is required")
	}
	deleted, err := api.Store.DeleteUser(c.Context(), username)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	if !deleted {
		return writeNotFound(c, fmt.Sprintf("user %s not found", username))
	}
	return c.SendStatus(fiber.StatusOK)
}

// Get /api/v3/user/:username
// Get user by user name.
func (api *UserAPI) GetUserByName(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	username := c.Params("username")
	if username == "" {
		return writeError(c, fiber.StatusBadRequest, "invalid_username", "username is required")
	}
	user, err := api.Store.GetUserByUsername(c.Context(), username)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(user)
}

// Get /api/v3/user/login
// Logs user into the system.
func (api *UserAPI) LoginUser(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	username := c.Query("username")
	password := c.Query("password")
	if username == "" || password == "" {
		return writeError(c, fiber.StatusBadRequest, "invalid_credentials", "username and password are required")
	}
	ok, err := api.Store.AuthenticateUser(c.Context(), username, password)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	if !ok {
		return writeError(c, fiber.StatusBadRequest, "invalid_credentials", "invalid username or password")
	}
	setLoginHeaders(c)
	return c.Status(fiber.StatusOK).SendString(fmt.Sprintf("logged in user session: %s", username))
}

// Get /api/v3/user/logout
// Logs out current logged in user session.
func (api *UserAPI) LogoutUser(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusOK)
}

// Put /api/v3/user/:username
// Update user.
func (api *UserAPI) UpdateUser(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	username := c.Params("username")
	if username == "" {
		return writeError(c, fiber.StatusBadRequest, "invalid_username", "username is required")
	}
	var user User
	if !bindJSON(c, &user) {
		return nil
	}
	updated, err := api.Store.UpdateUser(c.Context(), username, user)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	if !updated {
		return writeNotFound(c, fmt.Sprintf("user %s not found", username))
	}
	return c.SendStatus(fiber.StatusOK)
}
