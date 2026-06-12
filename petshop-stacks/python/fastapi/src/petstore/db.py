import asyncio
import json
import os
from typing import Any

import asyncpg


_pool: asyncpg.Pool | None = None
_pool_lock = asyncio.Lock()


def _db_url() -> str:
    url = os.environ.get("DATABASE_URL")
    if url:
        return url
    host = os.environ.get("POSTGRES_HOST", os.environ.get("PGHOST", "localhost"))
    port = os.environ.get("POSTGRES_PORT", os.environ.get("PGPORT", "5434"))
    user = os.environ.get("POSTGRES_USER", os.environ.get("PGUSER", "myuser"))
    password = os.environ.get("POSTGRES_PASSWORD", os.environ.get("PGPASSWORD", "mypassword"))
    db = os.environ.get("POSTGRES_DB", os.environ.get("PGDATABASE", "python-fastapi"))
    return f"postgresql://{user}:{password}@{host}:{port}/{db}"


async def _init_connection(conn: asyncpg.Connection) -> None:
    await conn.set_type_codec(
        "json",
        encoder=json.dumps,
        decoder=json.loads,
        schema="pg_catalog",
    )
    await conn.set_type_codec(
        "jsonb",
        encoder=json.dumps,
        decoder=json.loads,
        schema="pg_catalog",
    )


async def get_pool() -> asyncpg.Pool:
    global _pool
    if _pool is not None:
        return _pool
    async with _pool_lock:
        if _pool is None:
            _pool = await asyncpg.create_pool(
                dsn=_db_url(),
                min_size=1,
                max_size=10,
                init=_init_connection,
            )
    return _pool


def row_to_dict(record: asyncpg.Record) -> dict[str, Any]:
    return dict(record)
