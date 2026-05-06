use super::PlatformInfo;

pub struct UnknownPlatform;

impl PlatformInfo for UnknownPlatform {
    fn name(&self) -> &'static str {
        "unknown"
    }
}

pub fn current() -> impl PlatformInfo {
    UnknownPlatform
}
