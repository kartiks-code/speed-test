import postgres from 'postgres';

function buildDsn(): string {
  if (process.env.DATABASE_URL) return process.env.DATABASE_URL;
  const host = process.env.POSTGRES_HOST || 'localhost';
  const port = process.env.POSTGRES_PORT || '5434';
  const user = process.env.POSTGRES_USER || 'myuser';
  const password = process.env.POSTGRES_PASSWORD || 'mypassword';
  const db = process.env.POSTGRES_DB || 'bun-elysia';
  return `postgres://${user}:${password}@${host}:${port}/${db}`;
}

const maxConnections = parseInt(process.env.PG_POOL_MAX || '10', 10);

const sql = postgres(buildDsn(), { max: maxConnections });
export default sql;
export { buildDsn };
