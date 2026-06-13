import sql from './client';
import type { Pet, ApiResponse } from '../models';

function rowToPet(row: Record<string, unknown>): Pet {
  return {
    id: row.id as number,
    name: row.name as string,
    category: row.category ? JSON.parse(row.category as string) : undefined,
    photoUrls: row.photo_urls ? JSON.parse(row.photo_urls as string) : [],
    tags: row.tags ? JSON.parse(row.tags as string) : undefined,
    status: (row.status as Pet['status']) || undefined,
  };
}

export async function add(pet: Pet): Promise<Pet> {
  const [{ id }] = await sql`SELECT nextval('pet_id_seq') AS id`;
  const petId = pet.id ?? Number(id);
  await sql`
    INSERT INTO pet (id, name, category, photo_urls, tags, status)
    VALUES (
      ${petId},
      ${pet.name},
      ${pet.category ? JSON.stringify(pet.category) : null},
      ${JSON.stringify(pet.photoUrls)}::json,
      ${pet.tags ? JSON.stringify(pet.tags) : null}::json,
      ${pet.status ?? null}::pet_status
    )
    ON CONFLICT (id) DO UPDATE SET
      name       = EXCLUDED.name,
      category   = EXCLUDED.category,
      photo_urls = EXCLUDED.photo_urls,
      tags       = EXCLUDED.tags,
      status     = EXCLUDED.status
  `;
  return { ...pet, id: petId };
}

export async function update(pet: Pet): Promise<Pet | null> {
  if (pet.id === undefined) return null;
  const rows = await sql`
    UPDATE pet
    SET name       = ${pet.name},
        category   = ${pet.category ? JSON.stringify(pet.category) : null},
        photo_urls = ${JSON.stringify(pet.photoUrls)}::json,
        tags       = ${pet.tags ? JSON.stringify(pet.tags) : null}::json,
        status     = ${pet.status ?? null}::pet_status
    WHERE id = ${pet.id}
    RETURNING id, name, category::text, photo_urls::text, tags::text, status::text
  `;
  if (rows.length === 0) return null;
  return rowToPet(rows[0]);
}

export async function findByStatus(status: string): Promise<Pet[]> {
  const rows = await sql`
    SELECT id, name, category::text, photo_urls::text, tags::text, status::text
    FROM pet
    WHERE status = ${status}::pet_status
  `;
  return rows.map(rowToPet);
}

export async function findByTags(tags: string[]): Promise<Pet[]> {
  if (!tags || tags.length === 0) {
    const rows = await sql`
      SELECT id, name, category::text, photo_urls::text, tags::text, status::text
      FROM pet
    `;
    return rows.map(rowToPet);
  }
  const rows = await sql`
    SELECT id, name, category::text, photo_urls::text, tags::text, status::text
    FROM pet
    WHERE tags IS NOT NULL
      AND EXISTS (
        SELECT 1
        FROM json_array_elements(tags) AS t
        WHERE t->>'name' = ANY(${tags})
      )
  `;
  return rows.map(rowToPet);
}

export async function findById(id: number): Promise<Pet | null> {
  const rows = await sql`
    SELECT id, name, category::text, photo_urls::text, tags::text, status::text
    FROM pet
    WHERE id = ${id}
  `;
  if (rows.length === 0) return null;
  return rowToPet(rows[0]);
}

export async function updateWithForm(
  id: number,
  name?: string,
  status?: string,
): Promise<boolean> {
  const existing = await findById(id);
  if (!existing) return false;
  const newName = name ?? existing.name;
  const newStatus = status ?? existing.status ?? null;
  await sql`
    UPDATE pet
    SET name   = ${newName},
        status = ${newStatus}::pet_status
    WHERE id = ${id}
  `;
  return true;
}

export async function remove(id: number): Promise<boolean> {
  const result = await sql`DELETE FROM pet WHERE id = ${id}`;
  return result.count > 0;
}

export async function addPhoto(
  petId: number,
  content: Buffer,
  contentType = 'application/octet-stream',
  metadata?: string,
): Promise<ApiResponse | null> {
  const exists = await sql`SELECT 1 FROM pet WHERE id = ${petId}`;
  if (exists.length === 0) return null;
  // postgres.js serialises Buffer as BYTEA automatically
  await sql`
    INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)
    VALUES (nextval('pet_photo_id_seq'), ${petId}, ${contentType}, ${metadata ?? null}, ${content})
  `;
  return {
    code: 200,
    type: contentType,
    message: `File uploaded for pet ${petId}, ${content.length} bytes`,
  };
}
