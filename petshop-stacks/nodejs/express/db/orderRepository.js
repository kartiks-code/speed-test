const { query } = require('./pool');

let idCounter = Date.now();
const nextId = () => ++idCounter;

const mapRow = (row) => ({
  id: row.id !== null ? Number(row.id) : undefined,
  petId: row.pet_id !== null ? Number(row.pet_id) : undefined,
  quantity: row.quantity !== null ? Number(row.quantity) : undefined,
  shipDate: row.ship_date ? row.ship_date.toISOString() : undefined,
  status: row.status || undefined,
  complete: row.complete !== null ? row.complete : undefined,
});

const place = async (order) => {
  const id = order.id != null ? order.id : nextId();
  await query(
    `INSERT INTO "order" ("id", pet_id, quantity, ship_date, status, complete)
     VALUES ($1, $2, $3, $4, cast($5 as order_status), $6)`,
    [
      id,
      order.petId != null ? order.petId : null,
      order.quantity != null ? order.quantity : null,
      order.shipDate ? new Date(order.shipDate) : null,
      order.status || null,
      order.complete != null ? order.complete : null,
    ],
  );
  return { ...order, id };
};

const findById = async (orderId) => {
  const result = await query(
    `SELECT "id", pet_id, quantity, ship_date, status::text, complete
     FROM "order" WHERE "id" = $1`,
    [orderId],
  );
  if (result.rows.length === 0) {
    const err = new Error('Order not found');
    err.status = 404;
    throw err;
  }
  return mapRow(result.rows[0]);
};

const deleteOrder = async (orderId) => {
  await query('DELETE FROM "order" WHERE "id" = $1', [orderId]);
};

module.exports = { place, findById, deleteOrder };
