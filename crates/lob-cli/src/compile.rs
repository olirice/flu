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

/// Resolved paths to lob rlib files needed for compilation
struct RlibPaths {
    lob_prelude: PathBuf,
    lob_core: PathBuf,
    deps_dir: PathBuf,
}

/// Compiler for lob expressions
pub struct Compiler {
    /// Path to rustc executable
    rustc_path: PathBuf,
    /// Path to sysroot (for embedded toolchain)
    sysroot: Option<PathBuf>,
}

/// Find a file matching `{prefix}*.rlib` in a directory
fn find_rlib_in_dir(dir: &Path, prefix: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        let name_str = path.file_name()?.to_str()?;
        let is_rlib = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rlib"));
        if name_str.starts_with(prefix) && is_rlib {
            Some(path)
        } else {
            None
        }
    })
}

impl Compiler {
    /// Try to locate lob rlib files in a build directory (debug or release).
    ///
    /// First checks for direct rlibs (`liblob_prelude.rlib` — produced by `cargo build`),
    /// then falls back to searching `deps/` for hashed rlibs (`liblob_prelude-<hash>.rlib`
    /// — the only form produced by `cargo test`).
    fn find_rlibs_in(build_dir: &Path) -> Option<RlibPaths> {
        let deps_dir = build_dir.join("deps");

        // Direct rlibs (from `cargo build`)
        let prelude = build_dir.join("liblob_prelude.rlib");
        let core = build_dir.join("liblob_core.rlib");
        if prelude.exists() && core.exists() {
            return Some(RlibPaths {
                lob_prelude: prelude,
                lob_core: core,
                deps_dir,
            });
        }

        // Hashed rlibs in deps/ (from `cargo test`)
        if deps_dir.is_dir() {
            let prelude = find_rlib_in_dir(&deps_dir, "liblob_prelude-")?;
            let core = find_rlib_in_dir(&deps_dir, "liblob_core-")?;
            return Some(RlibPaths {
                lob_prelude: prelude,
                lob_core: core,
                deps_dir,
            });
        }

        None
    }

    /// Find the rlib paths for `lob_prelude` and `lob_core` across multiple strategies
    fn find_rlib_paths() -> Option<RlibPaths> {
        // Strategy 1: Use CARGO_MANIFEST_DIR (works during cargo test/run)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let root = PathBuf::from(manifest_dir);
            for ancestor in root.ancestors() {
                let target = ancestor.join("target");
                if let Some(rlibs) = Self::find_rlibs_in(&target.join("debug")) {
                    return Some(rlibs);
                }
                if let Some(rlibs) = Self::find_rlibs_in(&target.join("release")) {
                    return Some(rlibs);
                }
            }
        }

        // Strategy 2: Look relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Test executables live in deps/
                if exe_dir.ends_with("deps") {
                    if let Some(build_dir) = exe_dir.parent() {
                        if let Some(rlibs) = Self::find_rlibs_in(build_dir) {
                            return Some(rlibs);
                        }
                    }
                }

                if let Some(target_parent) = exe_dir.parent() {
                    if let Some(rlibs) = Self::find_rlibs_in(&target_parent.join("debug")) {
                        return Some(rlibs);
                    }
                    if let Some(rlibs) = Self::find_rlibs_in(&target_parent.join("release")) {
                        return Some(rlibs);
                    }
                }

                if let Some(rlibs) = Self::find_rlibs_in(exe_dir) {
                    return Some(rlibs);
                }
            }
        }

        // Strategy 3: Look in current working directory
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(rlibs) = Self::find_rlibs_in(&cwd.join("target").join("debug")) {
                return Some(rlibs);
            }
            if let Some(rlibs) = Self::find_rlibs_in(&cwd.join("target").join("release")) {
                return Some(rlibs);
            }
        }

        // Strategy 4: Walk up from cwd
        if let Ok(mut current) = std::env::current_dir() {
            loop {
                let target = current.join("target");
                if let Some(rlibs) = Self::find_rlibs_in(&target.join("debug")) {
                    return Some(rlibs);
                }
                if let Some(rlibs) = Self::find_rlibs_in(&target.join("release")) {
                    return Some(rlibs);
                }
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
        // Compile to a temp directory so intermediate .rcgu.o files don't land
        // in the cache (where concurrent cache-clear could delete them).
        let temp_dir = std::env::temp_dir().join(format!(
            "lob-compile-{}-{}",
            std::process::id(),
            output_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        std::fs::create_dir_all(&temp_dir)?;
        let temp_output = temp_dir.join("binary");

        let mut cmd = Command::new(&self.rustc_path);

        cmd.arg("--edition=2021")
            .arg("-C")
            .arg("opt-level=3")
            .arg("--crate-type")
            .arg("bin")
            .arg("-o")
            .arg(&temp_output)
            .arg(source_path);

        // Add extern crate paths for lob-prelude and its dependencies
        if let Some(rlibs) = Self::find_rlib_paths() {
            cmd.arg("--extern")
                .arg(format!("lob_prelude={}", rlibs.lob_prelude.display()))
                .arg("--extern")
                .arg(format!("lob_core={}", rlibs.lob_core.display()))
                .arg("-L")
                .arg(format!("dependency={}", rlibs.deps_dir.display()));
        }

        // Add sysroot if provided (for embedded toolchain)
        if let Some(sysroot) = &self.sysroot {
            cmd.arg("--sysroot").arg(sysroot);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let _ = std::fs::remove_dir_all(&temp_dir);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let formatted = LobError::format_compilation_error(&stderr, user_expr);
            return Err(LobError::Compilation(formatted));
        }

        // Move compiled binary to the cache location
        std::fs::rename(&temp_output, output_path)
            .or_else(|_| std::fs::copy(&temp_output, output_path).map(|_| ()))?;
        let _ = std::fs::remove_dir_all(&temp_dir);

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
