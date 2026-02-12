//! Embedded Rust toolchain extraction and management

use crate::error::{LobError, Result};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

/// Embedded toolchain manager
pub struct EmbeddedToolchain {
    /// Directory containing the extracted toolchain
    toolchain_dir: PathBuf,
}

impl EmbeddedToolchain {
    /// Ensure the embedded toolchain is extracted and ready to use
    ///
    /// On first run, this will extract the embedded toolchain archive to
    /// `~/.cache/lob/toolchain/`. Subsequent runs will use the cached extraction.
    pub fn ensure_extracted() -> Result<Self> {
        let toolchain_dir = Self::toolchain_dir()?;

        if !toolchain_dir.exists() {
            eprintln!("First run: extracting embedded Rust toolchain...");
            Self::extract_embedded_toolchain(&toolchain_dir)?;
            eprintln!("Toolchain ready!");
        }

        Ok(Self { toolchain_dir })
    }

    /// Get the toolchain directory path
    fn toolchain_dir() -> Result<PathBuf> {
        dirs::cache_dir()
            .ok_or_else(|| LobError::Toolchain("No cache directory available".to_string()))
            .map(|dir| dir.join("lob").join("toolchain"))
    }

    /// Extract the embedded toolchain archive
    fn extract_embedded_toolchain(dest: &PathBuf) -> Result<()> {
        // The toolchain archive is embedded at compile time
        const TOOLCHAIN_ARCHIVE: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/toolchain.tar.zst"));

        // Allow const_is_empty check here - it's intentional to check if toolchain was embedded
        #[allow(clippy::const_is_empty)]
        if TOOLCHAIN_ARCHIVE.is_empty() {
            return Err(LobError::Toolchain(
                "No embedded toolchain available. This binary was built without an embedded toolchain.".to_string(),
            ));
        }

        fs::create_dir_all(dest).map_err(|e| {
            LobError::Toolchain(format!("Failed to create toolchain directory: {}", e))
        })?;

        // Decompress zstd archive
        let cursor = Cursor::new(TOOLCHAIN_ARCHIVE);
        let decoder = zstd::Decoder::new(cursor)
            .map_err(|e| LobError::Toolchain(format!("Failed to decompress toolchain: {}", e)))?;

        // Extract tar archive
        let mut archive = tar::Archive::new(decoder);
        archive
            .unpack(dest)
            .map_err(|e| LobError::Toolchain(format!("Failed to extract toolchain: {}", e)))?;

        // Make rustc executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let rustc = dest.join("bin").join("rustc");
            if rustc.exists() {
                let mut perms = fs::metadata(&rustc)
                    .map_err(|e| {
                        LobError::Toolchain(format!("Failed to read rustc permissions: {}", e))
                    })?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&rustc, perms).map_err(|e| {
                    LobError::Toolchain(format!("Failed to set rustc permissions: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Get the path to the rustc binary
    pub fn rustc_path(&self) -> PathBuf {
        self.toolchain_dir.join("bin").join("rustc")
    }

    /// Get the sysroot path for the embedded toolchain
    pub fn sysroot(&self) -> PathBuf {
        self.toolchain_dir.clone()
    }

    /// Check if the embedded toolchain is available and valid
    pub fn is_valid(&self) -> bool {
        self.rustc_path().exists()
    }
}
