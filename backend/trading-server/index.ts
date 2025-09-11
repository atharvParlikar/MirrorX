import { Hono } from "hono";
import { cors } from "hono/cors";
import { jwt } from "hono/jwt";
import { setCookie } from "hono/cookie";
import { sendMail } from "./sendMail";
import jsonwebtoken from "jsonwebtoken";
import { Kafka } from "kafkajs";
import { nanoid } from "nanoid";

const app = new Hono();

const requstMap = new Map<string, ({ message }: { message: string }) => void>();

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
});

const producer = kafka.producer();

await producer.connect();

app.use("*", cors());

app.post("/api/v1/signup", async (c) => {
  const { email }: { email: string } = await c.req.json();

  // const token = jsonwebtoken.sign(email, "jwtsecret");

  // if (process.env.NODE_ENV === "production") {
  //   sendMail(email, token);
  // } else {
  //   console.log(
  //     `please visit ${process.env.BACKEND_URL}/api/v1/signin/post?token=${token}`
  //   );
  // }

  await producer.send({
    topic: "priceUpdate",
    messages: [
      {
        key: "createUser",
        value: email,
      }
    ]
  });

  return c.json({ ok: true });
});

app.post("/api/v1/signin", async (c) => {
  const token = c.req.query("token") as string;

  try {
    if (jsonwebtoken.verify(token, "jwtsecret")) {
      setCookie(c, "token", token, { httpOnly: true });
      return c.json({});
    }
  } catch (e) {
    return c.json({ error: "Invalid token" }, 401);
  }
});

app.post("/api/v1/order/open", async (c) => {
  const orderRequest = await c.req.json();
  const headers = c.req.header();
  const user_id = headers.authorization?.split(' ')[1];
  const order_id = nanoid();

  const order = { ...orderRequest, order_id, user_id };

  await producer.send({
    topic: "priceUpdate",
    messages: [
      {
        key: "order",
        value: JSON.stringify(order),
      }
    ]
  });

  return c.json({ message: "sup nigga" });
});

export default app;
