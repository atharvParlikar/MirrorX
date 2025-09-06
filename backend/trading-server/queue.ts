import amqp from "amqplib";

const queue = "order";

const conn = await amqp.connect("amqp://admin:admin@localhost:5672");
const channel = await conn.createChannel();

await channel.assertQueue(queue, { durable: true });

channel.sendToQueue(queue, Buffer.from("Hello order queue!"));
console.log("Message sent");

channel.consume(queue, (msg) => {
  if (msg !== null) {
    console.log("Received:", msg.content.toString());
    channel.ack(msg);
  }
});
