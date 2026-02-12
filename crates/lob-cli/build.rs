//! Build script to embed Rust toolchain
//!
//! This script creates a minimal Rust toolchain archive and embeds it in the lob binary.
//! The toolchain includes rustc and the necessary standard library components.
//!
//! Set `LOB_EMBED_TOOLCHAIN=1` to embed a full toolchain. Otherwise, an empty
//! placeholder is created (and the runtime will fall back to system rustc).

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let archive_path = out_dir.join("toolchain.tar.zst");

    // Check if we should embed a full toolchain
    let embed_toolchain = env::var("LOB_EMBED_TOOLCHAIN").unwrap_or_default() == "1";

    if embed_toolchain {
        println!(
            "cargo:warning=Embedding Rust toolchain (this will increase binary size significantly)"
        );
        create_toolchain_archive(&archive_path)?;
    } else {
        println!("cargo:warning=Creating placeholder toolchain (will use system rustc at runtime)");
        create_placeholder_archive(&archive_path)?;
    }

    println!("cargo:rerun-if-env-changed=LOB_EMBED_TOOLCHAIN");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Create a full toolchain archive from the system's Rust installation
fn create_toolchain_archive(archive_path: &Path) -> io::Result<()> {
    // Find system rustc
    let rustc_path = find_rustc()?;
    let sysroot = get_sysroot(&rustc_path)?;

    println!("cargo:warning=Found rustc at: {}", rustc_path.display());
    println!("cargo:warning=Found sysroot at: {}", sysroot.display());

    // Create a temporary directory for the toolchain
    let temp_dir = env::temp_dir().join(format!("lob-toolchain-{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Create toolchain structure
    let toolchain_bin = temp_dir.join("bin");
    let toolchain_lib = temp_dir.join("lib");
    fs::create_dir_all(&toolchain_bin)?;
    fs::create_dir_all(&toolchain_lib)?;

    // Copy rustc binary
    let dest_rustc = toolchain_bin.join("rustc");
    fs::copy(&rustc_path, &dest_rustc)?;
    println!(
        "cargo:warning=Copied rustc: {} bytes",
        fs::metadata(&dest_rustc)?.len()
    );

    // Copy essential sysroot components
    copy_sysroot_libs(&sysroot, &toolchain_lib)?;

    // Create compressed archive
    create_compressed_archive(&temp_dir, archive_path)?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_dir)?;

    let archive_size = fs::metadata(archive_path)?.len();
    println!(
        "cargo:warning=Created toolchain archive: {} MB",
        archive_size / 1_000_000
    );

    Ok(())
}

/// Create an empty placeholder archive
fn create_placeholder_archive(archive_path: &Path) -> io::Result<()> {
    File::create(archive_path)?;
    Ok(())
}

/// Find the rustc binary
fn find_rustc() -> io::Result<PathBuf> {
    let output = Command::new("which")
        .arg("rustc")
        .output()
        .or_else(|_| Command::new("where").arg("rustc").output())?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "rustc not found in PATH",
        ));
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No rustc path"))?
        .to_string();

    Ok(PathBuf::from(path))
}

/// Get the sysroot path from rustc
fn get_sysroot(rustc: &Path) -> io::Result<PathBuf> {
    let output = Command::new(rustc).arg("--print").arg("sysroot").output()?;

    if !output.status.success() {
        return Err(io::Error::other("Failed to get sysroot"));
    }

    let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(sysroot))
}

/// Copy essential libraries from sysroot
fn copy_sysroot_libs(sysroot: &Path, dest_lib: &Path) -> io::Result<()> {
    let src_lib = sysroot.join("lib");
    if !src_lib.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Sysroot lib directory not found: {}", src_lib.display()),
        ));
    }

    // Copy rustlib for the host target
    let rustlib_src = src_lib.join("rustlib");
    if rustlib_src.exists() {
        let rustlib_dest = dest_lib.join("rustlib");
        copy_dir_recursive(&rustlib_src, &rustlib_dest)?;
    }

    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            // Only copy essential files (skip source, docs, etc.)
            if let Some(ext) = src_path.extension() {
                let ext = ext.to_string_lossy();
                if ext == "rlib" || ext == "so" || ext == "dylib" || ext == "dll" || ext == "a" {
                    fs::copy(&src_path, &dst_path)?;
                }
            }
        }
    }

    Ok(())
}

/// Create a compressed tar archive
fn create_compressed_archive(source_dir: &Path, archive_path: &Path) -> io::Result<()> {
    let archive_file = File::create(archive_path)?;
    let encoder = zstd::Encoder::new(archive_file, 19)?; // Max compression
    let mut tar = tar::Builder::new(encoder.auto_finish());

    tar.append_dir_all(".", source_dir)?;
    tar.finish()?;

    Ok(())
}
