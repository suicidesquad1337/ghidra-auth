use async_fs as fs;
use async_lock::RwLock;
use blocking::unblock;
use scrypt::{scrypt_check, scrypt_simple};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{io, path::Path, sync::Arc};

#[derive(Default, Debug)]
pub struct UserStore {
    users: RwLock<Vec<User>>,
}

impl UserStore {
    /// Serializes this store and saves it to the given file.
    pub async fn save(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let bytes =
            bincode::serialize(self.users.get_mut()).expect("this should not fail to serialize");
        fs::write(path, bytes).await
    }

    /// Reads bytes from the given file and deserializes it into a `UserStore`.
    pub async fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(path).await?;
        let store: Vec<User> =
            bincode::deserialize(&bytes).expect("someone edited the users file. aborting.");
        Ok(UserStore {
            users: RwLock::new(store),
        })
    }

    /// Inserts a new `User` into this store by hashing the raw password and the username
    /// that was given.
    pub async fn new_user(self: &Arc<Self>, name: impl Into<SmolStr>, raw_password: String) {
        let password = unblock(move || {
            scrypt_simple(&raw_password, &scrypt::ScryptParams::recommended())
                .expect("failed to generate random salt")
        })
        .await;

        self.users.write().await.push(User {
            name: name.into(),
            password,
        });
    }

    /// Removes a user, if one exists, with the given username.
    pub async fn delete_user(self: &Arc<Self>, name: impl Into<SmolStr>) {
        let name = name.into();
        self.users.write().await.retain(|user| user.name != name);
    }

    /// Checks if a user with the given name exists, and verifies the password
    /// with the password stored for given the user.
    pub async fn check_user(self: &Arc<Self>, name: impl Into<SmolStr>, password: String) -> bool {
        let name = name.into();

        if let Some(user) = self
            .users
            .read()
            .await
            .iter()
            .find(|user| user.name == name)
        {
            let user_password = user.password.clone();
            unblock(move || scrypt_check(&password, &user_password).is_ok()).await
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The username
    pub name: SmolStr,
    /// The `scrypt` encoded password hash.
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_store() {
        async fn inner() {
            let store = Arc::new(UserStore::default());
            store.new_user("stu", "hello123".into()).await;

            assert!(store.check_user("stu", "hello123".into()).await);
            assert!(!store.check_user("stu", "invalid".into()).await);

            store.delete_user("stu").await;

            assert!(!store.check_user("stu", "hello123".into()).await);
        }

        smol::block_on(inner());
    }
}
