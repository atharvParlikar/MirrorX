import rabbit from "rabbitmq-stream-js-client";

const streamName = "order";

const client = await rabbit.connect({
  hostname: "localhost",
  port: 5672,
  username: "guest",
  password: "guest",
  vhost: "/"
});

const streamSizeRetention = 5 * 1e9;

await client.createStream({ stream: streamName, arguments: { "max-length-bytes": streamSizeRetention } });

export const orderPublisher = await client.declarePublisher({ stream: streamName });
