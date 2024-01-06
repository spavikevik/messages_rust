use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordUtil<'a> {
    salt: SaltString,
    argon2: Argon2<'a>,
}

impl<'a> PasswordUtil<'a> {
    pub fn new() -> PasswordUtil<'a> {
        let salt: SaltString = SaltString::generate(&mut OsRng).to_owned();
        let argon2 = Argon2::default();

        Self { salt, argon2 }
    }

    pub fn hash(&self, password: String) -> Option<String> {
        let result = self
            .argon2
            .hash_password(password.as_bytes(), &self.salt)
            .ok()?;

        Some(result.to_string())
    }
}
