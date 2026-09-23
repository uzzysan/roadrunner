use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub migration_database_url: Option<String>,
    pub default_carrier_id: Uuid,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub port: u16,
    pub host: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: Option<String>,
    pub stripe_webhook_api_version: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let production = env::var("APP_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .is_ok_and(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "production" | "prod"
                )
            });
        let stripe_webhook_secret = optional_env("STRIPE_WEBHOOK_SECRET");
        let stripe_webhook_api_version = optional_env("STRIPE_WEBHOOK_API_VERSION");
        validate_stripe_webhook_config(
            production,
            stripe_webhook_secret.as_deref(),
            stripe_webhook_api_version.as_deref(),
        )
        .unwrap_or_else(|message| panic!("{message}"));

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            migration_database_url: optional_env("MIGRATION_DATABASE_URL"),
            default_carrier_id: env::var("DEFAULT_CARRIER_ID")
                .unwrap_or_else(|_| "00000000-0000-4000-8000-000000000001".to_string())
                .parse()
                .expect("DEFAULT_CARRIER_ID must be a UUID"),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(86400),
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            stripe_secret_key: env::var("STRIPE_SECRET_KEY")
                .unwrap_or_else(|_| "sk_test_...".to_string()),
            stripe_webhook_secret,
            stripe_webhook_api_version,
        }
    }
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn validate_stripe_webhook_config(
    production: bool,
    secret: Option<&str>,
    api_version: Option<&str>,
) -> Result<(), &'static str> {
    if secret.is_some() != api_version.is_some() {
        return Err(
            "STRIPE_WEBHOOK_SECRET and STRIPE_WEBHOOK_API_VERSION must be configured together",
        );
    }
    if production && secret.is_none() {
        return Err(
            "STRIPE_WEBHOOK_SECRET and STRIPE_WEBHOOK_API_VERSION must be set in production",
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_stripe_webhook_config;

    #[test]
    fn webhook_config_may_be_disabled_outside_production() {
        assert!(validate_stripe_webhook_config(false, None, None).is_ok());
    }

    #[test]
    fn webhook_config_is_fail_closed_in_production() {
        assert!(validate_stripe_webhook_config(true, None, None).is_err());
        assert!(validate_stripe_webhook_config(true, Some("whsec_test"), None).is_err());
        assert!(validate_stripe_webhook_config(true, None, Some("2024-06-20")).is_err());
        assert!(
            validate_stripe_webhook_config(true, Some("whsec_test"), Some("2024-06-20")).is_ok()
        );
    }
}
