mod channel;
pub use channel::{Channel, ProtocolError};

mod events;

mod handler;
pub use handler::EventRecipient;

mod listener;

mod app;
pub use app::GrapevineApp;

use std::sync::{Arc, Mutex};

type Shared<T> = Arc<Mutex<T>>;
