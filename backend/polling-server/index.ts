import { createClient } from "redis";
import type { MarkPriceWsMessage } from "./types";
import { getBuyPrice, getSellPrice } from "./utils";

const client = createClient();
await client.connect();

const baseEndpoint = "wss://fstream.binance.com";

const socket = new WebSocket(baseEndpoint + "/stream?streams=btcusdt@markPrice@1s");

function handleBinanceMessage(msg: Bun.BunMessageEvent<any>) {
  const parsedMsg = JSON.parse(msg.data);
  const data: MarkPriceWsMessage['data'] = parsedMsg.data;
  console.log(data);
  client.publish("priceUpdates", JSON.stringify(
    {
      buy: getBuyPrice(parseFloat(data.p)),
      sell: getSellPrice(parseFloat(data.p)),
      symbol: "BTC"
    }
  ));
}

socket.addEventListener("message", (msg) => {
  try {
    handleBinanceMessage(msg);
  } catch (err) {
    console.log("error parsing binance ws message");
    console.log(err);
  }
});

