use super::PlatformInfo;

pub struct IosPlatform;

impl PlatformInfo for IosPlatform {
    fn name(&self) -> &'static str {
        "ios"
    }
}

pub fn current() -> impl PlatformInfo {
    IosPlatform
}
