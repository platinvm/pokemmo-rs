// Module declarations
pub mod context;
pub mod extensions;
pub mod payloads;
pub mod prelude;
pub mod traits;
pub mod utils;

// Re-exports for convenient access
pub use context::Context;
pub use extensions::{ReadFrameExt, WriteFrameExt};
pub use traits::{Packet, Payload};
