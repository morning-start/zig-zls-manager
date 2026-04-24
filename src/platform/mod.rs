pub mod trait_def;
pub mod windows;
pub mod macos;
pub mod linux;

// 重新导出核心类型和函数
pub use trait_def::{PlatformTrait, detect_platform, current_target_triple};