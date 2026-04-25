pub mod linux;
pub mod macos;
pub mod trait_def;
pub mod windows;

// 重新导出核心类型和函数
pub use trait_def::{PlatformTrait, current_target_triple, detect_platform, parse_target_triple};
