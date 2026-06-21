use super::style;
use crate::core;
use crate::core::{CoreOutput, Task};
use anyhow::Result;
use eframe::egui::Ui;
use std::sync::Arc;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::{error, warn};

fn get_frame_option() -> Result<eframe::NativeOptions> {
    let icon_bytes = include_bytes!("../../asset/icon.png");
    let image = image::load_from_memory(icon_bytes)?.to_rgba8();
    let (width, height) = image.dimensions();
    let rgba_pixels = image.into_raw();
    let icon_data = egui::IconData {
        rgba: rgba_pixels,
        width,
        height,
    };
    Ok(eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Task Fighter")
            .with_icon(Arc::new(icon_data)),
        ..Default::default()
    })
}

#[tokio::main]
pub async fn start_app() -> Result<()> {
    let native_options = get_frame_option()?;

    eframe::run_native(
        "Task Fighter",
        native_options,
        Box::new(|cc| {
            let app: Box<dyn eframe::App> = Box::new(App::new(&cc.egui_ctx));
            Ok(app)
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {:?}", e))?;

    Ok(())
}

pub enum AppState {
    Default,
    Edit(Task),
    Create,
}

pub struct App {
    pub state: AppState,
    pub core: core::Core,
    pub output: core::CoreOutput,
    pub displayed_tasks: Option<Vec<Task>>,
    pub temp_task: Task,
    pub scan_pattern: String,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        style::setup_style(ctx);
        Self {
            state: AppState::Default,
            core: core::Core::new().unwrap(),
            output: core::CoreOutput::Idle,
            displayed_tasks: None,
            temp_task: Task::default(),
            scan_pattern: String::new(),
        }
    }
}

// 💡 提示いただいたコードが `update` ではなく `ui` になっていたため、
// eframe のライフサイクルに合わせる形で `update` と `CentralPanel` を挟む形に調整しています
impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.poll_background_tasks(ui);
        match self.state {
            AppState::Default => self.default_view(ui, frame),
            AppState::Create => self.create_view(ui, frame),
            AppState::Edit(_) => self.edit_view(ui, frame),
        }
    }
}

impl App {
    fn poll_background_tasks(&mut self, ui: &mut Ui) {
        let next_output = match &mut self.output {
            CoreOutput::Idle => None,

            // データを状態(self.displayed_tasks)に格納する必要がある個別ケース
            CoreOutput::FetchActiveTasks(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("アクティブタスクの取得に失敗しました: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => {
                    ui.spinner();
                    None
                }
                Err(TryRecvError::Closed) => {
                    error!("チャンネルが予期せず閉じられました (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::ScanTasksByFts(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("検索に失敗しました: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => {
                    ui.spinner();
                    None
                }
                Err(TryRecvError::Closed) => {
                    error!("チャンネルが予期せず閉じられました (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },

            // 成功時に共通して Idle に戻るだけのケースを一括処理
            other_output => {
                macro_rules! handle_rx {
                    ($rx:expr, $err_msg:expr) => {
                        match $rx.try_recv() {
                            Ok(Ok(_)) => Some(CoreOutput::Idle),
                            Ok(Err(e)) => {
                                error!("{}: {:?}", $err_msg, e);
                                Some(CoreOutput::Idle)
                            }
                            Err(TryRecvError::Empty) => {
                                ui.spinner();
                                None
                            }
                            Err(TryRecvError::Closed) => {
                                error!("チャンネルが予期せず閉じられました ({})", $err_msg);
                                Some(CoreOutput::Idle)
                            }
                        }
                    };
                }

                match other_output {
                    CoreOutput::InsertTask(rx) => handle_rx!(rx, "タスクの挿入に失敗しました"),
                    CoreOutput::FetchAllTasks(rx) => handle_rx!(rx, "全タスクの取得に失敗しました"),
                    CoreOutput::FetchTaskById(rx) => {
                        handle_rx!(rx, "指定タスクの取得に失敗しました")
                    }
                    CoreOutput::FetchIncompleteTasks(rx) => {
                        handle_rx!(rx, "未完了タスクの取得に失敗しました")
                    }
                    CoreOutput::UpdateTask(rx) => handle_rx!(rx, "タスクの更新に失敗しました"),
                    CoreOutput::MailDaily(rx) => handle_rx!(rx, "デイリーメール送信に失敗しました"),
                    _ => None,
                }
            }
        };

        // 状態の更新を反映
        if let Some(next) = next_output {
            self.output = next;
        }
    }
}
