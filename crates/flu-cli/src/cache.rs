//! Binary caching system for compiled flu expressions

use crate::error::{FluError, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Manages compiled binary cache
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| FluError::Cache("No cache directory found".to_string()))?
            .join("flu");

        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(cache_dir.join("binaries"))?;
        fs::create_dir_all(cache_dir.join("sources"))?;

        Ok(Self { cache_dir })
    }

    /// Get the cache directory path
    #[allow(clippy::missing_const_for_fn)]
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Hash source code to generate cache key
    #[allow(clippy::unused_self)]
    pub fn hash_source(&self, source: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if a binary exists in cache
    pub fn get_binary(&self, hash: &str) -> Option<PathBuf> {
        let path = self.cache_dir.join("binaries").join(hash);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Store source code in cache (for debugging)
    pub fn store_source(&self, hash: &str, source: &str) -> Result<PathBuf> {
        let path = self.cache_dir.join("sources").join(format!("{}.rs", hash));
        fs::write(&path, source)?;
        Ok(path)
    }

    /// Get binary path (whether it exists or not)
    pub fn binary_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join("binaries").join(hash)
    }

    /// Get source path (whether it exists or not)
    #[allow(dead_code)]
    pub fn source_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join("sources").join(format!("{}.rs", hash))
    }

    /// Clear all cached binaries
    pub fn clear(&self) -> Result<()> {
        let binaries_dir = self.cache_dir.join("binaries");
        if binaries_dir.exists() {
            fs::remove_dir_all(&binaries_dir)?;
            fs::create_dir_all(&binaries_dir)?;
        }

        let sources_dir = self.cache_dir.join("sources");
        if sources_dir.exists() {
            fs::remove_dir_all(&sources_dir)?;
            fs::create_dir_all(&sources_dir)?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let binaries_dir = self.cache_dir.join("binaries");
        let mut binary_count = 0;
        let mut total_size = 0u64;

        if binaries_dir.exists() {
            for entry in fs::read_dir(&binaries_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    binary_count += 1;
                    total_size += entry.metadata()?.len();
                }
            }
        }

        Ok(CacheStats {
            binary_count,
            total_size,
        })
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Number of cached binaries
    pub binary_count: usize,
    /// Total size of cached binaries in bytes
    pub total_size: u64,
}

impl CacheStats {
    /// Format total size in human-readable format
    pub fn format_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if self.total_size >= GB {
            format!("{:.2} GB", self.total_size as f64 / GB as f64)
        } else if self.total_size >= MB {
            format!("{:.2} MB", self.total_size as f64 / MB as f64)
        } else if self.total_size >= KB {
            format!("{:.2} KB", self.total_size as f64 / KB as f64)
        } else {
            format!("{} B", self.total_size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_consistency() {
        let cache = Cache::new().unwrap();
        let source = "let x = 42;";
        let hash1 = cache.hash_source(source);
        let hash2 = cache.hash_source(source);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_uniqueness() {
        let cache = Cache::new().unwrap();
        let hash1 = cache.hash_source("let x = 1;");
        let hash2 = cache.hash_source("let x = 2;");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn format_size() {
        let stats = CacheStats {
            binary_count: 10,
            total_size: 1024,
        };
        assert_eq!(stats.format_size(), "1.00 KB");

        let stats = CacheStats {
            binary_count: 10,
            total_size: 1024 * 1024,
        };
        assert_eq!(stats.format_size(), "1.00 MB");
    }
}
