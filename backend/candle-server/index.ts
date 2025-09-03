import express from "express";
import { Pool } from "pg";

const pool = new Pool({
  host: "localhost",
  port: 5433,           // your TimescaleDB container
  user: "ts_admin",
  password: "ultrasecret",
  database: "timeseriesdb",
});

const app = express();
app.use(express.json());

// GET /api/v1/candles?asset=BTC&startTime=unix&endTime=unix&ts=1m/3m/5m/10m[&limit=]
app.get("/api/v1/candles", async (req, res) => {
  try {
    const { asset, startTime, endTime, ts } = req.query;
    const limit = Math.min(parseInt(req.query.limit as string ?? "500", 10), 5000);

    if (!asset || !startTime || !endTime || !ts) {
      return res.status(400).json({ error: "asset,startTime,endTime,ts are required" });
    }

    const start = Number(startTime);
    const end = Number(endTime);
    if (!Number.isFinite(start) || !Number.isFinite(end)) {
      return res.status(400).json({ error: "startTime/endTime must be unix seconds" });
    }
    if (start > end) {
      return res.status(400).json({ error: "startTime must be <= endTime" });
    }

    // whitelist the view name so we don't string-inject arbitrary SQL
    const view = (() => {
      switch (ts) {
        case "1m": return "candles_1m";
        case "3m": return "candles_3m";
        case "5m": return "candles_5m";
        case "10m": return "candles_10m";
        default: return null;
      }
    })();
    if (!view) return res.status(400).json({ error: "invalid ts (use 1m,3m,5m,10m)" });

    // Cast numerics to float8 so pg returns JS numbers (not strings)
    const sql = `
      SELECT
        bucket::timestamptz    AS bucket,
        symbol,
        open::float8           AS open,
        high::float8           AS high,
        low::float8            AS low,
        close::float8          AS close
      FROM ${view}
      WHERE symbol = $1
        AND bucket BETWEEN to_timestamp($2) AND to_timestamp($3)
      ORDER BY bucket ASC
      LIMIT $4
    `;

    const { rows } = await pool.query(sql, [asset, start, end, limit]);
    res.json(rows);
  } catch (err) {
    console.error("GET /api/v1/candles error:", err);
    res.status(500).json({ error: "internal error" });
  }
});

const port = process.env.PORT ?? 3000;
app.listen(port, () => console.log(`🚀 http://localhost:${port}`));
