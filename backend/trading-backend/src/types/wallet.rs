use std::collections::HashMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone)]
pub struct Wallet {
    pub user_id: String,
    pub balance: Decimal,
}

pub struct Wallets {
    pub wallet_map: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Wallets {
        return Wallets {
            wallet_map: HashMap::new(),
        };
    }

    pub fn update_balance(&mut self, user_id: String, new_balance: Decimal) -> Result<(), String> {
        self.wallet_map
            .get_mut(&user_id)
            .map(|wallet| wallet.balance = new_balance)
            .ok_or_else(|| "Could not find wallet".to_string())
    }

    pub fn get_balance(&self, user_id: &String) -> Option<Decimal> {
        match self.wallet_map.get(user_id) {
            Some(wallet) => Some(wallet.balance),
            None => None,
        }
    }

    pub fn create(&mut self, user_id: String) -> Result<(), String> {
        if self.wallet_map.contains_key(&user_id) {
            return Err("Wallet already exists".to_string());
        }

        self.wallet_map.insert(
            user_id.clone(),
            Wallet {
                user_id: user_id.clone(),
                balance: dec!(10_000.0),
            },
        );

        Ok(())
    }
}
