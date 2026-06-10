from typing import Dict, Optional

from fastapi import HTTPException

from petstore.apis.store_api_base import BaseStoreApi
from petstore.db import get_pool
from petstore.models.order import Order


def _row_to_order(row) -> Order:
    return Order(
        id=row["id"],
        petId=row["pet_id"],
        quantity=row["quantity"],
        shipDate=row["ship_date"],
        status=str(row["status"]) if row["status"] is not None else None,
        complete=row["complete"],
    )


class StoreApiImpl(BaseStoreApi):
    async def get_inventory(self) -> Dict[str, int]:
        pool = await get_pool()
        async with pool.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT status::text, COUNT(*) AS cnt
                FROM pet
                WHERE status IS NOT NULL
                GROUP BY status
                """
            )
        return {row["status"]: row["cnt"] for row in rows}

    async def place_order(self, order: Optional[Order]) -> Order:
        if order is None:
            raise HTTPException(status_code=400, detail="Order body is required")

        pool = await get_pool()
        async with pool.acquire() as conn:
            if order.id is None:
                order_id = await conn.fetchval(
                    'SELECT COALESCE(MAX(id), 0) + 1 FROM "order"'
                )
            else:
                order_id = order.id

            row = await conn.fetchrow(
                """
                INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
                VALUES ($1, $2, $3, $4, $5::order_status, $6)
                ON CONFLICT (id) DO UPDATE
                    SET pet_id    = EXCLUDED.pet_id,
                        quantity  = EXCLUDED.quantity,
                        ship_date = EXCLUDED.ship_date,
                        status    = EXCLUDED.status,
                        complete  = EXCLUDED.complete
                RETURNING *
                """,
                order_id,
                order.pet_id,
                order.quantity,
                order.ship_date,
                order.status,
                order.complete,
            )
            return _row_to_order(row)

    async def get_order_by_id(self, orderId: int) -> Order:
        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow(
                'SELECT * FROM "order" WHERE id = $1', orderId
            )
            if row is None:
                raise HTTPException(status_code=404, detail="Order not found")
            return _row_to_order(row)

    async def delete_order(self, orderId: int) -> None:
        pool = await get_pool()
        async with pool.acquire() as conn:
            result = await conn.execute(
                'DELETE FROM "order" WHERE id = $1', orderId
            )
            if result == "DELETE 0":
                raise HTTPException(status_code=404, detail="Order not found")
