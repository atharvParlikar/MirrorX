use crate::types::types::{
    CreateUserMessage, IncomingPrices, KafkaMessages, OpenOrderRequest, SignUpRequest,
};

pub fn handle_kafka_message(key: &str, message: &str) -> KafkaMessages {
    println!("{}", message);
    match key {
        "price" => {
            let prices: IncomingPrices = serde_json::from_str(message).unwrap();
            return KafkaMessages::IncomingPrices(prices);
        }
        "order" => {
            let order: OpenOrderRequest = serde_json::from_str(message).unwrap();
            return KafkaMessages::Order(order);
        }
        "createUser" => {
            let email = message.to_string();
            return KafkaMessages::CreateUser(SignUpRequest { email: email });
        }
        _ => return KafkaMessages::InvalidMessage,
    }
}
