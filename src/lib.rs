use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{io, path::Path};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UserStore {
    users: Vec<User>,
}

impl UserStore {
    /// Serializes this store and saves it to the given file.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let bytes = bincode::serialize(self).expect("this should not fail to serialize");
        std::fs::write(path, bytes)
    }

    /// Reads bytes from the given file and deserializes it into a `UserStore`.
    pub fn load(&self, path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let store: UserStore =
            bincode::deserialize(&bytes).expect("someone edited the users file. aborting.");
        Ok(store)
    }

    /// Inserts a new `User` into this store by hashing the raw password and the username
    /// that was given.
    pub fn new_user(&mut self, name: impl Into<SmolStr>, raw_password: &str) {
        let password = scrypt::scrypt_simple(raw_password, &scrypt::ScryptParams::recommended())
            .expect("failed to generate random salt");

        self.users.push(User {
            name: name.into(),
            password,
        });
    }

    /// Removes a user, if one exists, with the given username.
    pub fn delete_user(&mut self, name: impl Into<SmolStr>) {
        let name = name.into();
        self.users.retain(|user| user.name != name);
    }

    /// Checks if a user with the given name exists, and verifies the password
    /// with the password stored for given the user.
    pub fn check_user(&self, name: impl Into<SmolStr>, password: &str) -> bool {
        let name = name.into();

        if let Some(user) = self.users.iter().find(|user| user.name == name) {
            scrypt::scrypt_check(password, user.password.as_ref()).is_ok()
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
        let mut store = UserStore::default();
        store.new_user("stu", "hello123");

        assert!(store.check_user("stu", "hello123"));
        assert!(!store.check_user("stu", "invalid"));

        store.delete_user("stu");

        assert!(!store.check_user("stu", "hello123"));
    }
}
