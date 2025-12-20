use argon2::{
    Algorithm, Argon2, Params, PasswordHash, Version,
    password_hash::{
        Error, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::{OsRng, RngCore},
    },
};

pub fn hash_password(password: String) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

    Ok(password_hash)
}

pub fn verify_password_hash(password: String, hash: String) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(&hash)?;

    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_valid)
}

pub fn generate_token() -> String {
    let mut token = [0u8; 32];
    OsRng.fill_bytes(&mut token);

    hex::encode(token)
}
