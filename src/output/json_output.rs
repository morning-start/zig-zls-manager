use serde::Serialize;

use crate::utils::error::ZzmError;

/// 以 JSON 格式输出数据
pub fn print_json<T: Serialize>(data: &T) -> Result<(), ZzmError> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// 以 JSON 格式输出错误信息
#[allow(dead_code)] // 预留: JSON 错误输出
pub fn print_json_error(message: &str) {
    let error = serde_json::json!({
        "error": message
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&error).unwrap_or_default()
    );
}
