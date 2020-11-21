#[allow(non_camel_case_types)]

pub use libc2::*;

#[cfg(target_arch = "wasm32")]
pub use std::os::raw::*;
#[cfg(target_arch = "wasm32")]
pub type time_t = i64;
#[cfg(target_arch = "wasm32")]
pub type suseconds_t = i32;
#[cfg(target_arch = "wasm32")]
#[repr(C)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}
