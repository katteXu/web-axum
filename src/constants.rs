use once_cell::sync::Lazy;

use crate::model::Keys;

pub const KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("Env JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub const PORT: Lazy<String> = Lazy::new(|| {
    let port = std::env::var("PORT").expect("Env PORT must be set");
    port
});

pub const DATABASE_URL: Lazy<String> = Lazy::new(|| {
    let db_url = std::env::var("DATABASE_URL").expect("Env DATABASE_URL must be set");
    db_url
});
