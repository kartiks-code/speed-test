package petstore

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
)

func requireStore(c *fiber.Ctx, store Store) bool {
	if store == nil {
		_ = c.Status(fiber.StatusInternalServerError).JSON(Error{Code: "store_not_configured", Message: "PostgreSQL store is not configured"})
		return false
	}
	return true
}

func bindJSON(c *fiber.Ctx, target any) bool {
	if err := c.BodyParser(target); err != nil {
		_ = writeError(c, fiber.StatusBadRequest, "invalid_request", err.Error())
		return false
	}
	return true
}

func parseInt64Param(c *fiber.Ctx, name string) (int64, bool) {
	raw := c.Params(name)
	id, err := strconv.ParseInt(raw, 10, 64)
	if err != nil {
		_ = writeError(c, fiber.StatusBadRequest, "invalid_id", fmt.Sprintf("%s must be an integer", name))
		return 0, false
	}
	return id, true
}

func queryList(c *fiber.Ctx, name string) []string {
	args := c.Request().URI().QueryArgs()
	var values []string
	args.VisitAll(func(key, val []byte) {
		if string(key) == name {
			values = append(values, string(val))
		}
	})
	return compactStrings(values)
}

func optionalQuery(c *fiber.Ctx, name string) *string {
	args := c.Request().URI().QueryArgs()
	if !args.Has(name) {
		return nil
	}
	v := string(args.Peek(name))
	return &v
}

func validateStatuses(statuses []string) bool {
	for _, status := range statuses {
		if !validPetStatus(status) {
			return false
		}
	}
	return true
}

func statusForError(err error) int {
	switch {
	case errors.Is(err, ErrNotFound):
		return fiber.StatusNotFound
	case errors.Is(err, ErrInvalidInput):
		return fiber.StatusBadRequest
	default:
		return fiber.StatusInternalServerError
	}
}

func handleStoreError(c *fiber.Ctx, err error) {
	_ = writeError(c, statusForError(err), "error", err.Error())
}

func writeError(c *fiber.Ctx, status int, code string, message string) error {
	return c.Status(status).JSON(Error{Code: code, Message: message})
}

func writeNotFound(c *fiber.Ctx, message string) error {
	return writeError(c, fiber.StatusNotFound, "not_found", message)
}

func uploadMessage(petID int64, metadata string, size int) string {
	parts := []string{fmt.Sprintf("petId: %d", petID), fmt.Sprintf("bytes: %d", size)}
	if metadata != "" {
		parts = append(parts, "additionalMetadata: "+metadata)
	}
	return strings.Join(parts, ", ")
}

func setLoginHeaders(c *fiber.Ctx) {
	c.Set("X-Rate-Limit", "5000")
	c.Set("X-Expires-After", time.Now().UTC().Add(time.Hour).Format(time.RFC3339))
}
