package petstore

import (
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
)

// ApiHandleFunctions groups the three API handler sets.
type ApiHandleFunctions struct {
	PetAPI   PetAPI
	StoreAPI StoreAPI
	UserAPI  UserAPI
}

// NewFiberApp creates and configures the Fiber application with all routes.
func NewFiberApp(routes ApiHandleFunctions) *fiber.App {
	app := fiber.New(fiber.Config{
		DisableStartupMessage: true,
	})

	app.Use(recover.New())
	if os.Getenv("FIBER_DISABLE_REQUEST_LOGGING") != "true" {
		app.Use(logger.New())
	}

	api := app.Group("/api/v3")

	// Static pet routes must come before parameterized ones.
	api.Get("/pet/findByStatus", routes.PetAPI.FindPetsByStatus)
	api.Get("/pet/findByTags", routes.PetAPI.FindPetsByTags)
	api.Post("/pet", routes.PetAPI.AddPet)
	api.Put("/pet", routes.PetAPI.UpdatePet)
	api.Post("/pet/:petId/uploadImage", routes.PetAPI.UploadFile)
	api.Get("/pet/:petId", routes.PetAPI.GetPetById)
	api.Post("/pet/:petId", routes.PetAPI.UpdatePetWithForm)
	api.Delete("/pet/:petId", routes.PetAPI.DeletePet)

	// Static store routes before parameterized ones.
	api.Get("/store/inventory", routes.StoreAPI.GetInventory)
	api.Post("/store/order", routes.StoreAPI.PlaceOrder)
	api.Get("/store/order/:orderId", routes.StoreAPI.GetOrderById)
	api.Delete("/store/order/:orderId", routes.StoreAPI.DeleteOrder)

	// Static user routes before parameterized ones.
	api.Post("/user/createWithList", routes.UserAPI.CreateUsersWithListInput)
	api.Get("/user/login", routes.UserAPI.LoginUser)
	api.Get("/user/logout", routes.UserAPI.LogoutUser)
	api.Post("/user", routes.UserAPI.CreateUser)
	api.Get("/user/:username", routes.UserAPI.GetUserByName)
	api.Put("/user/:username", routes.UserAPI.UpdateUser)
	api.Delete("/user/:username", routes.UserAPI.DeleteUser)

	return app
}
