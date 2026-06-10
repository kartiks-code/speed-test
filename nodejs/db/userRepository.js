const { query } = require('./pool');

const mapRow = (row) => ({
  id: row.id !== null ? Number(row.id) : undefined,
  username: row.username,
  firstName: row.first_name || undefined,
  lastName: row.last_name || undefined,
  email: row.email || undefined,
  password: row.password || undefined,
  phone: row.phone || undefined,
  userStatus: row.user_status !== null ? Number(row.user_status) : undefined,
});

const create = async (user) => {
  await query(
    `INSERT INTO "user" ("id", username, first_name, last_name, email, "password", phone, user_status)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
    [
      user.id != null ? user.id : null,
      user.username,
      user.firstName || null,
      user.lastName || null,
      user.email || null,
      user.password || null,
      user.phone || null,
      user.userStatus != null ? user.userStatus : null,
    ],
  );
  return user;
};

const findByUsername = async (username) => {
  const result = await query(
    `SELECT "id", username, first_name, last_name, email, "password", phone, user_status
     FROM "user" WHERE username = $1`,
    [username],
  );
  if (result.rows.length === 0) {
    const err = new Error('User not found');
    err.status = 404;
    throw err;
  }
  return mapRow(result.rows[0]);
};

const update = async (username, user) => {
  await query(
    `UPDATE "user" SET "id" = $1, first_name = $2, last_name = $3, email = $4,
     "password" = $5, phone = $6, user_status = $7 WHERE username = $8`,
    [
      user.id != null ? user.id : null,
      user.firstName || null,
      user.lastName || null,
      user.email || null,
      user.password || null,
      user.phone || null,
      user.userStatus != null ? user.userStatus : null,
      username,
    ],
  );
};

const deleteUser = async (username) => {
  await query('DELETE FROM "user" WHERE username = $1', [username]);
};

const authenticate = async (username, password) => {
  const result = await query(
    'SELECT 1 FROM "user" WHERE username = $1 AND "password" = $2',
    [username, password],
  );
  return result.rows.length > 0;
};

module.exports = { create, findByUsername, update, deleteUser, authenticate };
