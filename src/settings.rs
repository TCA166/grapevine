use std::{net::SocketAddr, path::PathBuf};

const OUR_NAME: &str = "You";
const DEFAULT_KEY_PATH: &'static str = ".";

pub struct Settings {
    listening: Option<SocketAddr>,
    username: Option<String>,
    default_key_path: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new(None, None, None)
    }
}

impl Settings {
    pub fn new(
        listening: Option<SocketAddr>,
        username: Option<String>,
        default_key_path: Option<PathBuf>,
    ) -> Self {
        let default_key_path = default_key_path
            .unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_PATH).canonicalize().unwrap());

        Self {
            listening,
            username,
            default_key_path,
        }
    }

    pub fn username(&self) -> &str {
        if let Some(uname) = self.username.as_ref() {
            uname.as_str()
        } else {
            OUR_NAME
        }
    }

    pub fn listening(&self) -> &Option<SocketAddr> {
        &self.listening
    }

    pub fn default_key_path(&self) -> &PathBuf {
        &self.default_key_path
    }
}
