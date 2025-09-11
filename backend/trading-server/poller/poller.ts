import { Kafka } from 'kafkajs';
import { WebSocket } from "ws";

const ws = new WebSocket('wss://ws.backpack.exchange/');

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
});

const producer = kafka.producer();

type BookTicker = {
  e: 'bookTicker';
  s: string;
  a: string;
  A: string;
  b: string;
  B: string;
  T: number;
  E: number;
  u: number;
};

type Price = {
  bid: string;
  ask: string;
};

let BTC: Price | null = null;
let SOL: Price | null = null;
let ETH: Price | null = null;

ws.on('open', () => {
  ws.send(
    JSON.stringify({
      method: 'SUBSCRIBE',
      params: ['bookTicker.SOL_USDC_PERP'],
      id: 1,
    })
  );

  ws.send(
    JSON.stringify({
      method: 'SUBSCRIBE',
      params: ['bookTicker.BTC_USDC_PERP'],
      id: 2,
    })
  );

  ws.send(
    JSON.stringify({
      method: 'SUBSCRIBE',
      params: ['bookTicker.ETH_USDC_PERP'],
      id: 3,
    })
  );
});

ws.on('message', (msg) => {
  try {
    const parsedMessage: BookTicker = JSON.parse(msg.toString()).data;

    const price = {
      bid: parsedMessage.b,
      ask: parsedMessage.a,
    };

    if (parsedMessage.s === 'BTC_USDC_PERP') BTC = price;
    else if (parsedMessage.s === 'ETH_USDC_PERP') ETH = price;
    else if (parsedMessage.s === 'SOL_USDC_PERP') SOL = price;
  } catch (err) {
    console.error('WebSocket message parsing error:', err);
  }
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err);
});

let counter = 1;

async function startProducer() {
  try {
    await producer.connect();
    console.log('Kafka producer connected');

    setInterval(async () => {
      if (!BTC || !ETH || !SOL) {
        console.log('Waiting for all prices:', { BTC, ETH, SOL });
        return;
      }

      try {
        await producer.send({
          topic: 'priceUpdate',
          messages: [
            {
              key: "price",
              partition: 0,
              value: JSON.stringify({
                BTC,
                ETH,
                SOL,
              }),
            },
          ],
        });
        console.log('Sent message', counter++);
      } catch (err) {
        console.error('Kafka send error:', err);
      }
    }, 100);
  } catch (err) {
    console.error('Kafka producer connection error:', err);
  }
}

startProducer();

process.on('SIGINT', async () => {
  console.log('Disconnecting producer and closing WebSocket...');
  await producer.disconnect();
  ws.close();
  process.exit(0);
});
