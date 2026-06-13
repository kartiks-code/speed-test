package petstore

import (
	"fmt"

	"github.com/gofiber/fiber/v2"
)

type StoreAPI struct {
	Store Store
}

// Delete /api/v3/store/order/:orderId
// Delete purchase order by identifier.
func (api *StoreAPI) DeleteOrder(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "orderId")
	if !ok {
		return nil
	}
	deleted, err := api.Store.DeleteOrder(c.Context(), id)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	if !deleted {
		return writeNotFound(c, fmt.Sprintf("order %d not found", id))
	}
	return c.SendStatus(fiber.StatusOK)
}

// Get /api/v3/store/inventory
// Returns pet inventories by status.
func (api *StoreAPI) GetInventory(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	inventory, err := api.Store.Inventory(c.Context())
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(inventory)
}

// Get /api/v3/store/order/:orderId
// Find purchase order by identifier.
func (api *StoreAPI) GetOrderById(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	id, ok := parseInt64Param(c, "orderId")
	if !ok {
		return nil
	}
	order, err := api.Store.GetOrderByID(c.Context(), id)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(order)
}

// Post /api/v3/store/order
// Place an order for a pet.
func (api *StoreAPI) PlaceOrder(c *fiber.Ctx) error {
	if !requireStore(c, api.Store) {
		return nil
	}
	var order Order
	if !bindJSON(c, &order) {
		return nil
	}
	if !validOrderStatus(order.Status) {
		return writeError(c, fiber.StatusBadRequest, "invalid_status", "order status must be placed, approved, or delivered")
	}
	created, err := api.Store.CreateOrder(c.Context(), order)
	if err != nil {
		handleStoreError(c, err)
		return nil
	}
	return c.JSON(created)
}
