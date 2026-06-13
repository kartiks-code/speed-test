package petstore

import (
	"fmt"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
)

func TestParseInt64Param(t *testing.T) {
	tests := []struct {
		name     string
		raw      string
		wantID   int64
		wantOK   bool
		wantHTTP int
	}{
		{name: "valid positive", raw: "42", wantID: 42, wantOK: true},
		{name: "valid negative", raw: "-7", wantID: -7, wantOK: true},
		{name: "zero", raw: "0", wantID: 0, wantOK: true},
		{name: "non numeric", raw: "abc", wantOK: false, wantHTTP: http.StatusBadRequest},
		{name: "empty", raw: "", wantOK: false, wantHTTP: http.StatusBadRequest},
		{name: "float", raw: "1.5", wantOK: false, wantHTTP: http.StatusBadRequest},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var gotID int64
			var gotOK bool

			app := fiber.New(fiber.Config{DisableStartupMessage: true})
			// Use an optional param so that /items/ (empty segment) still matches.
			app.Get("/items/:petId?", func(c *fiber.Ctx) error {
				gotID, gotOK = parseInt64Param(c, "petId")
				return nil
			})

			path := "/items/" + tc.raw
			resp := sendTo(app, http.MethodGet, path, "")

			if gotOK != tc.wantOK {
				t.Fatalf("parseInt64Param ok = %v, want %v", gotOK, tc.wantOK)
			}
			if tc.wantOK {
				if gotID != tc.wantID {
					t.Fatalf("parseInt64Param id = %d, want %d", gotID, tc.wantID)
				}
				return
			}
			if resp.StatusCode != tc.wantHTTP {
				t.Fatalf("status = %d, want %d", resp.StatusCode, tc.wantHTTP)
			}
			resp.Body.Close()
		})
	}
}

func TestQueryList(t *testing.T) {
	tests := []struct {
		name     string
		rawQuery string
		want     []string
	}{
		{name: "absent", rawQuery: "", want: nil},
		{name: "single", rawQuery: "status=available", want: []string{"available"}},
		{name: "repeated", rawQuery: "status=available&status=sold", want: []string{"available", "sold"}},
		{name: "comma separated", rawQuery: "status=available,pending", want: []string{"available", "pending"}},
		{name: "whitespace and empties trimmed", rawQuery: "status=%20available%20,,%20sold", want: []string{"available", "sold"}},
		{name: "empty value", rawQuery: "status=", want: nil},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var got []string
			app := fiber.New(fiber.Config{DisableStartupMessage: true})
			app.Get("/", func(c *fiber.Ctx) error {
				got = queryList(c, "status")
				return nil
			})
			path := "/"
			if tc.rawQuery != "" {
				path += "?" + tc.rawQuery
			}
			sendTo(app, http.MethodGet, path, "").Body.Close()
			if !equalStrings(got, tc.want) {
				t.Fatalf("queryList = %#v, want %#v", got, tc.want)
			}
		})
	}
}

func TestOptionalQuery(t *testing.T) {
	t.Run("present", func(t *testing.T) {
		var got *string
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Get("/", func(c *fiber.Ctx) error {
			got = optionalQuery(c, "name")
			return nil
		})
		sendTo(app, http.MethodGet, "/?name=fido", "").Body.Close()
		if got == nil || *got != "fido" {
			t.Fatalf("optionalQuery = %v, want pointer to %q", got, "fido")
		}
	})
	t.Run("present but empty", func(t *testing.T) {
		var got *string
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Get("/", func(c *fiber.Ctx) error {
			got = optionalQuery(c, "name")
			return nil
		})
		sendTo(app, http.MethodGet, "/?name=", "").Body.Close()
		if got == nil || *got != "" {
			t.Fatalf("optionalQuery = %v, want pointer to empty string", got)
		}
	})
	t.Run("absent", func(t *testing.T) {
		var got *string
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Get("/", func(c *fiber.Ctx) error {
			got = optionalQuery(c, "name")
			return nil
		})
		sendTo(app, http.MethodGet, "/", "").Body.Close()
		if got != nil {
			t.Fatalf("optionalQuery = %v, want nil", got)
		}
	})
}

func TestValidateStatuses(t *testing.T) {
	tests := []struct {
		name     string
		statuses []string
		want     bool
	}{
		{name: "empty is valid", statuses: nil, want: true},
		{name: "all valid", statuses: []string{"available", "pending", "sold"}, want: true},
		{name: "one invalid", statuses: []string{"available", "bogus"}, want: false},
		{name: "case sensitive", statuses: []string{"Available"}, want: false},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if got := validateStatuses(tc.statuses); got != tc.want {
				t.Fatalf("validateStatuses(%v) = %v, want %v", tc.statuses, got, tc.want)
			}
		})
	}
}

func TestRequireStore(t *testing.T) {
	t.Run("nil store writes 500", func(t *testing.T) {
		var ok bool
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Get("/", func(c *fiber.Ctx) error {
			ok = requireStore(c, nil)
			return nil
		})
		resp := sendTo(app, http.MethodGet, "/", "")
		resp.Body.Close()
		if ok {
			t.Fatal("requireStore(nil) = true, want false")
		}
		if resp.StatusCode != http.StatusInternalServerError {
			t.Fatalf("status = %d, want %d", resp.StatusCode, http.StatusInternalServerError)
		}
	})
	t.Run("configured store passes", func(t *testing.T) {
		var ok bool
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Get("/", func(c *fiber.Ctx) error {
			ok = requireStore(c, newFakeStore())
			return nil
		})
		resp := sendTo(app, http.MethodGet, "/", "")
		resp.Body.Close()
		if !ok {
			t.Fatal("requireStore(store) = false, want true")
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("status = %d, want %d (untouched)", resp.StatusCode, http.StatusOK)
		}
	})
}

func TestBindJSON(t *testing.T) {
	t.Run("valid", func(t *testing.T) {
		var pet Pet
		var ok bool
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Post("/", func(c *fiber.Ctx) error {
			ok = bindJSON(c, &pet)
			return nil
		})
		sendTo(app, http.MethodPost, "/", `{"name":"fido","photoUrls":["a"]}`).Body.Close()
		if !ok {
			t.Fatal("bindJSON = false, want true")
		}
		if pet.Name != "fido" {
			t.Fatalf("pet.Name = %q, want fido", pet.Name)
		}
	})
	t.Run("malformed json writes 400", func(t *testing.T) {
		var ok bool
		app := fiber.New(fiber.Config{DisableStartupMessage: true})
		app.Post("/", func(c *fiber.Ctx) error {
			var p Pet
			ok = bindJSON(c, &p)
			return nil
		})
		resp := sendTo(app, http.MethodPost, "/", `{"name":`)
		resp.Body.Close()
		if ok {
			t.Fatal("bindJSON = true, want false for malformed body")
		}
		if resp.StatusCode != http.StatusBadRequest {
			t.Fatalf("status = %d, want %d", resp.StatusCode, http.StatusBadRequest)
		}
	})
}

func TestStatusForErrorMappings(t *testing.T) {
	tests := []struct {
		name string
		err  error
		want int
	}{
		{name: "not found", err: ErrNotFound, want: http.StatusNotFound},
		{name: "invalid input", err: ErrInvalidInput, want: http.StatusBadRequest},
		{name: "wrapped not found", err: fmt.Errorf("lookup: %w", ErrNotFound), want: http.StatusNotFound},
		{name: "wrapped invalid input", err: fmt.Errorf("save: %w", ErrInvalidInput), want: http.StatusBadRequest},
		{name: "unknown", err: fmt.Errorf("boom"), want: http.StatusInternalServerError},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if got := statusForError(tc.err); got != tc.want {
				t.Fatalf("statusForError = %d, want %d", got, tc.want)
			}
		})
	}
}

func TestHandleStoreError(t *testing.T) {
	app := fiber.New(fiber.Config{DisableStartupMessage: true})
	app.Get("/", func(c *fiber.Ctx) error {
		handleStoreError(c, ErrInvalidInput)
		return nil
	})
	resp := sendTo(app, http.MethodGet, "/", "")
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d, want %d", resp.StatusCode, http.StatusBadRequest)
	}
	defer resp.Body.Close()
}

func TestUploadMessage(t *testing.T) {
	tests := []struct {
		name     string
		petID    int64
		metadata string
		size     int
		want     string
	}{
		{name: "with metadata", petID: 5, metadata: "note", size: 12, want: "petId: 5, bytes: 12, additionalMetadata: note"},
		{name: "without metadata", petID: 7, metadata: "", size: 0, want: "petId: 7, bytes: 0"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if got := uploadMessage(tc.petID, tc.metadata, tc.size); got != tc.want {
				t.Fatalf("uploadMessage = %q, want %q", got, tc.want)
			}
		})
	}
}

func TestSetLoginHeaders(t *testing.T) {
	var rateLimitHeader, expiresHeader string
	app := fiber.New(fiber.Config{DisableStartupMessage: true})
	app.Get("/", func(c *fiber.Ctx) error {
		setLoginHeaders(c)
		rateLimitHeader = c.GetRespHeader("X-Rate-Limit")
		expiresHeader = c.GetRespHeader("X-Expires-After")
		return nil
	})
	sendTo(app, http.MethodGet, "/", "").Body.Close()
	if rateLimitHeader != "5000" {
		t.Fatalf("X-Rate-Limit = %q, want 5000", rateLimitHeader)
	}
	if expiresHeader == "" {
		t.Fatal("X-Expires-After header missing")
	}
}

func equalStrings(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

