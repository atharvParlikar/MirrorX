import { createClient } from "redis";
import { broadcast, deserialize, serialize } from "./utils";
import type { LiquidationMessage, PriceUpdates } from "./types";

const client = await createClient().on("error", (err) => console.error("Redis client Error", err)).connect();

const user_to_ws = new Map<String, Bun.ServerWebSocket<string>>();
const ws_to_user = new Map<Bun.ServerWebSocket<string>, String>();

client.subscribe("liquidations", (message: string) => {
  const toLiquidate = deserialize<LiquidationMessage>(message);
  if (!toLiquidate) {
    console.error("could not parse liquidations message from redis pub/sub");
    return;
  }

  toLiquidate.positions.forEach((position) => {
    const ws = user_to_ws.get(position.user_id);
    const serialized = serialize({
      event: "force-liquidation",
      positionId: position.position_id
    });
    if (serialized) {
      ws?.send(serialized)
    }
  });
});

client.subscribe("priceUpdates", (message) => broadcast(ws_to_user.keys().toArray(), message));

Bun.serve({
  fetch(req, server) {
    if (server.upgrade(req)) {
      return;
    }
    return new Response("Upgrade failed", { status: 500 });
  },
  websocket: {
    message(ws: Bun.ServerWebSocket<string>, message: string) {
      //  TODO: proper authorization later
      if (message.startsWith("AUTH") && !ws_to_user.get(ws)) {
        const userId = message.split(" ")[1]?.trim()!;
        user_to_ws.set(userId, ws);
        ws_to_user.set(ws, userId);
      }
    }
  }
});
