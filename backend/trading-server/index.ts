import express from "express";
import cors from "cors";
import { orderPublisher } from "./queue";

const app = express();

app.use(cors());

app.post("/api/v1/order/open", (req, res) => {
  const order: OpenOrderRequest = req.body.order;
  //  TODO: AUTH

  orderPublisher.send(Buffer.from(JSON.stringify(order)));
})

app.listen(3000, () => {
  console.log("server is online bitch");
})
