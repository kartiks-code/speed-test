package petstore

import (
	"fmt"

	"github.com/gofiber/fiber/v2"
)

type PetAPI struct {
	Store Store
}

// Post /api/v3/pet
// Add a new pet to the store.
func (api *PetAPI) AddPet(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	var pet Pet
	if !bindJSON(c, &pet) {
		return nil
	}
	created, err := api.Store.CreatePet(c.Context(), pet)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(created)
}

// Delete /api/v3/pet/:petId
// Deletes a pet.
func (api *PetAPI) DeletePet(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "petId")
	if !ok {
		return nil
	}
	if _, err := api.Store.DeletePet(c.Context(), id); err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.SendStatus(fiber.StatusOK)
}

// Get /api/v3/pet/findByStatus
// Finds Pets by status.
func (api *PetAPI) FindPetsByStatus(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	statuses := queryList(c, "status")
	if len(statuses) == 0 {
		statuses = []string{"available"}
	}
	if !validateStatuses(statuses) {
		return writeError(c, fiber.StatusBadRequest, "invalid_status", "status must be available, pending, or sold")
	}
	pets, err := api.Store.FindPetsByStatus(c.Context(), statuses)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(pets)
}

// Get /api/v3/pet/findByTags
// Finds Pets by tags.
func (api *PetAPI) FindPetsByTags(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	pets, err := api.Store.FindPetsByTags(c.Context(), queryList(c, "tags"))
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(pets)
}

// Get /api/v3/pet/:petId
// Find pet by identifier.
func (api *PetAPI) GetPetById(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "petId")
	if !ok {
		return nil
	}
	pet, err := api.Store.GetPetByID(c.Context(), id)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(pet)
}

// Put /api/v3/pet
// Update an existing pet.
func (api *PetAPI) UpdatePet(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	var pet Pet
	if !bindJSON(c, &pet) {
		return nil
	}
	updated, err := api.Store.UpdatePet(c.Context(), pet)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(updated)
}

// Post /api/v3/pet/:petId
// Updates a pet in the store with form data.
func (api *PetAPI) UpdatePetWithForm(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "petId")
	if !ok {
		return nil
	}
	name := optionalQuery(c, "name")
	status := optionalQuery(c, "status")
	updated, err := api.Store.UpdatePetFields(c.Context(), id, name, status)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	if !updated {
		return writeNotFound(c, fmt.Sprintf("pet %d not found", id))
	}
	return c.SendStatus(fiber.StatusOK)
}

// Post /api/v3/pet/:petId/uploadImage
// Uploads an image.
func (api *PetAPI) UploadFile(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "petId")
	if !ok {
		return nil
	}
	if _, err := api.Store.GetPetByID(c.Context(), id); err != nil {
		handleStoreError(c, err)
		return nil
	}
	body := c.Body()
	metadata := c.Query("additionalMetadata")
	if err := api.Store.SavePetPhoto(c.Context(), id, body, "application/octet-stream", metadata); err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(ApiResponse{
		Code:    fiber.StatusOK,
		Type:    "upload",
		Message: uploadMessage(id, metadata, len(body)),
	})
}
