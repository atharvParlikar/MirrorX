import postgres from "postgres";

export const pg = postgres({
  host: 'localhost',
  port: 8812,
  database: 'qdb',
  user: 'admin',
  password: 'quest',
  ssl: false
});
