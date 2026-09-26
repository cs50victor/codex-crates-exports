//! Command parsing and safety utilities shared across Codex crates.

pub mod shell_detect;
pub mod shell_snapshot;
pub mod startup;

pub use startup::shell_startup_script;

pub mod bash;
pub mod command_safety;
pub mod parse_command;
pub mod powershell;

pub use command_safety::is_dangerous_command;
