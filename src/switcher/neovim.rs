use crate::config::NeovimConfig;
use crate::theme::ThemeMode;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub struct NeovimSwitcher {
    config: NeovimConfig,
}

impl NeovimSwitcher {
    pub fn new(config: NeovimConfig) -> Self {
        Self { config }
    }

    pub fn apply(&self, mode: ThemeMode) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let bg_val = match mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };

        let cmd_expr = format!("<Cmd>set background={}<CR>", bg_val);

        // Search for active Neovim server sockets in runtime dirs
        let mut sockets = Vec::new();
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            if let Ok(entries) = fs::read_dir(&runtime_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.starts_with("nvim") {
                        sockets.push(path);
                    }
                }
            }
        }

        for sock in sockets {
            let _ = Command::new("nvim")
                .args(["--server", sock.to_str().unwrap_or(""), "--remote-send", &cmd_expr])
                .output();
            log::info!("Sent background={} to Neovim instance at {:?}", bg_val, sock);
        }

        Ok(())
    }
}
