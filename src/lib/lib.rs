extern crate core;

pub mod http_magic;
pub mod utils;

pub mod config {

    #[cfg(target_os = "windows")]
    pub const SERVER_ROOT: &str = "srv\\www";

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub const SERVER_ROOT: &str = "srv/www";

    pub const MAX_RAND_FILENAME: usize = 200;

    pub const BIND_ADDRESS: &str = "127.0.0.1:1025";
    pub const NOT_FOUND: &str = "404.html";
    pub const CONFLICT: &str = "409.html";
    pub const CREATED: &str = "201.html";
    pub const METHOD_NOT_ALLOWED: &str = "405.html";
}

pub mod control {

    use std::sync::RwLock;
    use std::time::Duration;

    pub const MAX_KEEP_ALIVE_REQUESTS: u16 = 50;
    pub const KEEP_ALIVE_TIMEOUT: u8 = 5;
    pub const CONTROL_THREAD_SLEEP: u8 = 5;

    pub struct ControlStat {
        pub thread_index: usize,
        pub idle_time: Duration,
    }

    impl ControlStat {
        pub fn new() -> RwLock<Self> {
            RwLock::new(ControlStat {
                thread_index: usize::MAX as usize,
                idle_time: Duration::from_secs(0),
            })
        }
        pub fn reset(&mut self) {
            self.thread_index = usize::MAX as usize;
            self.idle_time = Duration::from_secs(0)
        }
    }
}
