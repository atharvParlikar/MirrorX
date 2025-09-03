import { Client } from "pg";

const client = new Client({
  host: "localhost",
  port: 5433,
  user: "ts_admin",
  password: "ultrasecret",
  database: "timeseriesdb",
});

await client.connect();

const schema = await Bun.file("./schema.sql").text();
await client.query(schema);

console.log("✅ TimescaleDB schema applied!");
await client.end();

