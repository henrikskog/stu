use lru::LruCache;
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

pub struct SyncLruCache<T> {
    pub cache: LruCache<String, T>,
    pub file_path: String,
}

impl<T> fmt::Debug for SyncLruCache<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncLruCache")
            .field("file_path", &self.file_path)
            .field("cache", &self.cache.iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<T> SyncLruCache<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new(size: NonZeroUsize, file_path: String) -> io::Result<Self>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cache = if let Ok(mut file) = File::open(&file_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let entries: Vec<CacheEntry<T>> = serde_json::from_str(&contents)?;
            let mut cache = LruCache::new(size);
            for entry in entries {
                cache.put(entry.key, entry.value);
            }
            cache
        } else {
            LruCache::new(size)
        };
        Ok(SyncLruCache { cache, file_path })
    }

    pub fn put(&mut self, key: String, value: T) -> io::Result<Option<T>> {
        let result = self.cache.put(key.clone(), value);
        self.sync_to_file()?;
        Ok(result)
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        self.cache.get(key).cloned()
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.cache.get_mut(key)
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
                key: k.clone(),
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
//     let mut cache = SyncLruCache::new(NonZeroUsize::new(2).unwrap(), file_path)?;

//     println!("{:?}", cache);

//     cache.put("apple".to_string(), 3);
//     cache.put("banana".to_string(), 2);

//         println!("{:?}", cache);

//     assert_eq!(*cache.get("apple").unwrap(), 3);
//     assert_eq!(*cache.get("banana").unwrap(), 2);
//     assert!(cache.get("pear").is_none());

//     assert_eq!(cache.put("banana".to_string(), 4).unwrap(), Some(2));
//     assert_eq!(cache.put("pear".to_string(), 5).unwrap(), None);

//     println!("{:?}", cache);

//     assert_eq!(*cache.get("pear").unwrap(), 5);
//     assert_eq!(*cache.get("banana").unwrap(), 4);
//     assert!(cache.get("apple").is_none());

//     {
//         let v = cache.get_mut("banana").unwrap();
//         *v = 6;
//     }

//     println!("{:?}", cache);

//     assert_eq!(*cache.get("banana").unwrap(), 6);

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
