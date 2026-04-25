use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// 下载进度条管理器
#[allow(dead_code)] // 预留: 下载进度展示
pub struct DownloadProgress {
    bar: ProgressBar,
}

#[allow(dead_code)] // 预留: 下载进度展示
impl DownloadProgress {
    /// 创建新的下载进度条
    pub fn new(name: &str, total_size: u64) -> Self {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} {msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
                )
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("█▓░"),
        );
        bar.set_message(name.to_string());
        Self { bar }
    }

    /// 更新进度
    pub fn update(&self, downloaded: u64) {
        self.bar.set_position(downloaded);
    }

    /// 完成进度条
    pub fn finish(&self) {
        self.bar.finish_with_message("完成");
    }

    /// 完成并显示成功消息
    pub fn finish_with_message(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }
}

/// 创建一个简单的 spinner（用于不确定进度的操作）
#[allow(dead_code)] // 预留: 长操作 spinner
pub fn create_spinner(msg: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    bar.set_message(msg.to_string());
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

/// 创建一个带进度条的下载任务
pub fn create_download_bar(name: &str, total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("█▓░"),
    );
    bar.set_message(name.to_string());
    bar
}