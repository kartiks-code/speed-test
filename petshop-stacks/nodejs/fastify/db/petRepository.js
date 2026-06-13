'use strict';
const { query } = require('./pool');

const mapRow = (row) => ({
  id: row.id !== null ? Number(row.id) : undefined,
  name: row.name,
  category: row.category
    ? (typeof row.category === 'string' ? JSON.parse(row.category) : row.category)
    : undefined,
  photoUrls: row.photo_urls
    ? (typeof row.photo_urls === 'string' ? JSON.parse(row.photo_urls) : row.photo_urls)
    : [],
  tags: row.tags
    ? (typeof row.tags === 'string' ? JSON.parse(row.tags) : row.tags)
    : undefined,
  status: row.status || undefined,
});

const add = async (pet) => {
  let id;
  if (pet.id != null) {
    id = pet.id;
  } else {
    const { rows } = await query("SELECT nextval('pet_id_seq') AS id");
    id = Number(rows[0].id);
  }
  await query(
    `INSERT INTO pet ("id", "name", category, photo_urls, tags, status)
     VALUES ($1, $2, $3, cast($4 as json), cast($5 as json), cast($6 as pet_status))
     ON CONFLICT ("id") DO UPDATE SET "name"=EXCLUDED."name", category=EXCLUDED.category,
     photo_urls=EXCLUDED.photo_urls, tags=EXCLUDED.tags, status=EXCLUDED.status`,
    [
      id,
      pet.name,
      pet.category != null ? JSON.stringify(pet.category) : null,
      JSON.stringify(pet.photoUrls),
      pet.tags != null ? JSON.stringify(pet.tags) : null,
      pet.status || null,
    ],
  );
  return { ...pet, id };
};

const update = async (pet) => {
  if (pet.id == null) {
    const err = new Error('Pet ID is required for update');
    err.status = 400;
    throw err;
  }
  const result = await query(
    `UPDATE pet SET "name" = $1, category = $2,
     photo_urls = cast($3 as json), tags = cast($4 as json), status = cast($5 as pet_status)
     WHERE "id" = $6`,
    [
      pet.name,
      pet.category != null ? JSON.stringify(pet.category) : null,
      JSON.stringify(pet.photoUrls),
      pet.tags != null ? JSON.stringify(pet.tags) : null,
      pet.status || null,
      pet.id,
    ],
  );
  if (result.rowCount === 0) {
    const err = new Error('Pet not found');
    err.status = 404;
    throw err;
  }
  return pet;
};

const findById = async (petId) => {
  const result = await query(
    `SELECT "id", "name", category, photo_urls, tags, status::text
     FROM pet WHERE "id" = $1`,
    [petId],
  );
  if (result.rows.length === 0) {
    const err = new Error('Pet not found');
    err.status = 404;
    throw err;
  }
  return mapRow(result.rows[0]);
};

const deletePet = async (petId) => {
  await query('DELETE FROM pet WHERE "id" = $1', [petId]);
};

const findByStatus = async (status) => {
  const result = await query(
    `SELECT "id", "name", category, photo_urls, tags, status::text
     FROM pet WHERE status = cast($1 as pet_status)`,
    [status],
  );
  return result.rows.map(mapRow);
};

const findByTags = async (tags) => {
  if (!tags || tags.length === 0) return [];
  const conditions = tags.map((_, i) => `tags::jsonb @> cast($${i + 1} as jsonb)`).join(' OR ');
  const params = tags.map((t) => JSON.stringify([{ name: t }]));
  const result = await query(
    `SELECT "id", "name", category, photo_urls, tags, status::text
     FROM pet WHERE ${conditions}`,
    params,
  );
  return result.rows.map(mapRow);
};

const updateWithForm = async (petId, name, status) => {
  if (name == null && status == null) return;
  const sets = [];
  const params = [];
  if (name != null) {
    sets.push(`"name" = $${params.length + 1}`);
    params.push(name);
  }
  if (status != null) {
    sets.push(`status = cast($${params.length + 1} as pet_status)`);
    params.push(status);
  }
  params.push(petId);
  await query(`UPDATE pet SET ${sets.join(', ')} WHERE "id" = $${params.length}`, params);
};

const addPhoto = async (petId, content, contentType, metadata) => {
  const exists = await query('SELECT 1 FROM pet WHERE "id" = $1', [petId]);
  if (exists.rows.length === 0) {
    const err = new Error('Pet not found');
    err.status = 404;
    throw err;
  }
  await query(
    `INSERT INTO pet_photo ("id", pet_id, content_type, metadata, content)
     VALUES (nextval('pet_photo_id_seq'), $1, $2, $3, $4)`,
    [petId, contentType, metadata, content],
  );
  return content ? content.length : 0;
};

const getInventory = async () => {
  const result = await query(
    "SELECT status::text, cast(COUNT(*) as int) as cnt FROM pet GROUP BY status",
    [],
  );
  const inventory = {};
  for (const row of result.rows) {
    if (row.status != null) inventory[row.status] = row.cnt;
  }
  return inventory;
};

module.exports = {
  add, update, findById, deletePet, findByStatus, findByTags, updateWithForm, addPhoto, getInventory,
};
