package petstore

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestPetJSONRoundTrip(t *testing.T) {
	in := Pet{
		Id:        1,
		Name:      "fido",
		Category:  Category{Id: 3, Name: "dogs"},
		PhotoUrls: []string{"http://a", "http://b"},
		Tags:      []Tag{{Id: 7, Name: "friendly"}},
		Status:    "available",
	}
	data, err := json.Marshal(in)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}

	var out Pet
	if err := json.Unmarshal(data, &out); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if out.Name != in.Name || out.Status != in.Status || out.Category.Name != "dogs" {
		t.Fatalf("round-trip mismatch: %#v", out)
	}
	if len(out.PhotoUrls) != 2 || len(out.Tags) != 1 {
		t.Fatalf("collections lost in round-trip: %#v", out)
	}
}

func TestPetJSONFieldNames(t *testing.T) {
	data, err := json.Marshal(Pet{Name: "fido", PhotoUrls: []string{"a"}})
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	body := string(data)

	// photoUrls has no omitempty and must always be present.
	if !strings.Contains(body, `"photoUrls":["a"]`) {
		t.Fatalf("expected photoUrls present, got %s", body)
	}
	// status is omitempty: absent when empty.
	if strings.Contains(body, `"status"`) {
		t.Fatalf("expected status omitted when empty, got %s", body)
	}
	// Category is a non-pointer struct, so omitempty does NOT drop it; it
	// serializes as an (empty) object. This documents the generated behavior.
	if !strings.Contains(body, `"category":{}`) {
		t.Fatalf("expected empty category object, got %s", body)
	}
}

func TestPetUnmarshalEmptyCollections(t *testing.T) {
	var pet Pet
	if err := json.Unmarshal([]byte(`{"name":"x","photoUrls":[]}`), &pet); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if pet.PhotoUrls == nil {
		t.Fatal("expected non-nil empty PhotoUrls slice")
	}
	if len(pet.PhotoUrls) != 0 || len(pet.Tags) != 0 {
		t.Fatalf("expected empty collections, got %#v", pet)
	}
}

func TestPetStatusPreservedVerbatim(t *testing.T) {
	// JSON (de)serialization itself does not validate enum values; validation
	// happens in the store/handler layer. Any string should round-trip.
	for _, status := range []string{"available", "pending", "sold", "weird"} {
		var pet Pet
		if err := json.Unmarshal([]byte(`{"name":"x","photoUrls":["a"],"status":"`+status+`"}`), &pet); err != nil {
			t.Fatalf("unmarshal status %q: %v", status, err)
		}
		if pet.Status != status {
			t.Fatalf("status = %q, want %q", pet.Status, status)
		}
	}
}

func TestOrderJSONOmitsEmptyShipDate(t *testing.T) {
	data, err := json.Marshal(Order{Id: 1, PetId: 2})
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	// shipDate is omitempty but time.Time zero value is a struct, so it is NOT
	// omitted by encoding/json; it serializes as the zero timestamp.
	if !strings.Contains(string(data), `"shipDate"`) {
		t.Fatalf("expected shipDate present (zero time), got %s", string(data))
	}
}

func TestUserJSONRoundTrip(t *testing.T) {
	in := User{Id: 9, Username: "user1", FirstName: "First", LastName: "Last", Email: "a@b.test", Password: "secret", Phone: "555", UserStatus: 1}
	data, err := json.Marshal(in)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var out User
	if err := json.Unmarshal(data, &out); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if out != in {
		t.Fatalf("round-trip mismatch: got %#v, want %#v", out, in)
	}
}
