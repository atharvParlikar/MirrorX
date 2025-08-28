import { Sender } from "@questdb/nodejs-client";

export async function sendPrice(price: number) {
  // Connect to QuestDB ILP (default port 9009, NOT 9000)
  const sender = await Sender.fromConfig("http::addr=localhost:9000;");

  await sender
    .table("price_ticks")
    .symbol("symbol", "BTC")
    .floatColumn("price", price)
    .atNow(); // uses current timestamp

  await sender.flush();
  await sender.close();
}
