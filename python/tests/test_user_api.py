# coding: utf-8
"""Unit tests for the User API (7 operations).

User routes are unauthenticated.  The database is the in-memory fake from
``conftest.py``; no PostgreSQL is required.
"""

from fastapi.testclient import TestClient


# --------------------------------------------------------------------------- #
# create_user  (POST /user)
# --------------------------------------------------------------------------- #
def test_create_user_success(client: TestClient, db):
    resp = client.post(
        "/user",
        json={"id": 1, "username": "alice", "firstName": "Alice",
              "email": "alice@example.com", "userStatus": 1},
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["username"] == "alice"
    assert body["firstName"] == "Alice"
    assert body["userStatus"] == 1
    assert "alice" in db.users


def test_create_user_missing_body_is_400(client: TestClient):
    resp = client.post("/user")
    assert resp.status_code == 400
    assert "body is required" in resp.json()["detail"].lower()


def test_create_user_missing_username_is_400(client: TestClient):
    resp = client.post("/user", json={"firstName": "NoName"})
    assert resp.status_code == 400
    assert "Username is required" in resp.json()["detail"]


# --------------------------------------------------------------------------- #
# create_users_with_list_input  (POST /user/createWithList)
# --------------------------------------------------------------------------- #
def test_create_users_with_list_returns_last(client: TestClient, db):
    resp = client.post(
        "/user/createWithList",
        json=[{"username": "u1", "email": "u1@x.com"},
              {"username": "u2", "email": "u2@x.com"}],
    )
    assert resp.status_code == 200
    assert resp.json()["username"] == "u2"
    assert set(db.users) == {"u1", "u2"}


def test_create_users_with_list_empty_is_400(client: TestClient):
    resp = client.post("/user/createWithList", json=[])
    assert resp.status_code == 400
    assert "list is required" in resp.json()["detail"].lower()


def test_create_users_with_list_all_missing_username_is_400(client: TestClient):
    resp = client.post("/user/createWithList", json=[{"firstName": "x"}])
    assert resp.status_code == 400
    assert "No valid users" in resp.json()["detail"]


# --------------------------------------------------------------------------- #
# login_user  (GET /user/login)
# --------------------------------------------------------------------------- #
def test_login_user_success(client: TestClient, db):
    db.add_user(username="bob", password="secret")
    resp = client.get("/user/login",
                      params={"username": "bob", "password": "secret"})
    assert resp.status_code == 200
    assert resp.json().startswith("token:")


def test_login_user_wrong_password_is_400(client: TestClient, db):
    db.add_user(username="bob", password="secret")
    resp = client.get("/user/login",
                      params={"username": "bob", "password": "wrong"})
    assert resp.status_code == 400
    assert "Invalid username or password" in resp.json()["detail"]


def test_login_user_unknown_user_is_400(client: TestClient):
    resp = client.get("/user/login",
                      params={"username": "ghost", "password": "x"})
    assert resp.status_code == 400


def test_login_user_missing_params_is_400(client: TestClient):
    resp = client.get("/user/login")
    assert resp.status_code == 400
    assert "required" in resp.json()["detail"].lower()


# --------------------------------------------------------------------------- #
# logout_user  (GET /user/logout)
# --------------------------------------------------------------------------- #
def test_logout_user_is_noop(client: TestClient):
    resp = client.get("/user/logout")
    assert resp.status_code == 200
    assert resp.json() is None


# --------------------------------------------------------------------------- #
# get_user_by_name  (GET /user/{username})
# --------------------------------------------------------------------------- #
def test_get_user_by_name_success(client: TestClient, db):
    db.add_user(username="carol", id=5, first_name="Carol",
                email="carol@x.com", user_status=2)
    resp = client.get("/user/carol")
    assert resp.status_code == 200
    body = resp.json()
    assert body["username"] == "carol"
    assert body["firstName"] == "Carol"
    assert body["userStatus"] == 2


def test_get_user_by_name_not_found_is_404(client: TestClient):
    resp = client.get("/user/nobody")
    assert resp.status_code == 404
    assert resp.json()["detail"] == "User not found"


# --------------------------------------------------------------------------- #
# update_user  (PUT /user/{username})
# --------------------------------------------------------------------------- #
def test_update_user_success(client: TestClient, db):
    db.add_user(username="dave", email="old@x.com")
    resp = client.put(
        "/user/dave",
        json={"username": "dave", "email": "new@x.com", "phone": "555"},
    )
    assert resp.status_code == 200
    assert db.users["dave"]["email"] == "new@x.com"
    assert db.users["dave"]["phone"] == "555"


def test_update_user_not_found_is_404(client: TestClient):
    resp = client.put(
        "/user/ghost", json={"username": "ghost", "email": "g@x.com"}
    )
    assert resp.status_code == 404
    assert resp.json()["detail"] == "User not found"


def test_update_user_missing_body_is_400(client: TestClient, db):
    db.add_user(username="erin")
    resp = client.put("/user/erin")
    assert resp.status_code == 400
    assert "body is required" in resp.json()["detail"].lower()


# --------------------------------------------------------------------------- #
# delete_user  (DELETE /user/{username})
# --------------------------------------------------------------------------- #
def test_delete_user_success(client: TestClient, db):
    db.add_user(username="frank")
    resp = client.delete("/user/frank")
    assert resp.status_code == 200
    assert "frank" not in db.users


def test_delete_user_not_found_is_404(client: TestClient):
    resp = client.delete("/user/nobody")
    assert resp.status_code == 404
    assert resp.json()["detail"] == "User not found"
