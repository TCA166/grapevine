mod channel;
pub use channel::Channel;

mod events;

mod handler;
pub use handler::EventRecipient;

mod listener;
pub use listener::{PendingAesHandshake, PendingConnection, PendingRsaHandshake};

mod app;
pub use app::GrapevineApp;

use std::sync::{Arc, Mutex};

type Shared<T> = Arc<Mutex<T>>;
