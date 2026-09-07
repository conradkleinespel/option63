use crate::carddav::error::ProxyError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

pub const USERNAME_ENV: &str = "O63_CARDDAV_PROXY_USERNAME";
pub const PASSWORD_ENV: &str = "O63_CARDDAV_PROXY_PASSWORD";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

fn decode_value(field: &str, var: &str, encoded: &str) -> Result<String, ProxyError> {
    let bytes = BASE64
        .decode(encoded.trim())
        .map_err(|e| ProxyError::CredsBase64Decode(field.to_string(), var.to_string(), e))?;
    String::from_utf8(bytes).map_err(|_| {
        ProxyError::InvalidChallenge(format!(
            "credentials for environment variable `{var}` is not valid UTF-8 after base64 decoding"
        ))
    })
}

pub trait EnvReader {
    fn var(&self, key: &str) -> Option<String>;
}

pub struct SystemEnv;

impl EnvReader for SystemEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

pub fn load_credentials(env: &dyn EnvReader) -> Result<Credentials, ProxyError> {
    let username = env.var(USERNAME_ENV);
    let password = env.var(PASSWORD_ENV);
    credentials_from_env(username.as_deref(), password.as_deref())
}

fn credentials_from_env(
    username_env: Option<&str>,
    password_env: Option<&str>,
) -> Result<Credentials, ProxyError> {
    let username_encoded = username_env
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ProxyError::CredsMissingEnvironment(USERNAME_ENV.to_string()))?;
    let password_encoded = password_env
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ProxyError::CredsMissingEnvironment(PASSWORD_ENV.to_string()))?;

    let username = decode_value("username", USERNAME_ENV, username_encoded)?;
    let password = decode_value("password", PASSWORD_ENV, password_encoded)?;
    Ok(Credentials { username, password })
}

pub fn encode(username: &str, password: &str) -> (String, String) {
    let user = BASE64.encode(username.as_bytes());
    let pass = BASE64.encode(password.as_bytes());
    (user, pass)
}
