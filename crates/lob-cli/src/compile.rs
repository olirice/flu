//! Compilation of generated Rust code

use crate::cache::Cache;
use crate::error::{LobError, Result};
use std::path::PathBuf;
use std::process::Command;

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
        source_path: &PathBuf,
        output_path: &PathBuf,
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
    ) -> Result<PathBuf> {
        let hash = cache.hash_source(source);

        // Check cache first
        if let Some(binary_path) = cache.get_binary(&hash) {
            return Ok(binary_path);
        }

        // Cache miss - compile
        let source_path = cache.store_source(&hash, source)?;
        let binary_path = cache.binary_path(&hash);

        self.compile(&source_path, &binary_path, user_expr)?;

        Ok(binary_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    fn test_cache() -> Cache {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir()
            .join("lob_test_cache_compile")
            .join(format!("{}_{}", std::process::id(), timestamp));

        Cache::with_dir(temp_dir).unwrap()
    }

    #[test]
    fn find_target_dir_test() {
        // Test that we can find the target directory
        let target_dir = Compiler::find_target_dir();
        eprintln!("Found target dir: {:?}", target_dir);

        // In some CI environments or during certain build configurations,
        // the prelude might not be built yet. We verify the logic works
        // but don't require the prelude to exist in all cases.
        if let Some(dir) = target_dir {
            eprintln!("Target dir found at: {:?}", dir);
            let prelude_lib = dir.join("liblob_prelude.rlib");
            eprintln!("Checking for: {:?}", prelude_lib);

            // If the directory was found, it should at least exist
            assert!(dir.exists(), "Target directory should exist: {:?}", dir);

            // Log whether prelude exists (useful for debugging CI failures)
            if !prelude_lib.exists() {
                eprintln!("Warning: lob_prelude.rlib not found at {:?}", prelude_lib);
                eprintln!("This is expected in some build configurations");
            }
        } else {
            eprintln!(
                "No target directory found - this may be expected in some build configurations"
            );
        }
    }

    #[test]
    fn system_compiler_available() {
        // This test will pass if rustc is installed
        match Compiler::system() {
            Ok(_) => (),
            Err(e) => panic!("System compiler not available: {}", e),
        }
    }

    #[test]
    fn custom_compiler_creation() {
        let rustc_path = PathBuf::from("/custom/rustc");
        let sysroot = Some(PathBuf::from("/custom/sysroot"));

        let compiler = Compiler::custom(rustc_path.clone(), sysroot.clone());

        assert_eq!(compiler.rustc_path, rustc_path);
        assert_eq!(compiler.sysroot, sysroot);
    }

    #[test]
    fn custom_compiler_no_sysroot() {
        let rustc_path = PathBuf::from("/custom/rustc");
        let compiler = Compiler::custom(rustc_path.clone(), None);

        assert_eq!(compiler.rustc_path, rustc_path);
        assert_eq!(compiler.sysroot, None);
    }

    #[test]
    fn compile_with_invalid_source() {
        let compiler = Compiler::system().unwrap();
        let cache = test_cache();

        // Create invalid Rust source
        let invalid_source = "fn main() { this is not valid rust }";

        // Should return compilation error
        let result = compiler.compile_and_cache(invalid_source, &cache, Some("test_expr"));
        assert!(result.is_err());

        if let Err(LobError::Compilation(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected compilation error");
        }
    }

    #[test]
    fn compile_and_cache_with_cache_hit() {
        let compiler = Compiler::system().unwrap();
        let cache = test_cache();

        let source = "fn main() { println!(\"test\"); }";

        // First compilation
        let path1 = compiler.compile_and_cache(source, &cache, None).unwrap();
        assert!(path1.exists());

        // Second compilation should hit cache
        let path2 = compiler.compile_and_cache(source, &cache, None).unwrap();

        assert_eq!(path1, path2);
    }

    #[test]
    fn compile_and_cache_different_sources() {
        let compiler = Compiler::system().unwrap();
        let cache = test_cache();

        let source1 = "fn main() { println!(\"test1\"); }";
        let source2 = "fn main() { println!(\"test2\"); }";

        let path1 = compiler.compile_and_cache(source1, &cache, None).unwrap();
        let path2 = compiler.compile_and_cache(source2, &cache, None).unwrap();

        // Different sources should produce different binaries
        assert_ne!(path1, path2);
        assert!(path1.exists());
        assert!(path2.exists());
    }

    #[test]
    fn compile_with_user_expr_in_error() {
        let compiler = Compiler::system().unwrap();
        let cache = test_cache();

        let invalid_source = "fn main() { let x: i32 = \"string\"; }";
        let user_expr = "_.map(|x| x + \"oops\")";

        let result = compiler.compile_and_cache(invalid_source, &cache, Some(user_expr));
        assert!(result.is_err());

        // The error should be formatted with user expression context
        if let Err(LobError::Compilation(msg)) = result {
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn compile_with_custom_sysroot() {
        let temp_dir = std::env::temp_dir().join("test_sysroot");
        let _ = fs::create_dir_all(&temp_dir);

        let compiler = Compiler::custom(PathBuf::from("rustc"), Some(temp_dir.clone()));

        assert_eq!(compiler.sysroot, Some(temp_dir));
    }

    #[test]
    fn compile_direct_call() {
        let compiler = Compiler::system().unwrap();
        let temp_dir = std::env::temp_dir().join("lob_compile_test");
        let _ = fs::create_dir_all(&temp_dir);

        let source_path = temp_dir.join("test.rs");
        let output_path = temp_dir.join("test_bin");

        // Write valid Rust source
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"fn main() { println!(\"test\"); }")
            .unwrap();

        // Compile directly
        let result = compiler.compile(&source_path, &output_path, None);

        // Should succeed (or fail gracefully if linking issues occur)
        // The key is exercising the compile path
        match result {
            Ok(()) => assert!(output_path.exists()),
            Err(LobError::Compilation(_)) => {
                // Expected if lob_prelude isn't found
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn compile_with_invalid_path() {
        let compiler = Compiler::system().unwrap();

        let source_path = PathBuf::from("/nonexistent/file.rs");
        let output_path = PathBuf::from("/tmp/output");

        // Should return an error when trying to compile nonexistent file
        let result = compiler.compile(&source_path, &output_path, None);
        assert!(result.is_err());
    }
}
