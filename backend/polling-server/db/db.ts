import { Pool } from "pg";

const pool = new Pool({
  host: "localhost",
  port: 5433,
  user: "ts_admin",
  password: "ultrasecret",
  database: "timeseriesdb",
});

// insert a price into ticks
export async function insertTick(symbol: string, price: number,) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO ticks (time, symbol, price)
      VALUES (NOW(), $1, $2)
      RETURNING *;
    `;
    const values = [symbol, price];
    const result = await client.query(query, values);
    return result.rows[0];
  } finally {
    client.release(); // important
  }
}
