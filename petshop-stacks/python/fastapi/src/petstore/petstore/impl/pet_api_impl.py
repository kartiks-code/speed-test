import json
from typing import List, Optional, Tuple, Union

from fastapi import HTTPException

from petstore.apis.pet_api_base import BasePetApi
from petstore.db import get_pool
from petstore.models.api_response import ApiResponse
from petstore.models.category import Category
from petstore.models.pet import Pet
from petstore.models.tag import Tag


def _to_bytes(body: Optional[Union[bytes, str, Tuple[str, bytes]]]) -> bytes:
    if body is None:
        return b""
    if isinstance(body, tuple):
        body = body[1]
    if isinstance(body, str):
        return body.encode("utf-8")
    return bytes(body)


def _row_to_pet(row) -> Pet:
    category = None
    if row["category"] is not None:
        # category is stored as TEXT containing JSON
        cat_data = row["category"]
        if isinstance(cat_data, str):
            cat_data = json.loads(cat_data)
        category = Category.from_dict(cat_data)

    tags = None
    if row["tags"] is not None:
        # tags is a JSON column, decoded to Python list by asyncpg codec
        tags_data = row["tags"]
        if isinstance(tags_data, str):
            tags_data = json.loads(tags_data)
        tags = [Tag.from_dict(t) for t in tags_data]

    photo_urls = row["photo_urls"]
    if isinstance(photo_urls, str):
        photo_urls = json.loads(photo_urls)

    return Pet(
        id=row["id"],
        name=row["name"],
        category=category,
        photo_urls=photo_urls,
        tags=tags,
        status=str(row["status"]) if row["status"] is not None else None,
    )


class PetApiImpl(BasePetApi):
    async def add_pet(self, pet: Pet) -> Pet:
        pool = await get_pool()
        async with pool.acquire() as conn:
            if pet.id is None:
                pet_id = await conn.fetchval(
                    "SELECT nextval('pet_id_seq')"
                )
            else:
                pet_id = pet.id

            category_json = (
                json.dumps(pet.category.to_dict()) if pet.category else None
            )
            tags_json = (
                json.dumps([t.to_dict() for t in pet.tags]) if pet.tags else None
            )

            row = await conn.fetchrow(
                """
                INSERT INTO pet (id, name, category, photo_urls, tags, status)
                VALUES ($1, $2, $3, $4::json, $5::json, $6::pet_status)
                ON CONFLICT (id) DO UPDATE
                    SET name       = EXCLUDED.name,
                        category   = EXCLUDED.category,
                        photo_urls = EXCLUDED.photo_urls,
                        tags       = EXCLUDED.tags,
                        status     = EXCLUDED.status
                RETURNING *
                """,
                pet_id,
                pet.name,
                category_json,
                json.dumps(pet.photo_urls),
                tags_json,
                pet.status,
            )
            return _row_to_pet(row)

    async def update_pet(self, pet: Pet) -> Pet:
        pool = await get_pool()
        async with pool.acquire() as conn:
            if pet.id is None:
                raise HTTPException(
                    status_code=400, detail="Pet ID is required for update"
                )

            exists = await conn.fetchval(
                "SELECT 1 FROM pet WHERE id = $1", pet.id
            )
            if not exists:
                raise HTTPException(status_code=404, detail="Pet not found")

            category_json = (
                json.dumps(pet.category.to_dict()) if pet.category else None
            )
            tags_json = (
                json.dumps([t.to_dict() for t in pet.tags]) if pet.tags else None
            )

            row = await conn.fetchrow(
                """
                UPDATE pet
                SET name       = $2,
                    category   = $3,
                    photo_urls = $4::json,
                    tags       = $5::json,
                    status     = $6::pet_status
                WHERE id = $1
                RETURNING *
                """,
                pet.id,
                pet.name,
                category_json,
                json.dumps(pet.photo_urls),
                tags_json,
                pet.status,
            )
            return _row_to_pet(row)

    async def get_pet_by_id(self, petId: int) -> Pet:
        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow("SELECT * FROM pet WHERE id = $1", petId)
            if row is None:
                raise HTTPException(status_code=404, detail="Pet not found")
            return _row_to_pet(row)

    async def find_pets_by_status(self, status: Optional[str]) -> List[Pet]:
        pool = await get_pool()
        async with pool.acquire() as conn:
            if status:
                rows = await conn.fetch(
                    "SELECT * FROM pet WHERE status = $1::pet_status", status
                )
            else:
                rows = await conn.fetch("SELECT * FROM pet")
        return [_row_to_pet(r) for r in rows]

    async def find_pets_by_tags(self, tags: Optional[List[str]]) -> List[Pet]:
        pool = await get_pool()
        async with pool.acquire() as conn:
            if not tags:
                rows = await conn.fetch("SELECT * FROM pet")
                return [_row_to_pet(r) for r in rows]

            rows = await conn.fetch(
                """
                SELECT * FROM pet
                WHERE tags IS NOT NULL
                  AND EXISTS (
                      SELECT 1
                      FROM json_array_elements(tags) AS t
                      WHERE t->>'name' = ANY($1)
                  )
                """,
                tags,
            )
        return [_row_to_pet(r) for r in rows]

    async def update_pet_with_form(
        self,
        petId: int,
        name: Optional[str],
        status: Optional[str],
    ) -> None:
        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow("SELECT * FROM pet WHERE id = $1", petId)
            if row is None:
                raise HTTPException(status_code=404, detail="Pet not found")

            new_name = name if name is not None else row["name"]
            current_status = (
                str(row["status"]) if row["status"] is not None else None
            )
            new_status = status if status is not None else current_status

            await conn.execute(
                "UPDATE pet SET name = $2, status = $3::pet_status WHERE id = $1",
                petId,
                new_name,
                new_status,
            )

    async def delete_pet(self, petId: int, api_key: Optional[str]) -> None:
        pool = await get_pool()
        async with pool.acquire() as conn:
            result = await conn.execute("DELETE FROM pet WHERE id = $1", petId)
            if result == "DELETE 0":
                raise HTTPException(status_code=404, detail="Pet not found")

    async def upload_file(
        self,
        petId: int,
        additional_metadata: Optional[str],
        body: Optional[Union[bytes, str, Tuple[str, bytes]]],
    ) -> ApiResponse:
        content = _to_bytes(body)
        pool = await get_pool()
        async with pool.acquire() as conn:
            exists = await conn.fetchval(
                "SELECT 1 FROM pet WHERE id = $1", petId
            )
            if not exists:
                raise HTTPException(status_code=404, detail="Pet not found")

            await conn.execute(
                """
                INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)
                VALUES (
                    nextval('pet_photo_id_seq'),
                    $1, $2, $3, $4
                )
                """,
                petId,
                "application/octet-stream",
                additional_metadata,
                content,
            )

        msg = f"File uploaded for pet {petId}, {len(content)} bytes"
        if additional_metadata:
            msg += f" ({additional_metadata})"
        return ApiResponse(code=200, type="application/octet-stream", message=msg)
