pub trait PlatformInfo {
    fn name(&self) -> &'static str;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::current;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::current;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::current;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unknown;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use unknown::current;
