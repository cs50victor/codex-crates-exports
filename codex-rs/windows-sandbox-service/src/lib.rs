use anyhow::Result;

#[cfg(windows)]
pub mod installation_record;
#[cfg(windows)]
pub mod ipc;

#[cfg(windows)]
pub mod machine_policy;
#[cfg(windows)]
pub mod package_identity;
#[cfg(windows)]
pub mod package_lifecycle;
#[cfg(windows)]
pub mod provisioning;
#[cfg(windows)]
pub mod registered_runtime;
#[cfg(windows)]
pub mod service;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunMode {
    Service,
    #[cfg(debug_assertions)]
    Foreground,
}

pub fn run(mode: RunMode) -> Result<()> {
    #[cfg(windows)]
    {
        match mode {
            RunMode::Service => service::run(),
            #[cfg(debug_assertions)]
            RunMode::Foreground => service::run_foreground(),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = mode;
        anyhow::bail!("the Codex sandbox service is only available on Windows")
    }
}
