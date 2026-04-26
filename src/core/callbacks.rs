use crate::output::console_output;

/// 步骤进度回调类型
pub type StepFn = dyn Fn(usize, usize, &str);
/// 消息回调类型
pub type MessageFn = dyn Fn(&str);

/// 安装流程回调集合
///
/// 解耦 Core 层与具体输出实现，使 `--json` 输出不被污染，
/// 并为 TUI/GUI 留出扩展空间。
///
/// Commands 层负责注入具体实现（控制台输出 / JSON 事件收集 / 空回调）。
pub struct InstallCallbacks {
    /// 步骤进度回调: (step, total, message)
    pub on_step: Box<StepFn>,
    /// 成功消息回调
    pub on_success: Box<MessageFn>,
    /// 警告消息回调
    pub on_warning: Box<MessageFn>,
    /// 信息消息回调
    pub on_info: Box<MessageFn>,
}

impl InstallCallbacks {
    /// 控制台输出回调（默认）
    ///
    /// 使用 `console_output` 模块的标准彩色输出
    pub fn console() -> Self {
        Self {
            on_step: Box::new(console_output::print_step),
            on_success: Box::new(console_output::print_success),
            on_warning: Box::new(console_output::print_warning),
            on_info: Box::new(console_output::print_info),
        }
    }

    /// JSON 模式回调（静默）
    ///
    /// 安装过程中的步骤/进度信息不输出到终端，
    /// 避免污染 JSON 输出流
    pub fn silent() -> Self {
        Self {
            on_step: Box::new(|_, _, _| {}),
            on_success: Box::new(|_| {}),
            on_warning: Box::new(|_| {}),
            on_info: Box::new(|_| {}),
        }
    }
}

impl std::fmt::Debug for InstallCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstallCallbacks")
            .field("on_step", &"Fn(usize, usize, &str)")
            .field("on_success", &"Fn(&str)")
            .field("on_warning", &"Fn(&str)")
            .field("on_info", &"Fn(&str)")
            .finish()
    }
}
