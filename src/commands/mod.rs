mod processor;

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub use processor::{CommandProcessor, ExecutionStrategy};

pub struct CommandCache {
    cache: HashMap<String, (i32, String, Instant)>,
    ttl: Duration,
}

impl CommandCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn get(&self, command: &str) -> Option<(i32, String)> {
        if let Some((code, output, timestamp)) = self.cache.get(command) {
            if timestamp.elapsed() < self.ttl {
                return Some((*code, output.clone()));
            }
        }
        None
    }
    
    pub fn set(&mut self, command: String, result: (i32, String)) {
        self.cache.insert(command, (result.0, result.1, Instant::now()));
    }
    
    pub fn clear_expired(&mut self) {
        self.cache.retain(|_, (_, _, timestamp)| {
            timestamp.elapsed() < self.ttl
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_command_cache() {
        let mut cache = CommandCache::new();
        
        // Test setting and getting
        cache.set("test".to_string(), (0, "output".to_string()));
        assert_eq!(
            cache.get("test"),
            Some((0, "output".to_string()))
        );
        
        // Test expiration
        let mut cache = CommandCache::new();
        cache.ttl = Duration::from_millis(1);
        cache.set("test".to_string(), (0, "output".to_string()));
        sleep(Duration::from_millis(2));
        assert_eq!(cache.get("test"), None);
        
        // Test clear_expired
        let mut cache = CommandCache::new();
        cache.ttl = Duration::from_millis(1);
        cache.set("test1".to_string(), (0, "output1".to_string()));
        cache.set("test2".to_string(), (0, "output2".to_string()));
        sleep(Duration::from_millis(2));
        cache.clear_expired();
        assert!(cache.cache.is_empty());
    }
}