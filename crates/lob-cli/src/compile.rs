//! Compilation of generated Rust code

use crate::cache::Cache;
use crate::error::{LobError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of compilation with cache information
pub struct CompileResult {
    /// Path to the compiled binary
    pub binary_path: PathBuf,
    /// Whether the binary was found in cache
    pub cache_hit: bool,
}

/// Compiler for lob expressions
pub struct Compiler {
    /// Path to rustc executable
    rustc_path: PathBuf,
    /// Path to sysroot (for embedded toolchain)
    sysroot: Option<PathBuf>,
}

impl Compiler {
    /// Find the target directory containing compiled lob libraries
    fn find_target_dir() -> Option<PathBuf> {
        // Strategy 1: Use CARGO_MANIFEST_DIR (works during cargo test/run)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let root = PathBuf::from(manifest_dir);

            // Navigate up ancestors to find workspace root with target/ dir containing lob_prelude
            for ancestor in root.ancestors() {
                let debug_dir = ancestor.join("target").join("debug");
                let prelude_lib = debug_dir.join("liblob_prelude.rlib");
                if prelude_lib.exists() {
                    return Some(debug_dir);
                }

                let release_dir = ancestor.join("target").join("release");
                let prelude_lib = release_dir.join("liblob_prelude.rlib");
                if prelude_lib.exists() {
                    return Some(release_dir);
                }
            }
        }

        // Strategy 2: Look relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check if we're in deps/ subdirectory (test executable)
                if exe_dir.ends_with("deps") {
                    if let Some(build_dir) = exe_dir.parent() {
                        // We're in target/debug/deps or target/release/deps
                        let prelude_lib = build_dir.join("liblob_prelude.rlib");
                        if prelude_lib.exists() {
                            return Some(build_dir.to_path_buf());
                        }
                    }
                }

                // Regular executable path handling
                if let Some(target_parent) = exe_dir.parent() {
                    // Try debug first (avoids LTO linking issues)
                    let debug_dir = target_parent.join("debug");
                    let prelude_lib = debug_dir.join("liblob_prelude.rlib");
                    if prelude_lib.exists() {
                        return Some(debug_dir);
                    }

                    // Fall back to release
                    let release_dir = target_parent.join("release");
                    let prelude_lib = release_dir.join("liblob_prelude.rlib");
                    if prelude_lib.exists() {
                        return Some(release_dir);
                    }
                }

                // Try same directory as executable
                let target_dir = exe_dir.to_path_buf();
                let prelude_lib = target_dir.join("liblob_prelude.rlib");
                if prelude_lib.exists() {
                    return Some(target_dir);
                }
            }
        }

        // Strategy 3: Look in current working directory
        if let Ok(cwd) = std::env::current_dir() {
            // Try debug first
            let debug_dir = cwd.join("target").join("debug");
            let prelude_lib = debug_dir.join("liblob_prelude.rlib");
            if prelude_lib.exists() {
                return Some(debug_dir);
            }

            // Try release
            let release_dir = cwd.join("target").join("release");
            let prelude_lib = release_dir.join("liblob_prelude.rlib");
            if prelude_lib.exists() {
                return Some(release_dir);
            }
        }

        // Strategy 4: Try to find workspace root by walking up from cwd
        if let Ok(mut current) = std::env::current_dir() {
            loop {
                let debug_dir = current.join("target").join("debug");
                let prelude_lib = debug_dir.join("liblob_prelude.rlib");
                if prelude_lib.exists() {
                    return Some(debug_dir);
                }

                let release_dir = current.join("target").join("release");
                let prelude_lib = release_dir.join("liblob_prelude.rlib");
                if prelude_lib.exists() {
                    return Some(release_dir);
                }

                // Move up one directory
                if !current.pop() {
                    break;
                }
            }
        }

        None
    }

    /// Create a new compiler using system rustc
    pub fn system() -> Result<Self> {
        // Check if rustc is available
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|_| {
                LobError::Toolchain(
                    "rustc not found. Please install Rust from https://rustup.rs/".to_string(),
                )
            })?;

        if !output.status.success() {
            return Err(LobError::Toolchain(
                "rustc not working properly".to_string(),
            ));
        }

        Ok(Self {
            rustc_path: PathBuf::from("rustc"),
            sysroot: None,
        })
    }

    /// Create a compiler with custom rustc path and sysroot
    pub fn custom(rustc_path: PathBuf, sysroot: Option<PathBuf>) -> Self {
        Self {
            rustc_path,
            sysroot,
        }
    }

    /// Compile source code to binary
    pub fn compile(
        &self,
        source_path: &Path,
        output_path: &Path,
        user_expr: Option<&str>,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.rustc_path);

        cmd.arg("--edition=2021")
            .arg("-C")
            .arg("opt-level=3")
            .arg("--crate-type")
            .arg("bin")
            .arg("-o")
            .arg(output_path)
            .arg(source_path);

        // Add extern crate paths for lob-prelude and its dependencies
        if let Some(target_dir) = Self::find_target_dir() {
            cmd.arg("--extern")
                .arg(format!(
                    "lob_prelude={}/liblob_prelude.rlib",
                    target_dir.display()
                ))
                .arg("--extern")
                .arg(format!(
                    "lob_core={}/liblob_core.rlib",
                    target_dir.display()
                ))
                .arg("-L")
                .arg(format!("dependency={}", target_dir.join("deps").display()));
        }

        // Add sysroot if provided (for embedded toolchain)
        if let Some(sysroot) = &self.sysroot {
            cmd.arg("--sysroot").arg(sysroot);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let formatted = LobError::format_compilation_error(&stderr, user_expr);
            return Err(LobError::Compilation(formatted));
        }

        Ok(())
    }

    /// Compile and cache a generated program
    pub fn compile_and_cache(
        &self,
        source: &str,
        cache: &Cache,
        user_expr: Option<&str>,
    ) -> Result<CompileResult> {
        let hash = cache.hash_source(source);

        // Check cache first
        if let Some(binary_path) = cache.get_binary(&hash) {
            return Ok(CompileResult {
                binary_path,
                cache_hit: true,
            });
        }

        // Cache miss - compile
        let source_path = cache.store_source(&hash, source)?;
        let binary_path = cache.binary_path(&hash);

        self.compile(&source_path, &binary_path, user_expr)?;

        Ok(CompileResult {
            binary_path,
            cache_hit: false,
        })
    }
}
