import { Sender } from "@questdb/nodejs-client";
import postgres from "postgres";

const sql = postgres({
  host: 'localhost',
  port: 8812,
  database: 'qdb',
  user: 'admin',
  password: 'quest',
  ssl: false
});

const sqlFile = await Bun.file("./questdb-init.sql").text();

await sql.unsafe(sqlFile);

console.log("timescaledb init successful!");

process.exit(0);
