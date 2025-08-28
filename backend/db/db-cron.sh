#!/bin/bash

# Configuration
QUESTDB_HOST="localhost:9000" # HTTP API endpoint (change if needed)
QUESTDB_USER="admin"          # QuestDB user (for HTTP or psql)
QUESTDB_PASSWORD="quest"      # QuestDB password
LOG_DIR="/var/log/questdb"    # Directory for logs
SQL_DIR="/tmp/questdb_sql"    # Directory for SQL files

# Ensure directories exist
mkdir -p "$LOG_DIR"
mkdir -p "$SQL_DIR"

# SQL queries for candlestick updates
cat <<EOF >"$SQL_DIR/candles_1m.sql"
INSERT INTO candles_1m (symbol, open_price, high_price, low_price, close_price, tick_count, timestamp)
SELECT 
    symbol,
    first(price) as open_price,
    max(price) as high_price,
    min(price) as low_price,
    last(price) as close_price,
    count() as tick_count,
    to_timestamp(
        floor(extract(epoch from timestamp) / 60) * 60 * 1000000,
        'us'
    ) as timestamp
FROM price_ticks 
WHERE timestamp >= dateadd('m', -2, now())
GROUP BY symbol, floor(extract(epoch from timestamp) / 60)
ON CONFLICT(symbol, timestamp) DO UPDATE SET
    open_price = EXCLUDED.open_price,
    high_price = EXCLUDED.high_price,
    low_price = EXCLUDED.low_price,
    close_price = EXCLUDED.close_price,
    tick_count = EXCLUDED.tick_count;
EOF

cat <<EOF >"$SQL_DIR/candles_3m.sql"
INSERT INTO candles_3m (symbol, open_price, high_price, low_price, close_price, tick_count, timestamp)
SELECT
    symbol,
    first(price) as open_price,
    max(price) as high_price,
    min(price) as low_price,
    last(price) as close_price,
    count() as tick_count,
    to_timestamp(
        floor(extract(epoch from timestamp) / 180) * 180 * 1000000,
        'us'
    ) as timestamp
FROM price_ticks 
WHERE timestamp >= dateadd('m', -6, now())
GROUP BY symbol, floor(extract(epoch from timestamp) / 180)
ON CONFLICT(symbol, timestamp) DO UPDATE SET
    open_price = EXCLUDED.open_price,
    high_price = EXCLUDED.high_price,
    low_price = EXCLUDED.low_price,
    close_price = EXCLUDED.close_price,
    tick_count = EXCLUDED.tick_count;
EOF

cat <<EOF >"$SQL_DIR/candles_5m.sql"
INSERT INTO candles_5m (symbol, open_price, high_price, low_price, close_price, tick_count, timestamp)
SELECT 
    symbol,
    first(price) as open_price,
    max(price) as high_price,
    min(price) as low_price,
    last(price) as close_price,
    count() as tick_count,
    to_timestamp(
        floor(extract(epoch from timestamp) / 300) * 300 * 1000000,
        'us'
    ) as timestamp
FROM price_ticks 
WHERE timestamp >= dateadd('m', -10, now())
GROUP BY symbol, floor(extract(epoch from timestamp) / 300)
ON CONFLICT(symbol, timestamp) DO UPDATE SET
    open_price = EXCLUDED.open_price,
    high_price = EXCLUDED.high_price,
    low_price = EXCLUDED.low_price,
    close_price = EXCLUDED.close_price,
    tick_count = EXCLUDED.tick_count;
EOF

# Function to execute SQL via HTTP API
execute_sql() {
  local sql_file=$1
  local log_file=$2
  local query
  query=$(cat "$sql_file")

  # Execute query via curl (HTTP API)
  curl -G --user "$QUESTDB_USER:$QUESTDB_PASSWORD" \
    --data-urlencode "query=$query" \
    "http://$QUESTDB_HOST/exec" >>"$log_file" 2>&1

  if [ $? -eq 0 ]; then
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Successfully executed $sql_file" >>"$log_file"
  else
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Error executing $sql_file" >>"$log_file"
  fi
}

# Execute SQL files (for manual testing or immediate execution)
execute_sql "$SQL_DIR/candles_1m.sql" "$LOG_DIR/candles_1m.log"
execute_sql "$SQL_DIR/candles_3m.sql" "$LOG_DIR/candles_3m.log"
execute_sql "$SQL_DIR/candles_5m.sql" "$LOG_DIR/candles_5m.log"

# Set up cron jobs
CRON_FILE="/tmp/questdb_cron"
cat <<EOF >"$CRON_FILE"
# QuestDB candlestick updates
* * * * * /bin/bash -c "/bin/bash $0 execute_sql $SQL_DIR/candles_1m.sql $LOG_DIR/candles_1m.log"
*/3 * * * * /bin/bash -c "/bin/bash $0 execute_sql $SQL_DIR/candles_3m.sql $LOG_DIR/candles_3m.log"
*/5 * * * * /bin/bash -c "/bin/bash $0 execute_sql $SQL_DIR/candles_5m.sql $LOG_DIR/candles_5m.log"
EOF

# Install cron jobs
crontab "$CRON_FILE"
echo "Cron jobs installed. Check logs in $LOG_DIR for execution details."

# Clean up
rm "$CRON_FILE"
