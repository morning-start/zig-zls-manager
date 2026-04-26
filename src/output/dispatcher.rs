use serde::Serialize;

use crate::output::json_output;
use crate::utils::error::ZzmError;

/// 输出行抽象 trait
///
/// 统一"数据转换 + 输出调度"逻辑，消除各命令中重复的
/// `if json { json_output } else { table_output }` 模式
pub trait OutputRow: Serialize + Clone {
    /// 转换为表格行数据
    fn to_table_row(&self) -> Vec<String>;

    /// 表头名称
    fn table_headers() -> Vec<&'static str>;
}

/// 统一输出调度：根据 json 标志选择 JSON 或表格输出
///
/// # 参数
/// - `data`: 要输出的行数据
/// - `json`: 是否以 JSON 格式输出
/// - `empty_msg`: 无数据时的提示消息（仅控制台模式显示）
pub fn output_list<T: OutputRow>(data: &[T], json: bool, empty_msg: Option<&str>) {
    if json {
        // JSON 模式：直接序列化输出
        if let Err(e) = json_output::print_json(data) {
            crate::output::console_output::print_error(&format!("JSON 输出失败: {e}"));
        }
    } else if data.is_empty() {
        // 空数据提示
        if let Some(msg) = empty_msg {
            crate::output::console_output::print_info(msg);
        }
    } else {
        // 表格模式：构建并渲染
        render_output_table(data);
    }
}

/// 输出单条数据（用于 current 等只返回一项的场景）
///
/// JSON 模式输出数组，表格模式输出单行
#[allow(dead_code)] // 预留: 单条数据输出场景
pub fn output_single<T: OutputRow>(data: &T, json: bool) {
    if json {
        if let Err(e) = json_output::print_json(data) {
            crate::output::console_output::print_error(&format!("JSON 输出失败: {e}"));
        }
    } else {
        let single = vec![data.clone()];
        render_output_table(&single);
    }
}

/// 输出任意可序列化数据（用于 info/doctor 等非表格场景）
///
/// JSON 模式序列化输出，否则无操作（由调用方自行处理控制台输出）
pub fn output_json_if<T: Serialize>(data: &T, json: bool) -> Result<(), ZzmError> {
    if json {
        json_output::print_json(data)?;
    }
    Ok(())
}

/// 渲染 OutputRow 数据为表格
fn render_output_table<T: OutputRow>(data: &[T]) {
    use tabled::settings::Style;

    let headers = T::table_headers();

    let mut builder = tabled::builder::Builder::default();
    builder.push_record(headers.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    for row in data {
        builder.push_record(row.to_table_row());
    }
    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize)]
    struct TestRow {
        name: String,
        value: i32,
    }

    impl OutputRow for TestRow {
        fn to_table_row(&self) -> Vec<String> {
            vec![self.name.clone(), self.value.to_string()]
        }

        fn table_headers() -> Vec<&'static str> {
            vec!["名称", "值"]
        }
    }

    #[test]
    fn test_output_row_impl() {
        let row = TestRow {
            name: "test".to_string(),
            value: 42,
        };
        assert_eq!(row.to_table_row(), vec!["test", "42"]);
        assert_eq!(TestRow::table_headers(), vec!["名称", "值"]);
    }

    #[test]
    fn test_output_row_serialize() {
        let row = TestRow {
            name: "test".to_string(),
            value: 42,
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"value\""));
    }

    #[test]
    fn test_output_json_if_not_json() {
        let row = TestRow {
            name: "test".to_string(),
            value: 42,
        };
        // json=false 时不输出任何内容，只返回 Ok
        let result = output_json_if(&row, false);
        assert!(result.is_ok());
    }
}
