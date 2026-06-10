import secrets
from typing import List, Optional

from fastapi import HTTPException

from petstore.apis.user_api_base import BaseUserApi
from petstore.db import get_pool
from petstore.models.user import User


def _row_to_user(row) -> User:
    return User(
        id=row["id"],
        username=row["username"],
        firstName=row["first_name"],
        lastName=row["last_name"],
        email=row["email"],
        password=row["password"],
        phone=row["phone"],
        userStatus=row["user_status"],
    )


async def _upsert_user(conn, user: User) -> User:
    row = await conn.fetchrow(
        """
        INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (username) DO UPDATE
            SET id          = EXCLUDED.id,
                first_name  = EXCLUDED.first_name,
                last_name   = EXCLUDED.last_name,
                email       = EXCLUDED.email,
                password    = EXCLUDED.password,
                phone       = EXCLUDED.phone,
                user_status = EXCLUDED.user_status
        RETURNING *
        """,
        user.id,
        user.username,
        user.first_name,
        user.last_name,
        user.email,
        user.password,
        user.phone,
        user.user_status,
    )
    return _row_to_user(row)


class UserApiImpl(BaseUserApi):
    async def create_user(self, user: Optional[User]) -> User:
        if user is None:
            raise HTTPException(status_code=400, detail="User body is required")
        if not user.username:
            raise HTTPException(status_code=400, detail="Username is required")

        pool = await get_pool()
        async with pool.acquire() as conn:
            return await _upsert_user(conn, user)

    async def create_users_with_list_input(
        self, user: Optional[List[User]]
    ) -> User:
        if not user:
            raise HTTPException(status_code=400, detail="User list is required")

        pool = await get_pool()
        async with pool.acquire() as conn:
            last = None
            for u in user:
                if u.username:
                    last = await _upsert_user(conn, u)
            if last is None:
                raise HTTPException(
                    status_code=400,
                    detail="No valid users (all missing username)",
                )
            return last

    async def login_user(
        self, username: Optional[str], password: Optional[str]
    ) -> str:
        if not username or not password:
            raise HTTPException(
                status_code=400, detail="Username and password are required"
            )

        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow(
                'SELECT * FROM "user" WHERE username = $1', username
            )

        if row is None or row["password"] != password:
            raise HTTPException(
                status_code=400, detail="Invalid username or password"
            )

        return f"token:{secrets.token_hex(16)}"

    async def logout_user(self) -> None:
        # No server-side session to invalidate
        return None

    async def get_user_by_name(self, username: str) -> User:
        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow(
                'SELECT * FROM "user" WHERE username = $1', username
            )
        if row is None:
            raise HTTPException(status_code=404, detail="User not found")
        return _row_to_user(row)

    async def update_user(self, username: str, user: Optional[User]) -> None:
        if user is None:
            raise HTTPException(status_code=400, detail="User body is required")

        pool = await get_pool()
        async with pool.acquire() as conn:
            exists = await conn.fetchval(
                'SELECT 1 FROM "user" WHERE username = $1', username
            )
            if not exists:
                raise HTTPException(status_code=404, detail="User not found")

            await conn.execute(
                """
                UPDATE "user"
                SET id          = $2,
                    first_name  = $3,
                    last_name   = $4,
                    email       = $5,
                    password    = $6,
                    phone       = $7,
                    user_status = $8
                WHERE username = $1
                """,
                username,
                user.id,
                user.first_name,
                user.last_name,
                user.email,
                user.password,
                user.phone,
                user.user_status,
            )

    async def delete_user(self, username: str) -> None:
        pool = await get_pool()
        async with pool.acquire() as conn:
            result = await conn.execute(
                'DELETE FROM "user" WHERE username = $1', username
            )
            if result == "DELETE 0":
                raise HTTPException(status_code=404, detail="User not found")
