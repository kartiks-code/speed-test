package petstore

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
)

func requireStore(c *gin.Context, store Store) bool {
	if store == nil {
		writeError(c, http.StatusInternalServerError, "store_not_configured", "PostgreSQL store is not configured")
		return false
	}
	return true
}

func bindJSON(c *gin.Context, target any) bool {
	if err := c.ShouldBindJSON(target); err != nil {
		writeError(c, http.StatusBadRequest, "invalid_request", err.Error())
		return false
	}
	return true
}

func parseInt64Param(c *gin.Context, name string) (int64, bool) {
	raw := c.Param(name)
	id, err := strconv.ParseInt(raw, 10, 64)
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", fmt.Sprintf("%s must be an integer", name))
		return 0, false
	}
	return id, true
}

func queryList(c *gin.Context, name string) []string {
	values := c.QueryArray(name)
	if len(values) == 0 {
		value := c.Query(name)
		if value != "" {
			values = []string{value}
		}
	}
	return compactStrings(values)
}

func optionalQuery(c *gin.Context, name string) *string {
	value, ok := c.GetQuery(name)
	if !ok {
		return nil
	}
	return &value
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
		return http.StatusNotFound
	case errors.Is(err, ErrInvalidInput):
		return http.StatusBadRequest
	default:
		return http.StatusInternalServerError
	}
}

func handleStoreError(c *gin.Context, err error) {
	writeError(c, statusForError(err), "error", err.Error())
}

func writeError(c *gin.Context, status int, code string, message string) {
	c.JSON(status, Error{Code: code, Message: message})
}

func writeNotFound(c *gin.Context, message string) {
	writeError(c, http.StatusNotFound, "not_found", message)
}

func uploadMessage(petID int64, metadata string, size int) string {
	parts := []string{fmt.Sprintf("petId: %d", petID), fmt.Sprintf("bytes: %d", size)}
	if metadata != "" {
		parts = append(parts, "additionalMetadata: "+metadata)
	}
	return strings.Join(parts, ", ")
}

func readBody(c *gin.Context) ([]byte, bool) {
	body, err := io.ReadAll(c.Request.Body)
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_request", err.Error())
		return nil, false
	}
	return body, true
}

func setLoginHeaders(c *gin.Context) {
	c.Header("X-Rate-Limit", "5000")
	c.Header("X-Expires-After", time.Now().UTC().Add(time.Hour).Format(time.RFC3339))
}
