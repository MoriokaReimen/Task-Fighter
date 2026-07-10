use super::style;
use crate::widget::Graph;
use crate::widget::SearchConditionModal;
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::work::Work;
use anyhow::Result;
use core::{CoreOutput, Task, TryRecvError};
use eframe::egui::Ui;
use tracing::{error, warn};

/// Generates window configuration and initializes the application icon.
fn get_frame_option() -> Result<eframe::NativeOptions> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)?.to_rgba8();
    let (width, height) = image.dimensions();
    let rgba_pixels = image.into_raw();

    let icon_data = egui::IconData {
        rgba: rgba_pixels,
        width,
        height,
    };
    let initial_size = egui::vec2(1024.0, 768.0);
    Ok(eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Task Fighter")
            .with_icon(icon_data)
            .with_inner_size(initial_size)
            .with_min_inner_size(initial_size),
        ..Default::default()
    })
}

/// Main entry point for launching the native GUI application.
// 【修正2】async を削除して同期関数に変更
pub fn start_app() -> Result<()> {
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

/// Represents the current navigation/view state of the UI.
pub enum AppState {
    Default,
    EditTask,
    CreateTask,
    Graph,
    Time,
}

/// Main application state holder.
pub struct App {
    pub state: AppState,
    pub core: core::Core,
    pub work: Work,

    pub output: core::CoreOutput,
    /* Start of Included to work */
    pub displayed_tasks: Option<Vec<Task>>,
    pub plot_data: Option<Vec<(i32, i32, i32, i32)>>,
    pub temp_task: Task,
    pub start_time: jiff::Zoned,
    /* End of included to work */
    pub yes_no_cancel_modal: YesNoCancelModal,
    pub yes_no_modal: YesNoModal,
    pub warning_modal: WarningModal,
    pub search_condition_modal: SearchConditionModal,
    pub graph: Graph,
}

impl App {
    /// Initializes application state and applies global UI styling.
    fn new(ctx: &egui::Context) -> Self {
        style::setup_style(ctx);
        Self {
            state: AppState::Default,
            core: core::Core::new().unwrap(),
            work: Work::new(),

            output: core::CoreOutput::Idle,
            displayed_tasks: None,
            plot_data: None,
            temp_task: Task::default(),
            start_time: jiff::Zoned::now(),

            yes_no_cancel_modal: YesNoCancelModal::new("yes_no_cancel"),
            yes_no_modal: YesNoModal::new("yes_no"),
            warning_modal: WarningModal::new("warning"),
            search_condition_modal: SearchConditionModal::new("search_condition"),
            graph: Graph::new(),
        }
    }
}

impl eframe::App for App {
    /// Main UI update loop called on every frame render.
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        style::set_theme(ui.ctx());
        self.poll_background_tasks();

        // Render the appropriate view based on the current application state
        match self.state {
            AppState::Default => self.default_page(ui, frame),
            AppState::CreateTask => self.create_task_page(ui, frame),
            AppState::EditTask => self.edit_task_page(ui, frame),
            AppState::Graph => self.graph_page(ui, frame),
            AppState::Time => self.time_page(ui, frame),
        }
    }
}

impl App {
    /// Non-blocking check for responses from async background tasks.
    fn poll_background_tasks(&mut self) {
        let next_output = match &mut self.output {
            CoreOutput::Idle => None,

            // Handle cases where tasks data needs to be saved into the UI state (`self.displayed_tasks`)
            CoreOutput::FetchAllTask(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("Failed to fetch active tasks: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                // 【修正3】TryRecvError::Closed を Disconnected に変更（以下すべて同様）
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::SearchTask(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("Search query failed: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (ScanTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::PlotData(rx) => match rx.try_recv() {
                Ok(Ok(data)) => {
                    self.plot_data = Some(data);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("Plot Data failed: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (plot_data)");
                    Some(CoreOutput::Idle)
                }
            },

            // Batch handle simple tasks that just transition back to Idle upon completion
            other_output => {
                macro_rules! handle_rx {
                    ($rx:expr, $err_msg:expr) => {
                        match $rx.try_recv() {
                            Ok(Ok(_)) => Some(CoreOutput::Idle),
                            Ok(Err(e)) => {
                                error!("{}: {:?}", $err_msg, e);
                                Some(CoreOutput::Idle)
                            }
                            Err(TryRecvError::Empty) => None,
                            Err(TryRecvError::Closed) => {
                                error!("Channel disconnected unexpectedly ({})", $err_msg);
                                Some(CoreOutput::Idle)
                            }
                        }
                    };
                }

                match other_output {
                    CoreOutput::InsertTask(rx) => handle_rx!(rx, "Failed to insert task"),
                    CoreOutput::UpsertTask(rx) => handle_rx!(rx, "Failed to insert task"),
                    CoreOutput::FetchAllTask(rx) => handle_rx!(rx, "Failed to fetch all tasks"),
                    CoreOutput::FetchOneTask(rx) => handle_rx!(rx, "Failed to fetch task by ID"),
                    CoreOutput::UpdateTask(rx) => handle_rx!(rx, "Failed to update task"),
                    CoreOutput::MailDaily(rx) => handle_rx!(rx, "Failed to send daily report mail"),
                    _ => None,
                }
            }
        };

        // Apply state transition if a new output state is determined
        if let Some(next) = next_output {
            self.output = next;
        }
    }
}
