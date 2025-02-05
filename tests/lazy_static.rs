use lazy_static::lazy_static;
use std::collections::HashMap;

#[cfg(test)]
mod test_lazy_static {

    use super::*;
    lazy_static! {
        static ref HASHMAP: HashMap<u32, String> = {
            let mut m = HashMap::new();
            m.insert(1, "Alice".to_string());
            m.insert(2, "Bob".to_string());
            m
        };
    }

    #[test]
    fn test_get_value() {
        assert_eq!(HASHMAP.get(&1), Some(&"Alice".to_string()));
        assert_eq!(HASHMAP.get(&2), Some(&"Bob".to_string()));
        assert_eq!(HASHMAP.get(&3), None);
    }
}

#[cfg(test)]
mod test_config {
    use super::*;
    use std::sync::Mutex;

    lazy_static! {
        static ref CONFIG: Mutex<HashMap<String, String>> = {
            let mut m = HashMap::new();
            m.insert("host".to_string(), "localhost".to_string());
            m.insert("port".to_string(), "8080".to_string());
            Mutex::new(m)
        };
    }

    fn update_config(key: &str, value: &str) {
        let mut config = CONFIG.lock().unwrap();
        config.insert(key.to_string(), value.to_string());
    }

    fn get_config(key: &str) -> Option<String> {
        CONFIG.lock().unwrap().get(key).cloned()
    }

    #[test]
    fn test_update_config() {
        update_config("host", "127.0.0.1");
        assert_eq!(get_config("host"), Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_get_config() {
        assert_eq!(get_config("port"), Some("8080".to_string()));
        assert_eq!(get_config("unknown"), None);
    }
}

#[cfg(test)]
mod test_lock {
    use super::*;
    use std::sync::RwLock;

    lazy_static! {
        static ref CACHE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    }

    fn read_cache(key: &str) -> Option<String> {
        let cache = CACHE.read().unwrap();
        cache.get(key).cloned()
    }

    fn write_cache(key: &str, value: &str) {
        let mut cache = CACHE.write().unwrap();
        cache.insert(key.to_string(), value.to_string());
    }

    #[test]
    fn test_read_cache() {
        write_cache("foo", "bar");
        assert_eq!(read_cache("foo"), Some("bar".to_string()));
    }

    //性能压测
    #[test]
    fn test_performance() {
        let num_threads = 100;
        let num_keys = 10000;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                std::thread::spawn(move || {
                    for key in 0..num_keys {
                        write_cache(&format!("key{}", key), &format!("value{}", key));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(read_cache("key1"), Some("value1".to_string()));
        assert_eq!(read_cache("key9999"), Some("value9999".to_string()));
        //多线程get
        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                std::thread::spawn(move || {
                    for key in 0..num_keys {
                        read_cache(&format!("key{}", key));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
