use std::net::SocketAddr;

const OUR_NAME: &str = "You";

#[derive(Default)]
pub struct Settings {
    listening: Option<SocketAddr>,
    username: Option<String>,
}

impl Settings {
    pub fn new(listening: Option<SocketAddr>, username: Option<String>) -> Self {
        Self {
            listening,
            username,
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
}
