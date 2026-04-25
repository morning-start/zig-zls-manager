use console::style;
use std::cell::RefCell;

thread_local! {
    static NO_COLOR: RefCell<bool> = const { RefCell::new(false) };
}

/// 设置全局 no_color 标志
pub fn set_no_color(no_color: bool) {
    NO_COLOR.with(|flag| {
        *flag.borrow_mut() = no_color;
    });
}

/// 检查是否禁用彩色
fn is_no_color() -> bool {
    NO_COLOR.with(|flag| *flag.borrow())
}

/// 打印成功消息（绿色 ✓）
pub fn print_success(msg: &str) {
    if is_no_color() {
        println!("[OK] {}", msg);
    } else {
        println!("{} {}", style("✓").green().bold(), msg);
    }
}

/// 打印警告消息（黄色 ⚠）
pub fn print_warning(msg: &str) {
    if is_no_color() {
        println!("[WARN] {}", msg);
    } else {
        println!("{} {}", style("⚠").yellow().bold(), msg);
    }
}

/// 打印错误消息（红色 ✗）
pub fn print_error(msg: &str) {
    if is_no_color() {
        eprintln!("[ERROR] {}", msg);
    } else {
        eprintln!("{} {}", style("✗").red().bold(), msg);
    }
}

/// 打印信息消息（蓝色 ℹ）
pub fn print_info(msg: &str) {
    if is_no_color() {
        println!("[INFO] {}", msg);
    } else {
        println!("{} {}", style("ℹ").blue(), msg);
    }
}

/// 打印步骤消息（带编号的操作步骤）
pub fn print_step(step: usize, total: usize, msg: &str) {
    if is_no_color() {
        println!("[{}/{}] {}", step, total, msg);
    } else {
        println!("{} {}/{} {}", style("▸").cyan(), step, total, msg);
    }
}

/// 打印带标题的分隔线
pub fn print_header(title: &str) {
    if is_no_color() {
        println!("{}", title);
        println!("{}", "─".repeat(50));
    } else {
        println!("{}", style(title).bold());
        println!("{}", style("─".repeat(50)).dim());
    }
}
