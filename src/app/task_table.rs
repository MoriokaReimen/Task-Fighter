use crate::driver::{Priority, Task, TaskStatus};
use crate::fl;
use egui::{Color32, Label, Response, Ui};
use egui_extras::{Column, TableBuilder};

#[derive(Debug)]
pub struct TaskTable<'a> {
    tasks: &'a [Task],
    pub clicked: bool, // 外部から読み取れるよう pub に（または getter経由）
    pub clicked_task: Option<Task>,
}

impl<'a> TaskTable<'a> {
    pub fn new(tasks: &'a [Task]) -> Self {
        Self {
            tasks,
            clicked: false,
            clicked_task: None,
        }
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn clicked_task(&self) -> Option<Task> {
        self.clicked_task.clone()
    }

    // egui で状態 (clicked) を呼び出し元に持ち帰るための推奨パターン（&mut self を取る描画関数）
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let _clicked = false;
        let _clicked_index = 0;

        let inner_response = ui.scope(|ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .column(Column::exact(40.0)) // Checkbox
                .column(Column::remainder()) // title (トリミングされるためremainderが活きます)
                .column(Column::exact(80.0)) // priority
                .column(Column::exact(90.0)) // Due Date (日付表示に合わせて少し広めに調整)
                .column(Column::exact(100.0)) // Progress
                .column(Column::exact(60.0)); // Edit Button

            table
                .header(28.0, |mut header| {
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("done"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("title"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("priority"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("due_date"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("progress"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new(fl!("edit"))
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                })
                .body(|body| {
                    let row_height = 28.0;
                    body.rows(row_height, self.tasks.len(), |mut row| {
                        let row_index = row.index();
                        let task = &self.tasks[row_index];

                        row.col(|ui| {
                            let mut is_done = task.status == TaskStatus::Complete;
                            ui.add_enabled(false, egui::Checkbox::new(&mut is_done, ""));
                        });

                        // ✨ 変更ポイント: タイトルが長い場合に「...」にする
                        row.col(|ui| {
                            ui.add(Label::new(&task.title).truncate());
                        });

                        row.col(|ui| match task.priority {
                            Priority::High => {
                                ui.label(
                                    egui::RichText::new(fl!("high"))
                                        .color(Color32::from_rgb(255, 60, 60)),
                                );
                            }
                            Priority::Medium => {
                                ui.label(
                                    egui::RichText::new(fl!("medium"))
                                        .color(Color32::from_rgb(255, 215, 0)),
                                );
                            }
                            Priority::Low => {
                                ui.label(
                                    egui::RichText::new(fl!("low"))
                                        .color(Color32::from_rgb(60, 255, 60)),
                                );
                            }
                        });
                        row.col(|ui| {
                            // 仮定されている型に応じて適宜修正してください
                            ui.label(task.due_date.strftime("%Y/%m/%d").to_string());
                        });
                        row.col(|ui| {
                            let progress_fraction = task.progress / 100.0;
                            let status_icon = match task.status {
                                TaskStatus::Pending => "⏳",
                                TaskStatus::WorkInProgress => "🏃",
                                TaskStatus::Complete => "✅",
                                TaskStatus::Canceled => "🚫 ",
                            };
                            ui.add(
                                egui::ProgressBar::new(progress_fraction)
                                    .show_percentage()
                                    .text(format!("{:.0}% {}", task.progress, status_icon)),
                            );
                        });
                        row.col(|ui| {
                            if ui.button(fl!("edit")).clicked() {
                                self.clicked = true;
                                self.clicked_task = Some(task.clone());
                            }
                        });
                    });
                });
        });

        inner_response.response
    }
}
