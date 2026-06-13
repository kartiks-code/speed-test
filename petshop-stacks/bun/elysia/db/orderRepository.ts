import sql from './client';
import type { Order } from '../models';

function rowToOrder(row: Record<string, unknown>): Order {
  return {
    id: row.id as number,
    petId: row.pet_id as number | undefined,
    quantity: row.quantity as number | undefined,
    shipDate: row.ship_date ? String(row.ship_date) : undefined,
    status: (row.status as Order['status']) || undefined,
    complete: row.complete as boolean | undefined,
  };
}

export async function placeOrder(order: Order): Promise<Order> {
  const [{ id }] = await sql`SELECT nextval('order_id_seq') AS id`;
  const orderId = order.id ?? Number(id);
  const rows = await sql`
    INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
    VALUES (
      ${orderId},
      ${order.petId ?? null},
      ${order.quantity ?? null},
      ${order.shipDate ?? null},
      ${order.status ?? null}::order_status,
      ${order.complete ?? false}
    )
    ON CONFLICT (id) DO UPDATE SET
      pet_id    = EXCLUDED.pet_id,
      quantity  = EXCLUDED.quantity,
      ship_date = EXCLUDED.ship_date,
      status    = EXCLUDED.status,
      complete  = EXCLUDED.complete
    RETURNING id, pet_id, quantity, ship_date::text, status::text, complete
  `;
  return rowToOrder(rows[0]);
}

export async function findById(id: number): Promise<Order | null> {
  const rows = await sql`
    SELECT id, pet_id, quantity, ship_date::text, status::text, complete
    FROM "order"
    WHERE id = ${id}
  `;
  if (rows.length === 0) return null;
  return rowToOrder(rows[0]);
}

export async function remove(id: number): Promise<boolean> {
  const result = await sql`DELETE FROM "order" WHERE id = ${id}`;
  return result.count > 0;
}

export async function getInventory(): Promise<Record<string, number>> {
  const rows = await sql`
    SELECT status::text, COUNT(*)::int AS cnt
    FROM pet
    WHERE status IS NOT NULL
    GROUP BY status
  `;
  const inventory: Record<string, number> = {};
  for (const row of rows) {
    inventory[row.status as string] = row.cnt as number;
  }
  return inventory;
}
