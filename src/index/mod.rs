pub mod index;
pub use index::*;

pub mod prelude {
    pub use crate::index::{KeyStorage, NoKeys, UnverifiedIndex, VerifiedIndex, WithKeys};
}
