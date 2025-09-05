/// Basic messaging protocol functionality
mod protocol;
pub use protocol::Message;

/// [std::net::TcpStream] handling functionality through the [Channel] class
mod channel;
pub use channel::{Channel, ChannelDesc};

/// Library-wide events
mod events;

/// Handler for the events laid out in [events]
mod handler;
pub use handler::EventRecipient;

/// Server thread functionality
mod listener;
pub use listener::{PendingAesHandshake, PendingConnection, PendingRsaHandshake};

/// Core self contained app.
mod app;
pub use app::GrapevineApp;

use std::sync::{Arc, Mutex};

type Shared<T> = Arc<Mutex<T>>;
