#!/usr/bin/env bash

TOKEN="atharvp"
URL="http://localhost:8000/order/open"

# Hardcoded test order
TYPE="buy"
QTY="1.5"
ASSET="BTCUSDT"
STOP_LOSS="28000"
TAKE_PROFIT="32000"
LEVERAGE="5"

BODY=$(jq -n \
  --arg type "$TYPE" \
  --arg qty "$QTY" \
  --arg asset "$ASSET" \
  --arg stop_loss "$STOP_LOSS" \
  --arg take_profit "$TAKE_PROFIT" \
  --arg leverage "$LEVERAGE" \
  '{
        type: $type,
        qty: $qty,
        asset: $asset,
        stop_loss: $stop_loss,
        take_profit: $take_profit,
        leverage: $leverage
    }')

echo ">>> Sending order: $BODY"

curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$BODY" | jq .
