use std::collections::HashMap;

use uuid::Uuid;

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

    pub fn create_user(&mut self, username: String) -> Result<String, String> {
        let user_id = Uuid::new_v4().to_string();
        match self.user_map.insert(
            user_id.clone(),
            User {
                id: user_id.to_string(),
                username: username,
            },
        ) {
            Some(user) => Ok(user.id),
            None => Err("Could not create user".to_string()),
        }
    }
}
