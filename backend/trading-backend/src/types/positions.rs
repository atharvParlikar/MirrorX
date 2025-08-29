use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwapAny;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use uuid::Uuid;

use crate::types::types::{CurrentPrice, OpenOrderRequest, WalletManagerMsg};

#[derive(Clone, Debug)]
pub struct Position {
    pub position_id: String,
    pub asset: String,
    pub entry_price: Decimal,
    pub qty: Decimal,
    pub pnl: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub leverage: Option<Decimal>,
}

#[derive(Debug)]
pub struct Positions {
    pub position_map: HashMap<String, Vec<Position>>,
    pub latest_price: Arc<ArcSwapAny<Arc<CurrentPrice>>>,
}

impl Positions {
    pub fn new(latestPrice: Arc<ArcSwapAny<Arc<CurrentPrice>>>) -> Positions {
        return Positions {
            position_map: HashMap::new(),
            latest_price: latestPrice,
        };
    }

    pub async fn open(
        &mut self,
        user_id: String,
        order: OpenOrderRequest,
        wallet_tx: UnboundedSender<WalletManagerMsg>,
    ) -> Result<String, String> {
        let (responder_tx, responder_rx) = oneshot::channel::<Option<Decimal>>();
        wallet_tx
            .send(WalletManagerMsg::GetBalance {
                user_id: user_id.clone(),
                responder: responder_tx,
            })
            .unwrap();

        let balance: Decimal = match responder_rx.await {
            Ok(Some(balance)) => balance,
            Ok(None) => return Err("Wallet not found".to_string()),
            Err(err) => return Err(err.to_string()),
        };

        let current_price = if order.qty > dec!(0) {
            self.latest_price.load().bid
        } else {
            self.latest_price.load().ask
        };

        let entry_price = if order.qty > dec!(0) {
            self.latest_price.load().ask
        } else {
            self.latest_price.load().bid
        };

        if balance < current_price * order.qty.abs() {
            return Err(format!(
                "Not enough balance, Balance: {}, Needed: {}",
                balance, current_price,
            ));
        }

        let pnl = (current_price * order.qty) - (entry_price * order.qty);
        let position_id = Uuid::new_v4().to_string();

        let position = Position {
            position_id: position_id.clone(),
            asset: "BTC".to_string(),
            entry_price: entry_price,
            qty: order.qty,
            pnl: pnl,
            stop_loss: order.stop_loss,
            take_profit: order.take_profit,
            leverage: order.leverage,
        };

        match self.position_map.get_mut(&user_id.clone()) {
            Some(positions) => {
                positions.push(position);
            }
            None => {}
        };

        Ok(position_id)
    }

    pub async fn close(
        &mut self,
        user_id: &String,
        position_id: String,
        wallet_tx: UnboundedSender<WalletManagerMsg>,
    ) -> Result<(), String> {
        if !self.position_map.contains_key(user_id) {
            return Err("Could not find user".to_string());
        }

        let positions = self.position_map.get_mut(user_id).unwrap();
        let latest_price = self.latest_price.load();

        for position in positions.iter() {
            if position.position_id == position_id {
                let current_price = if position.qty > dec!(0) {
                    latest_price.bid
                } else {
                    latest_price.ask
                };

                let (oneshot_tx, oneshot_rx) = oneshot::channel::<Option<Decimal>>();
                wallet_tx
                    .send(WalletManagerMsg::GetBalance {
                        user_id: user_id.clone(),
                        responder: oneshot_tx,
                    })
                    .map_err(|x| x.to_string())?;

                let balance = match oneshot_rx.await {
                    Ok(Some(balance)) => balance,
                    Ok(None) => return Err("Could not get balance".to_string()),
                    Err(err) => return Err(err.to_string()),
                };

                let new_balance = (current_price * position.qty)
                    - (position.entry_price * position.qty)
                    + balance;

                let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<(), String>>();
                wallet_tx
                    .send(WalletManagerMsg::Credit {
                        user_id: user_id.clone(),
                        amount: new_balance,
                        responder: oneshot_tx,
                    })
                    .map_err(|x| x.to_string())?;

                oneshot_rx
                    .await
                    .map_err(|_| "[POSITIONS CLOSE ERROR] oneshot recv channel closed")??;
            }
        }

        Ok(())
    }

    pub fn list(&self, user_id: &String) -> Result<Vec<Position>, String> {
        match self.position_map.get(user_id) {
            Some(position_list) => Ok(position_list.clone()),
            None => Err("Could not find user positions".to_string()),
        }
    }
}
