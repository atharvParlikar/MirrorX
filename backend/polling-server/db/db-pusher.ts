import { createClient } from "redis";
import type { PriceUpdate } from "../types";
import { insertTick } from "./db.ts";

const client = createClient();
await client.connect();

async function pushUpdatesToDb(priceUpdate: PriceUpdate) {
  const price = (priceUpdate.buy + priceUpdate.sell) / 2;
  await insertTick("BTC", price);
  console.log("price inserted");
}

client.subscribe("priceUpdates", async (msg) => {
  const prices: PriceUpdate = JSON.parse(msg);
  await pushUpdatesToDb(prices);
});

