package petstore

import (
	"database/sql"
	"errors"
	"fmt"
	"reflect"
	"testing"
	"time"
)

func TestCompactStrings(t *testing.T) {
	tests := []struct {
		name string
		in   []string
		want []string
	}{
		{name: "nil", in: nil, want: nil},
		{name: "trims and drops empties", in: []string{" a ", "", "  "}, want: []string{"a"}},
		{name: "splits on comma", in: []string{"a,b , c"}, want: []string{"a", "b", "c"}},
		{name: "mixed", in: []string{"a", "b,c", " "}, want: []string{"a", "b", "c"}},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if got := compactStrings(tc.in); !reflect.DeepEqual(got, tc.want) {
				t.Fatalf("compactStrings(%#v) = %#v, want %#v", tc.in, got, tc.want)
			}
		})
	}
}

func TestInClauseQuery(t *testing.T) {
	query, args := inClauseQuery("WHERE x IN (", []string{"a", "b", "c"}, ") ORDER BY id")
	wantQuery := "WHERE x IN ($1, $2, $3) ORDER BY id"
	if query != wantQuery {
		t.Fatalf("query = %q, want %q", query, wantQuery)
	}
	if !reflect.DeepEqual(args, []any{"a", "b", "c"}) {
		t.Fatalf("args = %#v, want [a b c]", args)
	}

	emptyQuery, emptyArgs := inClauseQuery("IN (", nil, ")")
	if emptyQuery != "IN ()" || len(emptyArgs) != 0 {
		t.Fatalf("empty inClauseQuery = %q args=%#v", emptyQuery, emptyArgs)
	}
}

func TestValidPetStatus(t *testing.T) {
	for _, s := range []string{"available", "pending", "sold"} {
		if !validPetStatus(s) {
			t.Errorf("validPetStatus(%q) = false, want true", s)
		}
	}
	for _, s := range []string{"", "Available", "bogus", "SOLD"} {
		if validPetStatus(s) {
			t.Errorf("validPetStatus(%q) = true, want false", s)
		}
	}
}

func TestValidOrderStatus(t *testing.T) {
	for _, s := range []string{"", "placed", "approved", "delivered"} {
		if !validOrderStatus(s) {
			t.Errorf("validOrderStatus(%q) = false, want true", s)
		}
	}
	for _, s := range []string{"shipped", "Placed", "bogus"} {
		if validOrderStatus(s) {
			t.Errorf("validOrderStatus(%q) = true, want false", s)
		}
	}
}

func TestValidatePetForSave(t *testing.T) {
	tests := []struct {
		name    string
		pet     Pet
		wantErr error
	}{
		{name: "valid no status", pet: Pet{Name: "fido", PhotoUrls: []string{"a"}}, wantErr: nil},
		{name: "valid with status", pet: Pet{Name: "fido", PhotoUrls: []string{"a"}, Status: "sold"}, wantErr: nil},
		{name: "missing name", pet: Pet{PhotoUrls: []string{"a"}}, wantErr: ErrInvalidInput},
		{name: "missing photo urls", pet: Pet{Name: "fido"}, wantErr: ErrInvalidInput},
		{name: "invalid status", pet: Pet{Name: "fido", PhotoUrls: []string{"a"}, Status: "bogus"}, wantErr: ErrInvalidInput},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if err := validatePetForSave(tc.pet); !errors.Is(err, tc.wantErr) {
				t.Fatalf("validatePetForSave = %v, want %v", err, tc.wantErr)
			}
		})
	}
}

func TestStoreError(t *testing.T) {
	if got := storeError(sql.ErrNoRows); !errors.Is(got, ErrNotFound) {
		t.Fatalf("storeError(sql.ErrNoRows) = %v, want ErrNotFound", got)
	}
	wrapped := fmt.Errorf("query: %w", sql.ErrNoRows)
	if got := storeError(wrapped); !errors.Is(got, ErrNotFound) {
		t.Fatalf("storeError(wrapped ErrNoRows) = %v, want ErrNotFound", got)
	}
	other := errors.New("connection reset")
	if got := storeError(other); !errors.Is(got, other) {
		t.Fatalf("storeError(other) = %v, want passthrough", got)
	}
}

func TestEnvDefault(t *testing.T) {
	t.Setenv("UNIT_TEST_ENV_KEY", "value")
	if got := envDefault("UNIT_TEST_ENV_KEY", "fallback"); got != "value" {
		t.Fatalf("envDefault set = %q, want value", got)
	}
	if got := envDefault("UNIT_TEST_ENV_UNSET_KEY", "fallback"); got != "fallback" {
		t.Fatalf("envDefault unset = %q, want fallback", got)
	}
}

func TestPostgresDSNFromEnv(t *testing.T) {
	for _, key := range []string{
		"DATABASE_URL", "POSTGRES_DSN", "POSTGRES_USER", "POSTGRES_PASSWORD",
		"POSTGRES_HOST", "POSTGRES_PORT", "POSTGRES_DB", "POSTGRES_SSLMODE",
	} {
		t.Setenv(key, "")
	}

	t.Run("DATABASE_URL takes precedence", func(t *testing.T) {
		t.Setenv("DATABASE_URL", "postgres://full/dsn")
		t.Setenv("POSTGRES_DSN", "postgres://other")
		dsn, err := postgresDSNFromEnv()
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if dsn != "postgres://full/dsn" {
			t.Fatalf("dsn = %q, want DATABASE_URL value", dsn)
		}
	})

	t.Run("POSTGRES_DSN used when DATABASE_URL unset", func(t *testing.T) {
		t.Setenv("DATABASE_URL", "")
		t.Setenv("POSTGRES_DSN", "postgres://from/dsn")
		dsn, err := postgresDSNFromEnv()
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if dsn != "postgres://from/dsn" {
			t.Fatalf("dsn = %q, want POSTGRES_DSN value", dsn)
		}
	})

	t.Run("missing password errors", func(t *testing.T) {
		t.Setenv("DATABASE_URL", "")
		t.Setenv("POSTGRES_DSN", "")
		t.Setenv("POSTGRES_PASSWORD", "")
		if _, err := postgresDSNFromEnv(); err == nil {
			t.Fatal("expected error when POSTGRES_PASSWORD missing")
		}
	})

	t.Run("assembles from parts with escaping", func(t *testing.T) {
		t.Setenv("DATABASE_URL", "")
		t.Setenv("POSTGRES_DSN", "")
		t.Setenv("POSTGRES_USER", "my user")
		t.Setenv("POSTGRES_PASSWORD", "p@ss/word")
		t.Setenv("POSTGRES_HOST", "db.local")
		t.Setenv("POSTGRES_PORT", "5434")
		t.Setenv("POSTGRES_DB", "go-fiber")
		t.Setenv("POSTGRES_SSLMODE", "require")
		dsn, err := postgresDSNFromEnv()
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		want := "postgres://my+user:p%40ss%2Fword@db.local:5434/go-fiber?sslmode=require"
		if dsn != want {
			t.Fatalf("dsn = %q, want %q", dsn, want)
		}
	})
}

// fakeRow implements the scanner interface so scanPet/scanOrder/scanUser can be
// tested without a live database.
type fakeRow struct {
	values []any
	err    error
}

func (r fakeRow) Scan(dest ...any) error {
	if r.err != nil {
		return r.err
	}
	if len(dest) != len(r.values) {
		return fmt.Errorf("scan: got %d dest, want %d", len(dest), len(r.values))
	}
	for i, d := range dest {
		reflect.ValueOf(d).Elem().Set(reflect.ValueOf(r.values[i]))
	}
	return nil
}

func TestScanPet(t *testing.T) {
	t.Run("full row", func(t *testing.T) {
		row := fakeRow{values: []any{
			int64(1),
			"fido",
			sql.NullString{String: `{"id":3,"name":"dogs"}`, Valid: true},
			[]byte(`["http://a","http://b"]`),
			[]byte(`[{"id":7,"name":"friendly"}]`),
			sql.NullString{String: "available", Valid: true},
		}}
		pet, err := scanPet(row)
		if err != nil {
			t.Fatalf("scanPet: %v", err)
		}
		if pet.Id != 1 || pet.Name != "fido" || pet.Status != "available" {
			t.Fatalf("scanPet scalar fields = %#v", pet)
		}
		if pet.Category.Name != "dogs" || pet.Category.Id != 3 {
			t.Fatalf("category = %#v", pet.Category)
		}
		if len(pet.PhotoUrls) != 2 || pet.PhotoUrls[0] != "http://a" {
			t.Fatalf("photoUrls = %#v", pet.PhotoUrls)
		}
		if len(pet.Tags) != 1 || pet.Tags[0].Name != "friendly" {
			t.Fatalf("tags = %#v", pet.Tags)
		}
	})

	t.Run("null and empty json columns", func(t *testing.T) {
		row := fakeRow{values: []any{
			int64(2),
			"rex",
			sql.NullString{Valid: false},
			[]byte(nil),
			[]byte(nil),
			sql.NullString{Valid: false},
		}}
		pet, err := scanPet(row)
		if err != nil {
			t.Fatalf("scanPet: %v", err)
		}
		if pet.Status != "" || pet.Category.Name != "" || len(pet.PhotoUrls) != 0 || len(pet.Tags) != 0 {
			t.Fatalf("expected zero-value optional fields, got %#v", pet)
		}
	})

	t.Run("no rows maps to ErrNotFound", func(t *testing.T) {
		if _, err := scanPet(fakeRow{err: sql.ErrNoRows}); !errors.Is(err, ErrNotFound) {
			t.Fatalf("scanPet error = %v, want ErrNotFound", err)
		}
	})
}

func TestScanOrder(t *testing.T) {
	when := time.Date(2026, 6, 9, 12, 0, 0, 0, time.UTC)
	row := fakeRow{values: []any{
		int64(5),
		int64(1),
		int32(3),
		sql.NullTime{Time: when, Valid: true},
		sql.NullString{String: "placed", Valid: true},
		sql.NullBool{Bool: true, Valid: true},
	}}
	order, err := scanOrder(row)
	if err != nil {
		t.Fatalf("scanOrder: %v", err)
	}
	if order.Id != 5 || order.PetId != 1 || order.Quantity != 3 || order.Status != "placed" || !order.Complete {
		t.Fatalf("order = %#v", order)
	}
	if !order.ShipDate.Equal(when) {
		t.Fatalf("shipDate = %v, want %v", order.ShipDate, when)
	}

	if _, err := scanOrder(fakeRow{err: sql.ErrNoRows}); !errors.Is(err, ErrNotFound) {
		t.Fatalf("scanOrder error = %v, want ErrNotFound", err)
	}
}

func TestScanUser(t *testing.T) {
	row := fakeRow{values: []any{
		sql.NullInt64{Int64: 9, Valid: true},
		sql.NullString{String: "user1", Valid: true},
		sql.NullString{String: "First", Valid: true},
		sql.NullString{String: "Last", Valid: true},
		sql.NullString{String: "a@b.test", Valid: true},
		sql.NullString{String: "secret", Valid: true},
		sql.NullString{String: "555", Valid: true},
		sql.NullInt64{Int64: 1, Valid: true},
	}}
	user, err := scanUser(row)
	if err != nil {
		t.Fatalf("scanUser: %v", err)
	}
	if user.Id != 9 || user.Username != "user1" || user.Email != "a@b.test" || user.UserStatus != 1 {
		t.Fatalf("user = %#v", user)
	}

	if _, err := scanUser(fakeRow{err: sql.ErrNoRows}); !errors.Is(err, ErrNotFound) {
		t.Fatalf("scanUser error = %v, want ErrNotFound", err)
	}
}

type fakeResult struct {
	affected int64
	err      error
}

func (r fakeResult) LastInsertId() (int64, error) { return 0, nil }
func (r fakeResult) RowsAffected() (int64, error)  { return r.affected, r.err }

func TestRowsAffected(t *testing.T) {
	tests := []struct {
		name      string
		result    sql.Result
		inErr     error
		want      bool
		wantError bool
	}{
		{name: "exec error", result: nil, inErr: errBoom, wantError: true},
		{name: "rows affected error", result: fakeResult{err: errBoom}, wantError: true},
		{name: "no rows", result: fakeResult{affected: 0}, want: false},
		{name: "one row", result: fakeResult{affected: 1}, want: true},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got, err := rowsAffected(tc.result, tc.inErr)
			if tc.wantError {
				if err == nil {
					t.Fatal("expected error, got nil")
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if got != tc.want {
				t.Fatalf("rowsAffected = %v, want %v", got, tc.want)
			}
		})
	}
}
