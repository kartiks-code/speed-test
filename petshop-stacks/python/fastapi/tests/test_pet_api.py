# coding: utf-8
"""Unit tests for the Pet API (8 operations).

All pet routes are protected by the ``petstore_auth`` OAuth2 scheme, which
returns 403 when the ``Authorization`` header is missing, so every successful
request carries a bearer token.  The database is the in-memory fake from
``conftest.py``; no PostgreSQL is required.
"""

from fastapi.testclient import TestClient

AUTH = {"Authorization": "Bearer test-token"}


# --------------------------------------------------------------------------- #
# add_pet  (POST /pet)
# --------------------------------------------------------------------------- #
def test_add_pet_assigns_server_id_when_omitted(client: TestClient, db):
    resp = client.post(
        "/pet",
        headers=AUTH,
        json={"name": "Fido", "photoUrls": ["http://example.com/fido.jpg"]},
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 1  # first value from pet_id_seq
    assert body["name"] == "Fido"
    assert body["photoUrls"] == ["http://example.com/fido.jpg"]
    assert db.pets[1]["name"] == "Fido"


def test_add_pet_id_from_sequence(client: TestClient, db):
    db.add_pet(id=5, name="Rex", photo_urls=["http://x/rex.jpg"])
    resp = client.post(
        "/pet",
        headers=AUTH,
        json={"name": "New", "photoUrls": ["http://x/new.jpg"]},
    )
    assert resp.status_code == 200
    # sequence is independent of existing rows; first call returns 1
    assert resp.json()["id"] == 1


def test_add_pet_round_trips_category_tags_and_status(client: TestClient, db):
    payload = {
        "id": 42,
        "name": "Whiskers",
        "category": {"id": 1, "name": "cats"},
        "photoUrls": ["http://x/1.jpg", "http://x/2.jpg"],
        "tags": [{"id": 7, "name": "cute"}, {"id": 8, "name": "fluffy"}],
        "status": "available",
    }
    resp = client.post("/pet", headers=AUTH, json=payload)
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 42
    assert body["category"] == {"id": 1, "name": "cats"}
    assert body["photoUrls"] == ["http://x/1.jpg", "http://x/2.jpg"]
    assert [t["name"] for t in body["tags"]] == ["cute", "fluffy"]
    assert body["status"] == "available"
    # category is persisted as a JSON *string* (TEXT column)
    assert isinstance(db.pets[42]["category"], str)


def test_add_pet_missing_name_is_422(client: TestClient):
    resp = client.post(
        "/pet", headers=AUTH, json={"photoUrls": ["http://x/a.jpg"]}
    )
    assert resp.status_code == 422


def test_add_pet_invalid_status_enum_is_422(client: TestClient):
    resp = client.post(
        "/pet",
        headers=AUTH,
        json={"name": "Bad", "photoUrls": ["http://x/a.jpg"],
              "status": "flying"},
    )
    assert resp.status_code == 422


def test_add_pet_requires_authorization(client: TestClient):
    resp = client.post(
        "/pet", json={"name": "NoAuth", "photoUrls": ["http://x/a.jpg"]}
    )
    assert resp.status_code == 403


# --------------------------------------------------------------------------- #
# update_pet  (PUT /pet)
# --------------------------------------------------------------------------- #
def test_update_pet_success(client: TestClient, db):
    db.add_pet(id=10, name="Old", photo_urls=["http://x/old.jpg"],
               status="pending")
    resp = client.put(
        "/pet",
        headers=AUTH,
        json={"id": 10, "name": "Updated",
              "photoUrls": ["http://x/upd.jpg"], "status": "sold"},
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["name"] == "Updated"
    assert body["status"] == "sold"
    assert db.pets[10]["name"] == "Updated"


def test_update_pet_not_found_is_404(client: TestClient):
    resp = client.put(
        "/pet",
        headers=AUTH,
        json={"id": 999, "name": "Ghost", "photoUrls": ["http://x/g.jpg"]},
    )
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Pet not found"


def test_update_pet_without_id_is_400(client: TestClient):
    resp = client.put(
        "/pet",
        headers=AUTH,
        json={"name": "NoId", "photoUrls": ["http://x/n.jpg"]},
    )
    assert resp.status_code == 400
    assert "ID is required" in resp.json()["detail"]


# --------------------------------------------------------------------------- #
# find_pets_by_status  (GET /pet/findByStatus)
# --------------------------------------------------------------------------- #
def test_find_pets_by_status_filters(client: TestClient, db):
    db.add_pet(id=1, name="A", status="available")
    db.add_pet(id=2, name="B", status="sold")
    db.add_pet(id=3, name="C", status="sold")
    resp = client.get("/pet/findByStatus", headers=AUTH,
                      params={"status": "sold"})
    assert resp.status_code == 200
    ids = sorted(p["id"] for p in resp.json())
    assert ids == [2, 3]


def test_find_pets_by_status_defaults_to_available(client: TestClient, db):
    db.add_pet(id=1, name="A", status="available")
    db.add_pet(id=2, name="B", status="pending")
    resp = client.get("/pet/findByStatus", headers=AUTH)
    assert resp.status_code == 200
    body = resp.json()
    assert [p["id"] for p in body] == [1]


def test_find_pets_by_status_empty(client: TestClient):
    resp = client.get("/pet/findByStatus", headers=AUTH,
                      params={"status": "sold"})
    assert resp.status_code == 200
    assert resp.json() == []


# --------------------------------------------------------------------------- #
# find_pets_by_tags  (GET /pet/findByTags)
# --------------------------------------------------------------------------- #
def test_find_pets_by_tags_matches(client: TestClient, db):
    db.add_pet(id=1, name="A", tags=[{"id": 1, "name": "tag1"}])
    db.add_pet(id=2, name="B", tags=[{"id": 2, "name": "tag2"}])
    resp = client.get("/pet/findByTags", headers=AUTH,
                      params={"tags": "tag1"})
    assert resp.status_code == 200
    body = resp.json()
    assert [p["id"] for p in body] == [1]
    assert [t["name"] for t in body[0]["tags"]] == ["tag1"]


def test_find_pets_by_tags_no_match(client: TestClient, db):
    db.add_pet(id=1, name="A", tags=[{"id": 1, "name": "tag1"}])
    resp = client.get("/pet/findByTags", headers=AUTH,
                      params={"tags": "missing"})
    assert resp.status_code == 200
    assert resp.json() == []


# --------------------------------------------------------------------------- #
# get_pet_by_id  (GET /pet/{petId})
# --------------------------------------------------------------------------- #
def test_get_pet_by_id_success(client: TestClient, db):
    db.add_pet(id=15, name="Buddy",
               category={"id": 3, "name": "dogs"},
               photo_urls=["http://x/b.jpg"],
               tags=[{"id": 9, "name": "good-boy"}],
               status="available")
    resp = client.get("/pet/15", headers=AUTH)
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 15
    assert body["name"] == "Buddy"
    assert body["category"] == {"id": 3, "name": "dogs"}
    assert body["status"] == "available"


def test_get_pet_by_id_not_found_is_404(client: TestClient):
    resp = client.get("/pet/999", headers=AUTH)
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Pet not found"


# --------------------------------------------------------------------------- #
# update_pet_with_form  (POST /pet/{petId})
# --------------------------------------------------------------------------- #
def test_update_pet_with_form_success(client: TestClient, db):
    db.add_pet(id=7, name="Old", photo_urls=["http://x/o.jpg"],
               status="available")
    resp = client.post("/pet/7", headers=AUTH,
                       params={"name": "Renamed", "status": "sold"})
    assert resp.status_code == 200
    assert db.pets[7]["name"] == "Renamed"
    assert db.pets[7]["status"] == "sold"


def test_update_pet_with_form_partial_keeps_existing(client: TestClient, db):
    db.add_pet(id=8, name="Keep", photo_urls=["http://x/k.jpg"],
               status="available")
    resp = client.post("/pet/8", headers=AUTH, params={"status": "pending"})
    assert resp.status_code == 200
    assert db.pets[8]["name"] == "Keep"
    assert db.pets[8]["status"] == "pending"


def test_update_pet_with_form_not_found_is_404(client: TestClient):
    resp = client.post("/pet/999", headers=AUTH, params={"name": "X"})
    assert resp.status_code == 404


# --------------------------------------------------------------------------- #
# delete_pet  (DELETE /pet/{petId})
# --------------------------------------------------------------------------- #
def test_delete_pet_success(client: TestClient, db):
    db.add_pet(id=20, name="Goner", photo_urls=["http://x/g.jpg"])
    resp = client.delete("/pet/20", headers=AUTH)
    assert resp.status_code == 200
    assert 20 not in db.pets


def test_delete_pet_not_found_is_404(client: TestClient):
    resp = client.delete("/pet/999", headers=AUTH)
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Pet not found"


# --------------------------------------------------------------------------- #
# upload_file  (POST /pet/{petId}/uploadImage)
# --------------------------------------------------------------------------- #
def test_upload_file_success(client: TestClient, db):
    db.add_pet(id=30, name="Pic", photo_urls=["http://x/p.jpg"])
    resp = client.post(
        "/pet/30/uploadImage",
        headers=AUTH,
        params={"additionalMetadata": "vacation"},
        json="/path/to/file.jpg",
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["code"] == 200
    assert "File uploaded for pet 30" in body["message"]
    assert "vacation" in body["message"]
    # the binary content is persisted in the pet_photo table
    assert len(db.pet_photos) == 1
    photo = db.pet_photos[0]
    assert photo["pet_id"] == 30
    assert photo["metadata"] == "vacation"
    assert photo["content"] == b"/path/to/file.jpg"


def test_upload_file_pet_not_found_is_404(client: TestClient, db):
    resp = client.post("/pet/999/uploadImage", headers=AUTH,
                       json="/path/to/file.jpg")
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Pet not found"
    assert db.pet_photos == []
