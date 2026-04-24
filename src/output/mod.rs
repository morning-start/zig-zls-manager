pub mod console_output;
pub mod json_output;
pub mod table_output;
pub mod progress;

// 重新导出常用函数
pub use console_output::{print_success, print_warning, print_error, print_info, print_step};
pub use json_output::print_json;
pub use progress::DownloadProgress;