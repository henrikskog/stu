use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{self, Write};
use std::num::NonZeroUsize;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    key: String,
    value: T,
}

pub struct SyncMokaCache<T> {
    pub cache: Cache<String, T>,
    pub file_path: String,
}

impl<T> fmt::Debug for SyncMokaCache<T>
where
    T: fmt::Debug + Clone + Send + Sync + 'static, // Added 'static bound
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncMokaCache")
            .field("file_path", &self.file_path)
            .field("cache", &self.cache.iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<T> SyncMokaCache<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    pub fn new(size: NonZeroUsize, file_path: String) -> io::Result<Self>
    where
        T: for<'de> Deserialize<'de>,
    {
        println!("Initializing cache with size: {} at {}", size, file_path);
        let cache = Cache::builder().max_capacity(size.get() as u64).build();

        if let Ok(mut file) = File::open(&file_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let entries: Vec<CacheEntry<T>> = serde_json::from_str(&contents)?;
            for entry in entries {
                cache.insert(entry.key, entry.value);
            }
        }

        Ok(SyncMokaCache { cache, file_path })
    }

    pub fn put(&self, key: String, value: T) -> io::Result<()> {
        self.cache.insert(key.clone(), value);
        self.sync_to_file()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.cache.get(key)
    }

    fn sync_to_file(&self) -> io::Result<()> {
        let temp_file_path = format!("{}.tmp", self.file_path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)?;
        let entries: Vec<CacheEntry<T>> = self
            .cache
            .iter()
            .map(|(k, v)| CacheEntry {
                key: k.to_string(),
                value: v.clone(),
            })
            .collect();

        let json = serde_json::to_string(&entries)?;
        file.write_all(json.as_bytes())?;
        std::fs::rename(temp_file_path, &self.file_path)?;
        Ok(())
    }
}

// fn main2() -> io::Result<()> {
//     let file_path = "cache.json".to_string();
//     let cache = SyncMokaCache::new(NonZeroUsize::new(2).unwrap(), file_path)?;

//     println!("{:?}", cache);

//     cache.put("apple".to_string(), 3)?;
//     cache.put("banana".to_string(), 2)?;

//     println!("{:?}", cache);

//     assert_eq!(cache.get("apple").unwrap(), 3);
//     assert_eq!(cache.get("banana").unwrap(), 2);
//     assert!(cache.get("pear").is_none());

//     cache.put("banana".to_string(), 4)?;
//     cache.put("pear".to_string(), 5)?;

//     println!("{:?}", cache);

//     assert_eq!(cache.get("pear").unwrap(), 5);
//     assert_eq!(cache.get("banana").unwrap(), 4);
//     assert!(cache.get("apple").is_none());

//     // Moka does not support mutable references directly.
//     // You might need to use a different approach if mutability is required.

//     println!("{:?}", cache);

//     assert_eq!(cache.get("banana").unwrap(), 6);

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_main() {
//         println!("test_main");
//         main2();
//     }
// }
