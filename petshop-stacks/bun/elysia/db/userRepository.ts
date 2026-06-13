import sql from './client';
import type { User } from '../models';

function rowToUser(row: Record<string, unknown>): User {
  return {
    id: row.id as number | undefined,
    username: row.username as string | undefined,
    firstName: row.first_name as string | undefined,
    lastName: row.last_name as string | undefined,
    email: row.email as string | undefined,
    password: row.password as string | undefined,
    phone: row.phone as string | undefined,
    userStatus: row.user_status as number | undefined,
  };
}

async function upsert(user: User): Promise<User> {
  const [{ id }] = await sql`SELECT nextval('user_id_seq') AS id`;
  const userId = user.id ?? Number(id);
  const rows = await sql`
    INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
    VALUES (
      ${userId},
      ${user.username ?? null},
      ${user.firstName ?? null},
      ${user.lastName ?? null},
      ${user.email ?? null},
      ${user.password ?? null},
      ${user.phone ?? null},
      ${user.userStatus ?? null}
    )
    ON CONFLICT (username) DO UPDATE SET
      id          = EXCLUDED.id,
      first_name  = EXCLUDED.first_name,
      last_name   = EXCLUDED.last_name,
      email       = EXCLUDED.email,
      password    = EXCLUDED.password,
      phone       = EXCLUDED.phone,
      user_status = EXCLUDED.user_status
    RETURNING *
  `;
  return rowToUser(rows[0]);
}

export async function createUser(user: User): Promise<User> {
  return upsert(user);
}

export async function createUsersWithList(users: User[]): Promise<User> {
  let last: User | null = null;
  for (const u of users) {
    if (u.username) {
      last = await upsert(u);
    }
  }
  if (!last) throw new Error('No valid users (all missing username)');
  return last;
}

export async function findByUsername(username: string): Promise<User | null> {
  const rows = await sql`
    SELECT * FROM "user" WHERE username = ${username}
  `;
  if (rows.length === 0) return null;
  return rowToUser(rows[0]);
}

export async function updateUser(username: string, user: User): Promise<boolean> {
  const rows = await sql`
    UPDATE "user"
    SET id          = ${user.id ?? null},
        first_name  = ${user.firstName ?? null},
        last_name   = ${user.lastName ?? null},
        email       = ${user.email ?? null},
        password    = ${user.password ?? null},
        phone       = ${user.phone ?? null},
        user_status = ${user.userStatus ?? null}
    WHERE username = ${username}
    RETURNING id
  `;
  return rows.length > 0;
}

export async function remove(username: string): Promise<boolean> {
  const result = await sql`DELETE FROM "user" WHERE username = ${username}`;
  return result.count > 0;
}

export async function login(username: string, password: string): Promise<User | null> {
  const rows = await sql`
    SELECT * FROM "user"
    WHERE username = ${username} AND password = ${password}
  `;
  if (rows.length === 0) return null;
  return rowToUser(rows[0]);
}
