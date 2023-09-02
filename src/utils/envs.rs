use dotenvy::dotenv;
use lazy_static::lazy_static;

pub struct Environment {
    pub database_url: String,
    pub cookie_secret: String,
    pub redis_url: String,
}

impl Environment {
    pub fn init() -> Self {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let cookie_secret = std::env::var("COOKIE_SECRET").expect("COOKIE_SECRET must be set");
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

        Self {
            database_url,
            cookie_secret,
            redis_url,
        }
    }
}

lazy_static! {
    pub static ref ENV: Environment = Environment::init();
}

pub fn database_url() -> String {
    ENV.database_url.clone()
}

pub fn cookie_secret() -> String {
    ENV.cookie_secret.clone()
}

pub fn redis_url() -> String {
    ENV.redis_url.clone()
}
