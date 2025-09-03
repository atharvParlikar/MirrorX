-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Raw ticks table
CREATE TABLE IF NOT EXISTS ticks (
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL,
    price DOUBLE PRECISION NOT NULL
);

-- Convert to hypertable
SELECT create_hypertable('ticks', 'time', if_not_exists => TRUE);

-- 1m candles
CREATE MATERIALIZED VIEW IF NOT EXISTS candles_1m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close
FROM ticks
GROUP BY bucket, symbol
WITH NO DATA;

-- 3m candles
CREATE MATERIALIZED VIEW IF NOT EXISTS candles_3m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('3 minutes', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close
FROM ticks
GROUP BY bucket, symbol
WITH NO DATA;

-- 5m candles
CREATE MATERIALIZED VIEW IF NOT EXISTS candles_5m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close
FROM ticks
GROUP BY bucket, symbol
WITH NO DATA;

-- 10m candles
CREATE MATERIALIZED VIEW IF NOT EXISTS candles_10m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('10 minutes', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close
FROM ticks
GROUP BY bucket, symbol
WITH NO DATA;

-- Policy for 1m candles
DO $$
BEGIN
   IF NOT EXISTS (
      SELECT 1
      FROM timescaledb_information.jobs j
      JOIN timescaledb_information.continuous_aggregates ca
        ON j.hypertable_name = ca.view_name
      WHERE ca.view_name = 'candles_1m'
   ) THEN
      PERFORM add_continuous_aggregate_policy('candles_1m',
          start_offset => INTERVAL '1 hour',
          end_offset   => INTERVAL '1 minute',
          schedule_interval => INTERVAL '1 minute');
   END IF;
END$$;

-- Policy for 3m candles
DO $$
BEGIN
   IF NOT EXISTS (
      SELECT 1
      FROM timescaledb_information.jobs j
      JOIN timescaledb_information.continuous_aggregates ca
        ON j.hypertable_name = ca.view_name
      WHERE ca.view_name = 'candles_3m'
   ) THEN
      PERFORM add_continuous_aggregate_policy('candles_3m',
          start_offset => INTERVAL '3 hours',
          end_offset   => INTERVAL '3 minutes',
          schedule_interval => INTERVAL '3 minutes');
   END IF;
END$$;

-- Policy for 5m candles
DO $$
BEGIN
   IF NOT EXISTS (
      SELECT 1
      FROM timescaledb_information.jobs j
      JOIN timescaledb_information.continuous_aggregates ca
        ON j.hypertable_name = ca.view_name
      WHERE ca.view_name = 'candles_5m'
   ) THEN
      PERFORM add_continuous_aggregate_policy('candles_5m',
          start_offset => INTERVAL '6 hours',
          end_offset   => INTERVAL '5 minutes',
          schedule_interval => INTERVAL '5 minutes');
   END IF;
END$$;

-- Policy for 10m candles
DO $$
BEGIN
   IF NOT EXISTS (
      SELECT 1
      FROM timescaledb_information.jobs j
      JOIN timescaledb_information.continuous_aggregates ca
        ON j.hypertable_name = ca.view_name
      WHERE ca.view_name = 'candles_10m'
   ) THEN
      PERFORM add_continuous_aggregate_policy('candles_10m',
          start_offset => INTERVAL '12 hours',
          end_offset   => INTERVAL '10 minutes',
          schedule_interval => INTERVAL '10 minutes');
   END IF;
END$$;
