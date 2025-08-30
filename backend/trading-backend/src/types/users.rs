use std::collections::HashMap;

use rust_decimal_macros::dec;
use tokio::sync::{mpsc, oneshot};

use crate::types::types::WalletManagerMsg;

pub struct User {
    pub id: String,
    pub username: String,
}

pub struct Users {
    user_map: HashMap<String, User>,
}

impl Users {
    pub fn new() -> Users {
        Users {
            user_map: HashMap::new(),
        }
    }

    pub async fn create_user(
        &mut self,
        username: String,
        wallet_sender: mpsc::UnboundedSender<WalletManagerMsg>,
    ) -> Result<String, String> {
        let user_id = nanoid::nanoid!();
        self.user_map.insert(
            user_id.clone(),
            User {
                id: user_id.to_string(),
                username: username,
            },
        );

        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<(), String>>();

        wallet_sender
            .send(WalletManagerMsg::Create {
                user_id: user_id.clone(),
                responder: oneshot_tx,
            })
            .map_err(|_| "could not create wallet, canceling user creation".to_string())?;

        oneshot_rx.await.map_err(|_| {
            //  TODO: delete user here...
            return "could not create wallet, canceling user creation".to_string();
        })??;

        Ok(user_id)
    }
}
