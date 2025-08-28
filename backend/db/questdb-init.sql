DROP TABLE IF EXISTS price_ticks;
DROP TABLE IF EXISTS candles_1m;
DROP TABLE IF EXISTS candles_3m;
DROP TABLE IF EXISTS candles_5m;

-- Raw tick data table
CREATE TABLE price_ticks (
    symbol SYMBOL capacity 256 CACHE,
    price DOUBLE,
    timestamp TIMESTAMP
) timestamp (timestamp) PARTITION BY DAY WAL;

-- 1-minute candlestick table
CREATE TABLE candles_1m (
    symbol SYMBOL capacity 256 CACHE,
    open_price DOUBLE,
    high_price DOUBLE,
    low_price DOUBLE,
    close_price DOUBLE,
    tick_count LONG,
    timestamp TIMESTAMP
) timestamp (timestamp) PARTITION BY DAY WAL;

-- 3-minute candlestick table
CREATE TABLE candles_3m (
    symbol SYMBOL capacity 256 CACHE,
    open_price DOUBLE,
    high_price DOUBLE,
    low_price DOUBLE,
    close_price DOUBLE,
    tick_count LONG,
    timestamp TIMESTAMP
) timestamp (timestamp) PARTITION BY DAY WAL;

-- 5-minute candlestick table
CREATE TABLE candles_5m (
    symbol SYMBOL capacity 256 CACHE,
    open_price DOUBLE,
    high_price DOUBLE,
    low_price DOUBLE,
    close_price DOUBLE,
    tick_count LONG,
    timestamp TIMESTAMP
) timestamp (timestamp) PARTITION BY DAY WAL;

