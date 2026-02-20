pub mod build;
mod configs;
mod destinations;
mod detect;
pub mod launch;
mod schemes;

pub use build::cmd_build;
pub use configs::cmd_configs;
pub use destinations::cmd_destinations;
pub use detect::cmd_detect;
pub use launch::cmd_launch;
pub use schemes::cmd_schemes;
