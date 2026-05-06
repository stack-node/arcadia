use super::PlatformInfo;

pub struct WindowsPlatform;

impl PlatformInfo for WindowsPlatform {
    fn name(&self) -> &'static str {
        "windows"
    }
}

pub fn current() -> impl PlatformInfo {
    WindowsPlatform
}
