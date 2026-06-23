use super::style;
use crate::core::{self, CoreOutput, Task};
use anyhow::Result;
use eframe::egui::Ui;
use std::sync::Arc;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::{error, warn};

/// Generates window configuration and initializes the application icon.
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
    let initial_size = egui::vec2(800.0, 600.0);
    Ok(eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Task Fighter")
            .with_icon(Arc::new(icon_data))
            .with_inner_size(initial_size)
            .with_min_inner_size(initial_size),
        ..Default::default()
    })
}

/// Main entry point for launching the native GUI application.
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

/// Represents the current navigation/view state of the UI.
pub enum AppState {
    Default,
    Edit,
    Create,
}

/// Main application state holder.
pub struct App {
    pub state: AppState,
    pub core: core::Core,
    pub output: core::CoreOutput,
    pub displayed_tasks: Option<Vec<Task>>,
    pub temp_task: Task,
    pub scan_pattern: String,
}

impl App {
    /// Initializes application state and applies global UI styling.
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

impl eframe::App for App {
    /// Main UI update loop called on every frame render.
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        style::set_theme(ui.ctx());
        self.poll_background_tasks(ui);

        // Render the appropriate view based on the current application state
        match self.state {
            AppState::Default => self.default_view(ui, frame),
            AppState::Create => self.create_view(ui, frame),
            AppState::Edit => self.edit_view(ui, frame),
        }
    }
}

impl App {
    /// Non-blocking check for responses from async background tasks.
    fn poll_background_tasks(&mut self, ui: &mut Ui) {
        let next_output = match &mut self.output {
            CoreOutput::Idle => None,

            // Handle cases where tasks data needs to be saved into the UI state (`self.displayed_tasks`)
            CoreOutput::FetchActiveTasks(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("Failed to fetch active tasks: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => {
                    ui.spinner();
                    None
                }
                Err(TryRecvError::Closed) => {
                    error!("Channel closed unexpectedly (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::ScanTasksByFts(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.displayed_tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("Search query failed: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => {
                    ui.spinner();
                    None
                }
                Err(TryRecvError::Closed) => {
                    error!("Channel closed unexpectedly (ScanTasksByFts)");
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
                            Err(TryRecvError::Empty) => {
                                ui.spinner();
                                None
                            }
                            Err(TryRecvError::Closed) => {
                                error!("Channel closed unexpectedly ({})", $err_msg);
                                Some(CoreOutput::Idle)
                            }
                        }
                    };
                }

                match other_output {
                    CoreOutput::InsertTask(rx) => handle_rx!(rx, "Failed to insert task"),
                    CoreOutput::FetchAllTasks(rx) => handle_rx!(rx, "Failed to fetch all tasks"),
                    CoreOutput::FetchTaskById(rx) => handle_rx!(rx, "Failed to fetch task by ID"),
                    CoreOutput::FetchIncompleteTasks(rx) => {
                        handle_rx!(rx, "Failed to fetch incomplete tasks")
                    }
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
