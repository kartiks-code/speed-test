package petstore

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
)

func init() {
	gin.SetMode(gin.TestMode)
}

// newTestContext builds a Gin context backed by a recorder for a request to
// target. It is handy for exercising helpers that read from *gin.Context
// (query strings, params, body) without spinning up a router.
func newTestContext(method, target string, body string) (*gin.Context, *httptest.ResponseRecorder) {
	rec := httptest.NewRecorder()
	ctx, _ := gin.CreateTestContext(rec)
	ctx.Request = httptest.NewRequest(method, target, strings.NewReader(body))
	ctx.Request.Header.Set("Content-Type", "application/json")
	return ctx, rec
}

func TestParseInt64Param(t *testing.T) {
	tests := []struct {
		name    string
		raw     string
		wantID  int64
		wantOK  bool
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
			ctx, rec := newTestContext(http.MethodGet, "/", "")
			ctx.Params = gin.Params{{Key: "petId", Value: tc.raw}}

			id, ok := parseInt64Param(ctx, "petId")
			if ok != tc.wantOK {
				t.Fatalf("parseInt64Param ok = %v, want %v", ok, tc.wantOK)
			}
			if tc.wantOK {
				if id != tc.wantID {
					t.Fatalf("parseInt64Param id = %d, want %d", id, tc.wantID)
				}
				return
			}
			if rec.Code != tc.wantHTTP {
				t.Fatalf("status = %d, want %d", rec.Code, tc.wantHTTP)
			}
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
			ctx, _ := newTestContext(http.MethodGet, "/?"+tc.rawQuery, "")
			got := queryList(ctx, "status")
			if !equalStrings(got, tc.want) {
				t.Fatalf("queryList = %#v, want %#v", got, tc.want)
			}
		})
	}
}

func TestOptionalQuery(t *testing.T) {
	t.Run("present", func(t *testing.T) {
		ctx, _ := newTestContext(http.MethodGet, "/?name=fido", "")
		got := optionalQuery(ctx, "name")
		if got == nil || *got != "fido" {
			t.Fatalf("optionalQuery = %v, want pointer to %q", got, "fido")
		}
	})
	t.Run("present but empty", func(t *testing.T) {
		ctx, _ := newTestContext(http.MethodGet, "/?name=", "")
		got := optionalQuery(ctx, "name")
		if got == nil || *got != "" {
			t.Fatalf("optionalQuery = %v, want pointer to empty string", got)
		}
	})
	t.Run("absent", func(t *testing.T) {
		ctx, _ := newTestContext(http.MethodGet, "/", "")
		if got := optionalQuery(ctx, "name"); got != nil {
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
		ctx, rec := newTestContext(http.MethodGet, "/", "")
		if requireStore(ctx, nil) {
			t.Fatal("requireStore(nil) = true, want false")
		}
		if rec.Code != http.StatusInternalServerError {
			t.Fatalf("status = %d, want %d", rec.Code, http.StatusInternalServerError)
		}
	})
	t.Run("configured store passes", func(t *testing.T) {
		ctx, rec := newTestContext(http.MethodGet, "/", "")
		if !requireStore(ctx, newFakeStore()) {
			t.Fatal("requireStore(store) = false, want true")
		}
		if rec.Code != http.StatusOK {
			t.Fatalf("status = %d, want %d (untouched)", rec.Code, http.StatusOK)
		}
	})
}

func TestBindJSON(t *testing.T) {
	t.Run("valid", func(t *testing.T) {
		ctx, _ := newTestContext(http.MethodPost, "/", `{"name":"fido","photoUrls":["a"]}`)
		var pet Pet
		if !bindJSON(ctx, &pet) {
			t.Fatal("bindJSON = false, want true")
		}
		if pet.Name != "fido" {
			t.Fatalf("pet.Name = %q, want fido", pet.Name)
		}
	})
	t.Run("malformed json writes 400", func(t *testing.T) {
		ctx, rec := newTestContext(http.MethodPost, "/", `{"name":`)
		var pet Pet
		if bindJSON(ctx, &pet) {
			t.Fatal("bindJSON = true, want false for malformed body")
		}
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("status = %d, want %d", rec.Code, http.StatusBadRequest)
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
	ctx, rec := newTestContext(http.MethodGet, "/", "")
	handleStoreError(ctx, ErrInvalidInput)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusBadRequest)
	}
	if !strings.Contains(rec.Body.String(), ErrInvalidInput.Error()) {
		t.Fatalf("body = %q, want it to contain %q", rec.Body.String(), ErrInvalidInput.Error())
	}
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
	ctx, rec := newTestContext(http.MethodGet, "/", "")
	setLoginHeaders(ctx)
	if got := rec.Header().Get("X-Rate-Limit"); got != "5000" {
		t.Fatalf("X-Rate-Limit = %q, want 5000", got)
	}
	if got := rec.Header().Get("X-Expires-After"); got == "" {
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
