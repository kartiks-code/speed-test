# coding: utf-8
"""Pytest fixtures for the FastAPI Petstore server.

These tests are pure unit tests: they exercise the FastAPI routes and the
hand-written implementation classes in ``petstore.petstore.impl`` without
requiring a live PostgreSQL instance.

The implementation modules call ``get_pool()`` (imported from ``petstore.db``)
and then use ``async with pool.acquire() as conn`` followed by asyncpg's
``fetchval`` / ``fetchrow`` / ``fetch`` / ``execute`` methods.  We replace that
pool with an in-memory fake (``FakeDB`` + ``FakeConnection``) that interprets
the small, fixed set of SQL statements the implementation issues.  The fake
mirrors the relevant PostgreSQL/asyncpg behavior:

* ``pet.category`` is a ``TEXT`` column holding a JSON *string*.
* ``pet.photo_urls`` / ``pet.tags`` are JSON columns decoded to Python objects
  by the asyncpg codec, so the fake stores them as decoded objects.
* ``pet.status`` / ``order.status`` are enum types returned as text.
* server-assigned ids use ``MAX(id) + 1``.

The fake ``get_pool`` is patched into each implementation module so no network
or database is ever touched.
"""

import json

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from petstore.main import app as application


IMPL_MODULES = (
    "petstore.petstore.impl.pet_api_impl",
    "petstore.petstore.impl.store_api_impl",
    "petstore.petstore.impl.user_api_impl",
)


class FakeDB:
    """In-memory stand-in for the three Petstore tables."""

    def __init__(self) -> None:
        self.pets: dict[int, dict] = {}
        self.orders: dict[int, dict] = {}
        self.users: dict[str, dict] = {}
        self.pet_photos: list[dict] = []

    # --- seeding helpers used by tests ---------------------------------
    def add_pet(
        self,
        id: int,
        name: str,
        category: dict | None = None,
        photo_urls: list | None = None,
        tags: list | None = None,
        status: str | None = None,
    ) -> dict:
        row = {
            "id": id,
            "name": name,
            # category is stored as a JSON *string* (TEXT column)
            "category": json.dumps(category) if category is not None else None,
            "photo_urls": photo_urls if photo_urls is not None else [],
            "tags": tags,
            "status": status,
        }
        self.pets[id] = row
        return row

    def add_order(
        self,
        id: int,
        pet_id: int | None = None,
        quantity: int | None = None,
        ship_date=None,
        status: str | None = None,
        complete: bool | None = None,
    ) -> dict:
        row = {
            "id": id,
            "pet_id": pet_id,
            "quantity": quantity,
            "ship_date": ship_date,
            "status": status,
            "complete": complete,
        }
        self.orders[id] = row
        return row

    def add_user(
        self,
        username: str,
        id: int | None = None,
        first_name: str | None = None,
        last_name: str | None = None,
        email: str | None = None,
        password: str | None = None,
        phone: str | None = None,
        user_status: int | None = None,
    ) -> dict:
        row = {
            "id": id,
            "username": username,
            "first_name": first_name,
            "last_name": last_name,
            "email": email,
            "password": password,
            "phone": phone,
            "user_status": user_status,
        }
        self.users[username] = row
        return row


def _norm(query: str) -> str:
    """Collapse whitespace so substring matching is stable."""
    return " ".join(query.split())


class FakeConnection:
    """Interprets the fixed set of SQL statements the impl layer issues."""

    def __init__(self, db: FakeDB) -> None:
        self.db = db

    # --- pet row helpers ----------------------------------------------
    def _store_pet(self, args) -> dict:
        pet_id, name, category_json, photo_urls_json, tags_json, status = args
        row = {
            "id": pet_id,
            "name": name,
            # TEXT column: stored verbatim as the JSON string
            "category": category_json,
            # JSON columns: codec decodes to Python objects on read
            "photo_urls": json.loads(photo_urls_json)
            if photo_urls_json is not None
            else None,
            "tags": json.loads(tags_json) if tags_json is not None else None,
            "status": status,
        }
        self.db.pets[pet_id] = row
        return dict(row)

    def _store_order(self, args) -> dict:
        order_id, pet_id, quantity, ship_date, status, complete = args
        row = {
            "id": order_id,
            "pet_id": pet_id,
            "quantity": quantity,
            "ship_date": ship_date,
            "status": status,
            "complete": complete,
        }
        self.db.orders[order_id] = row
        return dict(row)

    def _store_user(self, args) -> dict:
        (id, username, first_name, last_name, email, password, phone,
         user_status) = args
        row = {
            "id": id,
            "username": username,
            "first_name": first_name,
            "last_name": last_name,
            "email": email,
            "password": password,
            "phone": phone,
            "user_status": user_status,
        }
        self.db.users[username] = row
        return dict(row)

    # --- asyncpg-compatible surface -----------------------------------
    async def fetchval(self, query: str, *args):
        q = _norm(query)
        if "COALESCE(MAX(id), 0) + 1 FROM pet" in q:
            return (max(self.db.pets) if self.db.pets else 0) + 1
        if 'COALESCE(MAX(id), 0) + 1 FROM "order"' in q:
            return (max(self.db.orders) if self.db.orders else 0) + 1
        if "SELECT 1 FROM pet WHERE id = $1" in q:
            return 1 if args[0] in self.db.pets else None
        if 'SELECT 1 FROM "user" WHERE username = $1' in q:
            return 1 if args[0] in self.db.users else None
        raise NotImplementedError(f"fetchval: {q!r}")

    async def fetchrow(self, query: str, *args):
        q = _norm(query)
        if q.startswith("INSERT INTO pet"):
            return self._store_pet(args)
        if q.startswith("UPDATE pet") and "RETURNING *" in q:
            return self._store_pet(args)
        if q.startswith("SELECT * FROM pet WHERE id = $1"):
            row = self.db.pets.get(args[0])
            return dict(row) if row is not None else None
        if q.startswith('INSERT INTO "order"'):
            return self._store_order(args)
        if q.startswith('SELECT * FROM "order" WHERE id = $1'):
            row = self.db.orders.get(args[0])
            return dict(row) if row is not None else None
        if q.startswith('INSERT INTO "user"'):
            return self._store_user(args)
        if q.startswith('SELECT * FROM "user" WHERE username = $1'):
            row = self.db.users.get(args[0])
            return dict(row) if row is not None else None
        raise NotImplementedError(f"fetchrow: {q!r}")

    async def fetch(self, query: str, *args):
        q = _norm(query)
        # store inventory aggregation
        if "COUNT(*)" in q and "GROUP BY status" in q:
            counts: dict[str, int] = {}
            for p in self.db.pets.values():
                status = p["status"]
                if status is not None:
                    counts[status] = counts.get(status, 0) + 1
            return [{"status": s, "cnt": c} for s, c in counts.items()]
        # find pets by tag name
        if "json_array_elements(tags)" in q:
            wanted = set(args[0])
            result = []
            for p in self.db.pets.values():
                tags = p["tags"]
                if not tags:
                    continue
                names = {t.get("name") for t in tags}
                if names & wanted:
                    result.append(dict(p))
            return result
        if q.startswith("SELECT * FROM pet WHERE status = $1::pet_status"):
            return [dict(p) for p in self.db.pets.values()
                    if p["status"] == args[0]]
        if q.startswith("SELECT * FROM pet"):
            return [dict(p) for p in self.db.pets.values()]
        raise NotImplementedError(f"fetch: {q!r}")

    async def execute(self, query: str, *args) -> str:
        q = _norm(query)
        if q.startswith("INSERT INTO pet_photo"):
            pet_id, content_type, metadata, content = args
            self.db.pet_photos.append({
                "id": len(self.db.pet_photos) + 1,
                "pet_id": pet_id,
                "content_type": content_type,
                "metadata": metadata,
                "content": content,
            })
            return "INSERT 0 1"
        if q.startswith("UPDATE pet SET name = $2, status = $3::pet_status"):
            pet_id, name, status = args
            row = self.db.pets.get(pet_id)
            if row is None:
                return "UPDATE 0"
            row["name"] = name
            row["status"] = status
            return "UPDATE 1"
        if q.startswith("DELETE FROM pet WHERE id = $1"):
            existed = self.db.pets.pop(args[0], None) is not None
            return f"DELETE {1 if existed else 0}"
        if q.startswith('DELETE FROM "order" WHERE id = $1'):
            existed = self.db.orders.pop(args[0], None) is not None
            return f"DELETE {1 if existed else 0}"
        if q.startswith('UPDATE "user"'):
            self._store_user(
                (args[1], args[0], args[2], args[3], args[4], args[5],
                 args[6], args[7])
            )
            return "UPDATE 1"
        if q.startswith('DELETE FROM "user" WHERE username = $1'):
            existed = self.db.users.pop(args[0], None) is not None
            return f"DELETE {1 if existed else 0}"
        raise NotImplementedError(f"execute: {q!r}")


class _Acquire:
    """Async context manager returned by ``FakePool.acquire()``."""

    def __init__(self, conn: FakeConnection) -> None:
        self._conn = conn

    async def __aenter__(self) -> FakeConnection:
        return self._conn

    async def __aexit__(self, *exc) -> bool:
        return False


class FakePool:
    def __init__(self, conn: FakeConnection) -> None:
        self._conn = conn

    def acquire(self) -> _Acquire:
        return _Acquire(self._conn)


@pytest.fixture
def db() -> FakeDB:
    """A fresh in-memory database for each test."""
    return FakeDB()


@pytest.fixture(autouse=True)
def _override_pool(db: FakeDB, monkeypatch) -> FakeDB:
    """Patch ``get_pool`` in every impl module to use the in-memory fake."""
    pool = FakePool(FakeConnection(db))

    async def fake_get_pool() -> FakePool:
        return pool

    for module in IMPL_MODULES:
        monkeypatch.setattr(f"{module}.get_pool", fake_get_pool)
    return db


@pytest.fixture
def app() -> FastAPI:
    application.dependency_overrides = {}
    return application


@pytest.fixture
def client(app) -> TestClient:
    return TestClient(app)
