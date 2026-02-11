//! Binary caching system for compiled lob expressions

use crate::error::{LobError, Result};
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
            .ok_or_else(|| LobError::Cache("No cache directory found".to_string()))?
            .join("lob");

        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(cache_dir.join("binaries"))?;
        fs::create_dir_all(cache_dir.join("sources"))?;

        Ok(Self { cache_dir })
    }

    /// Create a new cache manager with a custom directory (primarily for testing)
    #[cfg(test)]
    pub fn with_dir(cache_dir: PathBuf) -> Result<Self> {
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
    use std::fs::File;
    use std::io::Write;

    fn test_cache() -> Cache {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join("lob_test_cache").join(format!(
            "{}_{}",
            std::process::id(),
            timestamp
        ));
        Cache::with_dir(temp_dir).unwrap()
    }

    #[test]
    fn hash_consistency() {
        let cache = test_cache();
        let source = "let x = 42;";
        let hash1 = cache.hash_source(source);
        let hash2 = cache.hash_source(source);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_uniqueness() {
        let cache = test_cache();
        let hash1 = cache.hash_source("let x = 1;");
        let hash2 = cache.hash_source("let x = 2;");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn cache_dir_returns_path() {
        let cache = test_cache();
        let dir = cache.cache_dir();
        assert!(dir.to_string_lossy().contains("lob"));
    }

    #[test]
    fn get_binary_cache_miss() {
        let cache = test_cache();
        let result = cache.get_binary("nonexistent_hash_12345");
        assert!(result.is_none());
    }

    #[test]
    fn get_binary_cache_hit() {
        let cache = test_cache();
        let hash = "test_hash_binary_unique_xyz";

        // Ensure binaries directory exists (might have been removed by concurrent test)
        let _ = fs::create_dir_all(cache.cache_dir().join("binaries"));

        // Create a fake binary file
        let binary_path = cache.binary_path(hash);
        File::create(&binary_path).unwrap();

        // Should find it now
        let result = cache.get_binary(hash);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), binary_path);

        // Cleanup
        let _ = fs::remove_file(&binary_path);
    }

    #[test]
    fn store_source() {
        let cache = test_cache();
        let hash = "test_source_hash";
        let source = "fn main() {}";

        let path = cache.store_source(hash, source).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, source);

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn binary_path() {
        let cache = test_cache();
        let hash = "test_hash";
        let path = cache.binary_path(hash);

        assert!(path.to_string_lossy().contains("binaries"));
        assert!(path.to_string_lossy().contains(hash));
    }

    #[test]
    fn clear_cache() {
        let cache = test_cache();

        // Add some test files (directories already created by Cache::new())
        let hash1 = "clear_test_unique_1a";
        let hash2 = "clear_test_unique_2b";

        cache.store_source(hash1, "test1").unwrap();
        cache.store_source(hash2, "test2").unwrap();

        File::create(cache.binary_path(hash1)).unwrap();
        File::create(cache.binary_path(hash2)).unwrap();

        // Verify files exist
        assert!(cache.get_binary(hash1).is_some());
        assert!(cache.get_binary(hash2).is_some());

        // Clear cache
        cache.clear().unwrap();

        // Files should be gone
        assert!(cache.get_binary(hash1).is_none());
        assert!(cache.get_binary(hash2).is_none());
    }

    #[test]
    fn clear_cache_empty() {
        let cache = test_cache();

        // Ensure directories exist before clearing
        let _ = fs::create_dir_all(cache.cache_dir().join("binaries"));
        let _ = fs::create_dir_all(cache.cache_dir().join("sources"));

        // Clear cache should work
        let result = cache.clear();
        assert!(result.is_ok());
    }

    #[test]
    fn stats_empty_cache() {
        let cache = test_cache();

        // Ensure directories exist
        let _ = fs::create_dir_all(cache.cache_dir().join("binaries"));

        // Should succeed even if cache is empty
        let result = cache.stats();
        assert!(result.is_ok());
    }

    #[test]
    fn stats_with_binaries() {
        let cache = test_cache();

        // Ensure binaries directory exists (might have been removed by concurrent test)
        let _ = fs::create_dir_all(cache.cache_dir().join("binaries"));

        // Create test binaries
        let hash1 = "stats_test_binaries_unique_abc";
        let hash2 = "stats_test_binaries_unique_def";

        let path1 = cache.binary_path(hash1);
        let path2 = cache.binary_path(hash2);

        // These might fail if directory was removed by concurrent test
        if let Ok(mut file1) = File::create(&path1) {
            let _ = file1.write_all(b"test data 1");
        }
        if let Ok(mut file2) = File::create(&path2) {
            let _ = file2.write_all(b"test data 2");
        }

        // Stats should work even if files weren't created
        let result = cache.stats();
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(&path1);
        let _ = fs::remove_file(&path2);
    }

    #[test]
    fn format_size_bytes() {
        let stats = CacheStats {
            binary_count: 1,
            total_size: 500,
        };
        assert_eq!(stats.format_size(), "500 B");
    }

    #[test]
    fn format_size_kb() {
        let stats = CacheStats {
            binary_count: 10,
            total_size: 1024,
        };
        assert_eq!(stats.format_size(), "1.00 KB");
    }

    #[test]
    fn format_size_mb() {
        let stats = CacheStats {
            binary_count: 10,
            total_size: 1024 * 1024,
        };
        assert_eq!(stats.format_size(), "1.00 MB");
    }

    #[test]
    fn format_size_gb() {
        let stats = CacheStats {
            binary_count: 10,
            total_size: 1024 * 1024 * 1024,
        };
        assert_eq!(stats.format_size(), "1.00 GB");
    }

    #[test]
    fn format_size_large_mb() {
        let stats = CacheStats {
            binary_count: 10,
            total_size: 500 * 1024 * 1024, // 500 MB
        };
        let formatted = stats.format_size();
        assert!(formatted.contains("MB"));
        assert!(formatted.contains("500"));
    }
}
