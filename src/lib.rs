pub mod client;
pub mod event;
pub mod http;
pub mod model;

pub mod prelude {
    pub use crate::client::{Client, ClientBuilder, Context};
    pub use crate::event::EventHandler;
    pub use crate::model::*;
}