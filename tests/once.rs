#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod test_once {
    use std::sync::Arc;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
            println!("初始化操作只会执行一次");
        });
    }

    #[test]
    fn test_initialize() {
        initialize();
        initialize();
    }

    // 带返回值的一次初始化
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    static COUNTER_INIT: Once = Once::new();
    fn get_counter() -> usize {
        COUNTER_INIT.call_once(|| {
            println!("Counter initialized");
            COUNTER.store(42, Ordering::SeqCst);
        });
        COUNTER.load(Ordering::SeqCst)
    }

    #[test]
    fn test_get_counter() {
        assert_eq!(get_counter(), 42);
        assert_eq!(get_counter(), 42);
    }

    //实现一个单例模式
    use std::sync::Mutex;

    #[derive(Clone, Debug)]
    struct Database {
        url: String,
        _connection_pool: Vec<String>,
    }

    impl Database {
        fn new(url: &str) -> Database {
            Self {
                url: url.to_string(),
                _connection_pool: Vec::new(),
            }
        }

        fn query(self, sql: &str) -> Vec<String> {
            // 模拟查询操作
            vec![format!("Result for: {}", sql)]
        }
    }

    static mut INSTANCE: Option<Mutex<Database>> = None;
    static DATABASE_INIT: Once = Once::new();
    fn get_database() -> &'static Database {
        unsafe {
            DATABASE_INIT.call_once(|| {
                let db = Database::new("sqlite://example.db");
                INSTANCE = Some(Mutex::new(db));
            });

            INSTANCE
                .as_ref()
                .expect("Database has not been initialized!")
                .lock()
                .map(|guard| {
                    // 将 MutexGuard 转换为静态引用
                    Box::leak(Box::new((*guard).clone()))
                })
                .unwrap_or_else(|_| {
                    panic!("Failed to acquire the lock for the database");
                })
        }
    }

    #[test]
    fn test_get_database() {
        let db1 = get_database();
        let db2 = get_database();
        assert_eq!(db1.url, db2.url);
        assert_eq!(
            db1.clone().query("SELECT * FROM users"),
            db2.clone().query("SELECT * FROM users")
        );
    }

    //更好的方式

    lazy_static! {
        static ref DATABASE: Arc<Mutex<Database>> =
            Arc::new(Mutex::new(Database::new("sqlite://example.db")));
    }

    fn get_database2() -> Arc<Mutex<Database>> {
        Arc::clone(&DATABASE)
    }

    #[test]
    fn test_get_database2() {}
}

#[cfg(test)]
mod test_config {

    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::Once;

    struct Config {
        settings: HashMap<String, String>,
    }

    impl Config {
        fn new() -> Config {
            Config {
                settings: HashMap::new(),
            }
        }

        fn load_from_file(&mut self, _filename: &str) {
            self.settings
                .insert("host".to_string(), "localhost".to_string());
            self.settings.insert("port".to_string(), "8080".to_string());
        }
    }

    static mut CONFIG: Option<Mutex<Config>> = None;
    static INIT: Once = Once::new();

    fn get_config() -> &'static Mutex<Config> {
        unsafe {
            INIT.call_once(|| {
                let mut config = Config::new();
                config.load_from_file("config.ini");
                CONFIG = Some(Mutex::new(config));
            });
            CONFIG.as_ref().unwrap()
        }
    }

    #[test]
    fn test_get_config() {
        let config_mutex = get_config();
        let config = config_mutex.lock().unwrap();
        assert_eq!(config.settings.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.settings.get("port"), Some(&"8080".to_string()));
    }
}

#[cfg(test)]
mod test_threadpool {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::mpsc::Receiver;
    use std::sync::mpsc::Sender;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::Once;
    use std::thread;

    use std::sync::mpsc::channel;
    use std::time::Duration;

    type Job = Box<dyn FnOnce() + Send + 'static>;

    struct Worker {
        _id: usize,
        _thread: Option<thread::JoinHandle<()>>,
    }

    struct ThreadPool {
        _workers: Vec<Worker>,
        sender: Sender<Job>,
    }

    static mut THREAD_POOL: Option<ThreadPool> = None;
    static INIT: Once = Once::new();

    impl ThreadPool {
        fn new(size: usize) -> Self {
            let (sender, receiver) = channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool {
                _workers: workers,
                sender: sender,
            }
        }

        fn get_instance() -> &'static ThreadPool {
            unsafe {
                INIT.call_once(|| {
                    THREAD_POOL = Some(ThreadPool::new(4));
                });
                THREAD_POOL.as_ref().unwrap()
            }
        }
    }

    impl Worker {
        fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job", id);
                job();
            });

            Worker {
                _id: id,
                _thread: Some(thread),
            }
        }
    }

    #[test]
    fn test_thread_pool_concurrent_execution() {
        const TASK_COUNT: usize = 100;

        // Atomic counter to track completed tasks
        let counter = Arc::new(AtomicUsize::new(0));

        // Initialize thread pool
        let pool = ThreadPool::get_instance();

        for _ in 0..TASK_COUNT {
            let counter_clone = Arc::clone(&counter);
            pool.sender
                .send(Box::new(move || {
                    // Simulate task work with a sleep
                    thread::sleep(Duration::from_millis(10));
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                }))
                .unwrap();
        }

        // Wait for tasks to complete
        thread::sleep(Duration::from_secs(2));

        // Verify all tasks were completed
        assert_eq!(
            counter.load(Ordering::SeqCst),
            TASK_COUNT,
            "Not all tasks completed"
        );
    }
}
