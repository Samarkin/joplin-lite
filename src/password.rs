use crate::error::JoplinError;
use std::env;

pub trait JoplinPasswordProvider: Send {
    fn get_password(&mut self, key_id: &str) -> Result<String, JoplinError>;
}

pub struct JoplinEnvPasswordProvider;

impl JoplinEnvPasswordProvider {
    pub fn new() -> JoplinEnvPasswordProvider {
        JoplinEnvPasswordProvider {}
    }
}

impl JoplinPasswordProvider for JoplinEnvPasswordProvider {
    fn get_password(&mut self, _: &str) -> Result<String, JoplinError> {
        Ok(env::var("JOPLIN_PASSWORD")?)
    }
}

impl From<env::VarError> for JoplinError {
    fn from(err: env::VarError) -> Self {
        JoplinError::Password(format!("env var error: {}", err))
    }
}
