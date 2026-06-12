const { Pool } = require('pg');

const poolConfig = {
  host: process.env.POSTGRES_HOST || 'localhost',
  port: parseInt(process.env.POSTGRES_PORT || '5434', 10),
  database: process.env.POSTGRES_DB || 'nodejs-express',
  user: process.env.POSTGRES_USER || 'myuser',
  password: process.env.POSTGRES_PASSWORD || 'mypassword',
};

// Optional tuning knobs; unset keeps node-postgres defaults (max=10, idle 10s).
if (process.env.PG_POOL_MAX) {
  poolConfig.max = parseInt(process.env.PG_POOL_MAX, 10);
}
if (process.env.PG_POOL_IDLE_TIMEOUT_MS) {
  poolConfig.idleTimeoutMillis = parseInt(process.env.PG_POOL_IDLE_TIMEOUT_MS, 10);
}

const pool = new Pool(poolConfig);

const query = (text, params) => pool.query(text, params);

module.exports = { pool, query };
