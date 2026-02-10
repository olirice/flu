//! Compilation of generated Rust code

use crate::cache::Cache;
use crate::error::{FluError, Result};
use std::path::PathBuf;
use std::process::Command;

/// Compiler for flu expressions
pub struct Compiler {
    /// Path to rustc executable
    rustc_path: PathBuf,
    /// Path to sysroot (for embedded toolchain)
    sysroot: Option<PathBuf>,
}

impl Compiler {
    /// Create a new compiler using system rustc
    pub fn system() -> Result<Self> {
        // Check if rustc is available
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|_| {
                FluError::Toolchain(
                    "rustc not found. Please install Rust from https://rustup.rs/".to_string(),
                )
            })?;

        if !output.status.success() {
            return Err(FluError::Toolchain(
                "rustc not working properly".to_string(),
            ));
        }

        Ok(Self {
            rustc_path: PathBuf::from("rustc"),
            sysroot: None,
        })
    }

    /// Create a compiler with custom rustc path and sysroot
    #[allow(dead_code)]
    pub fn custom(rustc_path: PathBuf, sysroot: Option<PathBuf>) -> Self {
        Self {
            rustc_path,
            sysroot,
        }
    }

    /// Compile source code to binary
    pub fn compile(&self, source_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
        let mut cmd = Command::new(&self.rustc_path);

        cmd.arg("--edition=2021")
            .arg("-C")
            .arg("opt-level=3")
            .arg("--crate-type")
            .arg("bin")
            .arg("-o")
            .arg(output_path)
            .arg(source_path);

        // Add extern crate paths for flu-prelude and its dependencies
        // These will be resolved from the current project
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let workspace_root = PathBuf::from(manifest_dir)
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf());

            if let Some(root) = workspace_root {
                let target_dir = root.join("target").join("debug");
                if target_dir.exists() {
                    cmd.arg("--extern")
                        .arg(format!(
                            "flu_prelude={}/libflu_prelude.rlib",
                            target_dir.display()
                        ))
                        .arg("--extern")
                        .arg(format!(
                            "flu_core={}/libflu_core.rlib",
                            target_dir.display()
                        ))
                        .arg("-L")
                        .arg(format!("dependency={}", target_dir.join("deps").display()));
                }
            }
        }

        // Add sysroot if provided (for embedded toolchain)
        if let Some(sysroot) = &self.sysroot {
            cmd.arg("--sysroot").arg(sysroot);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FluError::Compilation(stderr.to_string()));
        }

        Ok(())
    }

    /// Compile and cache a generated program
    pub fn compile_and_cache(&self, source: &str, cache: &Cache) -> Result<PathBuf> {
        let hash = cache.hash_source(source);

        // Check cache first
        if let Some(binary_path) = cache.get_binary(&hash) {
            return Ok(binary_path);
        }

        // Cache miss - compile
        let source_path = cache.store_source(&hash, source)?;
        let binary_path = cache.binary_path(&hash);

        self.compile(&source_path, &binary_path)?;

        Ok(binary_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_compiler_available() {
        // This test will pass if rustc is installed
        match Compiler::system() {
            Ok(_) => (),
            Err(e) => panic!("System compiler not available: {}", e),
        }
    }
}
