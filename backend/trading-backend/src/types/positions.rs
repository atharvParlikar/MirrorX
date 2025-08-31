use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwapAny;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::types::types::{CurrentPrice, OpenOrderRequest, WalletManagerMsg};

const LIQUIDATION_THRESHOLD: Decimal = dec!(0.1);

#[derive(Serialize, Clone, Debug)]
pub struct Position {
    pub position_id: String,
    pub asset: String,
    pub entry_price: Decimal,
    pub qty: Decimal,
    pub pnl: Decimal,
    pub margin: Decimal,
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
    pub fn new(latest_price: Arc<ArcSwapAny<Arc<CurrentPrice>>>) -> Positions {
        return Positions {
            position_map: HashMap::new(),
            latest_price,
        };
    }

    pub async fn open(
        &mut self,
        user_id: String,
        order: OpenOrderRequest,
        wallet_tx: UnboundedSender<WalletManagerMsg>,
    ) -> Result<String, String> {
        let (responder_tx, responder_rx) = oneshot::channel::<Option<Decimal>>();
        let sent = wallet_tx.send(WalletManagerMsg::GetBalance {
            user_id: user_id.clone(),
            responder: responder_tx,
        });

        if let Err(_) = sent {
            eprintln!("[error responding to open position function call]");
        }

        let balance: Decimal = match responder_rx.await {
            Ok(Some(balance)) => balance,
            Ok(None) => return Err("Wallet not found".to_string()),
            Err(err) => return Err(err.to_string()),
        };

        let current_price = if order.qty < dec!(0) {
            self.latest_price.load().bid
        } else {
            self.latest_price.load().ask
        };

        if current_price == dec!(0) {
            return Err("Could not process order, server error".to_string());
        }

        let entry_price = if order.qty > dec!(0) {
            self.latest_price.load().ask
        } else {
            self.latest_price.load().bid
        };

        let margin = order.margin.unwrap_or(dec!(0));

        if margin < dec!(0) {
            return Err("Margin cannot be negative".to_string());
        }

        let amount_required = current_price * order.qty.abs() + margin;

        if balance < current_price * order.qty.abs() + margin {
            return Err(format!(
                "Not enough balance, Balance: {}, Needed: {}",
                balance,
                current_price * order.qty.abs() + margin,
            ));
        }

        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<(), String>>();
        wallet_tx
            .send(WalletManagerMsg::Debit {
                user_id: user_id.clone(),
                amount: amount_required,
                responder: oneshot_tx,
            })
            .map_err(|err| err.to_string())?;

        oneshot_rx.await.map_err(|err| err.to_string())??;

        let pnl = (current_price * order.qty) - (entry_price * order.qty);
        let position_id = nanoid::nanoid!();

        let position = Position {
            position_id: position_id.clone(),
            asset: "BTC".to_string(),
            entry_price: entry_price,
            qty: order.qty,
            pnl: pnl,
            margin: margin,
            stop_loss: order.stop_loss,
            take_profit: order.take_profit,
            leverage: order.leverage,
        };

        match self.position_map.get_mut(&user_id.clone()) {
            Some(positions) => {
                positions.push(position);
            }
            None => {
                self.position_map.insert(user_id, vec![position]);
            }
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

        let mut position_index: Option<usize> = None;

        for (idx, position) in positions.iter().enumerate() {
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

                position_index = Some(idx);
            }
        }

        if let Some(idx) = position_index {
            positions.remove(idx);
        }

        Ok(())
    }

    pub fn list(&self, user_id: &String) -> Result<Vec<Position>, String> {
        match self.position_map.get(user_id) {
            Some(position_list) => Ok(position_list.clone()),
            None => Err("Could not find user positions".to_string()),
        }
    }

    pub async fn update_risk(
        &mut self,
        wallet_tx: UnboundedSender<WalletManagerMsg>,
    ) -> Result<(), String> {
        let latest_price = self.latest_price.load();
        let mut positions_to_liquidate: Vec<(String, String)> = Vec::new(); // vec of position_ids

        for (user_id, positions) in self.position_map.iter_mut() {
            for position in positions {
                let current_price = if position.qty < dec!(0) {
                    latest_price.bid
                } else {
                    latest_price.ask
                };

                position.pnl =
                    (current_price * position.qty) - (position.entry_price * position.qty);

                let margin = ((position.entry_price * position.qty.abs())
                    / position.leverage.unwrap_or(dec!(1)))
                    + position.margin;

                if margin < (position.pnl + position.margin) * LIQUIDATION_THRESHOLD {
                    positions_to_liquidate.push((user_id.clone(), position.position_id.clone()));
                    break;
                }

                if let Some(stop_loss_threshold) = position.stop_loss {
                    if position.pnl <= -stop_loss_threshold {
                        positions_to_liquidate
                            .push((user_id.clone(), position.position_id.clone()));
                        break;
                    }
                }

                if let Some(take_profit_threshold) = position.take_profit {
                    if position.pnl >= take_profit_threshold {
                        positions_to_liquidate
                            .push((user_id.clone(), position.position_id.clone()));
                        break;
                    }
                }
            }
        }

        for (user_id, position_id) in positions_to_liquidate {
            let positions = self.position_map.get(&user_id).unwrap();
            let position = positions
                .iter()
                .find(|p| p.position_id == position_id)
                .unwrap();

            self.close(&user_id, position.position_id.clone(), wallet_tx.clone())
                .await?;
        }

        Ok(())
    }
}
