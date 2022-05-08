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

//     pub struct ConnectionStats {
//         num_requests: u16,
//         idle_since_sw: stopwatch::Stopwatch,
//     }
//     impl ConnectionStats {
//         pub fn self_life_check(&self) -> bool {
//             // true if should die
//             self.num_requests == MAX_KEEP_ALIVE_REQUESTS
//                 || self.idle_since_sw.elapsed().as_secs() == KEEP_ALIVE_TIMEOUT as u64
//         }
//     }
//
//     impl Default for ConnectionStats {
//         fn default() -> Self {
//             ConnectionStats {
//                 num_requests: 0,
//                 idle_since_sw: stopwatch::Stopwatch::new(),
//             }
//         }
//     }
//     impl ConnectionStats {
//         pub fn reset(&mut self) {
//             self.num_requests = 0;
//             self.idle_since_sw.reset();
//         }
//     }
//
//     pub struct ThreadStats {
//         processing: bool,
//         stats: ConnectionStats,
//         pub verdict: AtomicBool, // true if should die
//     }
//     impl ThreadStats {
//         pub fn must_die(&self) -> bool {
//             self.verdict.load(Ordering::Relaxed) || self.stats.self_life_check()
//         }
//         pub fn die(&mut self) {
//             self.verdict.store(false, Ordering::Relaxed);
//             self.processing = false;
//             self.stats = ConnectionStats::default();
//         }
//         pub fn restart_timer(&mut self) {
//             self.stats.idle_since_sw.restart();
//         }
//         pub fn increment_requests(&mut self) {
//             self.stats.num_requests += 1;
//         }
//         pub fn set_processing(&mut self, processing: bool) {
//             self.processing = processing;
//         }
//         pub fn time_since_idle(&self) -> i64 {
//             self.stats.idle_since_sw.elapsed_ms()
//         }
//     }
//     impl Default for ThreadStats {
//         fn default() -> Self {
//             ThreadStats {
//                 processing: false,
//                 stats: ConnectionStats::default(),
//                 verdict: AtomicBool::new(false),
//             }
//         }
//     }
// }
