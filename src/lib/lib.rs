extern crate core;

pub mod http_magic;
pub mod utils;

pub mod config{
    #[cfg(target_os = "windows")]
    pub const SERVER_ROOT: &str = "srv\\www";

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub const SERVER_ROOT: &str = "srv/www";

    pub const MAX_RAND_FILENAME : usize = 200;
    pub const BIND_ADDRESS: &str = "127.0.0.1:1025";
    pub const NOT_FOUND: &str = "404.html";
    pub const CONFLICT: &str = "409.html";
    pub const CREATED: &str = "201.html";

}



