# coding: utf-8
"""Unit tests for the Store API (4 operations).

Store routes are unauthenticated.  The database is the in-memory fake from
``conftest.py``; no PostgreSQL is required.
"""

from fastapi.testclient import TestClient


# --------------------------------------------------------------------------- #
# get_inventory  (GET /store/inventory)
# --------------------------------------------------------------------------- #
def test_get_inventory_counts_by_status(client: TestClient, db):
    db.add_pet(id=1, name="A", status="available")
    db.add_pet(id=2, name="B", status="available")
    db.add_pet(id=3, name="C", status="sold")
    db.add_pet(id=4, name="D", status=None)  # excluded (status IS NULL)
    resp = client.get("/store/inventory")
    assert resp.status_code == 200
    assert resp.json() == {"available": 2, "sold": 1}


def test_get_inventory_empty(client: TestClient):
    resp = client.get("/store/inventory")
    assert resp.status_code == 200
    assert resp.json() == {}


# --------------------------------------------------------------------------- #
# place_order  (POST /store/order)
# --------------------------------------------------------------------------- #
def test_place_order_assigns_server_id(client: TestClient, db):
    resp = client.post(
        "/store/order",
        json={"petId": 5, "quantity": 2, "status": "placed",
              "complete": False},
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 1  # MAX(id)+1 on empty "order" table
    assert body["petId"] == 5
    assert body["quantity"] == 2
    assert body["status"] == "placed"
    assert body["complete"] is False
    assert db.orders[1]["pet_id"] == 5


def test_place_order_round_trips_ship_date(client: TestClient):
    resp = client.post(
        "/store/order",
        json={"id": 99, "petId": 1, "quantity": 1,
              "shipDate": "2024-01-01T10:00:00Z", "status": "approved",
              "complete": True},
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 99
    assert body["status"] == "approved"
    assert body["complete"] is True
    assert body["shipDate"].startswith("2024-01-01T10:00:00")


def test_place_order_invalid_status_enum_is_422(client: TestClient):
    resp = client.post(
        "/store/order",
        json={"petId": 1, "quantity": 1, "status": "teleported"},
    )
    assert resp.status_code == 422


def test_place_order_missing_body_is_400(client: TestClient):
    resp = client.post("/store/order")
    assert resp.status_code == 400
    assert "required" in resp.json()["detail"].lower()


# --------------------------------------------------------------------------- #
# get_order_by_id  (GET /store/order/{orderId})
# --------------------------------------------------------------------------- #
def test_get_order_by_id_success(client: TestClient, db):
    db.add_order(id=3, pet_id=7, quantity=4, status="delivered",
                 complete=True)
    resp = client.get("/store/order/3")
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == 3
    assert body["petId"] == 7
    assert body["quantity"] == 4
    assert body["status"] == "delivered"
    assert body["complete"] is True


def test_get_order_by_id_not_found_is_404(client: TestClient):
    resp = client.get("/store/order/999")
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Order not found"


# --------------------------------------------------------------------------- #
# delete_order  (DELETE /store/order/{orderId})
# --------------------------------------------------------------------------- #
def test_delete_order_success(client: TestClient, db):
    db.add_order(id=8, pet_id=1, quantity=1, status="placed")
    resp = client.delete("/store/order/8")
    assert resp.status_code == 200
    assert 8 not in db.orders


def test_delete_order_not_found_is_404(client: TestClient):
    resp = client.delete("/store/order/999")
    assert resp.status_code == 404
    assert resp.json()["detail"] == "Order not found"
