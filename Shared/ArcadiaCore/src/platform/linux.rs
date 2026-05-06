use super::PlatformInfo;

pub struct LinuxPlatform;

impl PlatformInfo for LinuxPlatform {
    fn name(&self) -> &'static str {
        "linux"
    }
}

pub fn current() -> impl PlatformInfo {
    LinuxPlatform
}
