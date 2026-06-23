use crate::driver::{Priority, Task};
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
                .column(Column::exact(30.0)) // id
                .column(Column::remainder()) // title (トリミングされるためremainderが活きます)
                .column(Column::exact(60.0)) // priority
                .column(Column::exact(70.0)) // Due Date (日付表示に合わせて少し広めに調整)
                .column(Column::exact(100.0)) // Progress
                .column(Column::exact(60.0)); // Edit Button

            table
                .header(28.0, |mut header| {
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("ID").color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("Title")
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("Priority")
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("Due Date")
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("Progress")
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                    header.col(|ui| {
                        ui.strong(
                            egui::RichText::new("Edit").color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                    });
                })
                .body(|body| {
                    let row_height = 28.0;
                    body.rows(row_height, self.tasks.len(), |mut row| {
                        let row_index = row.index();
                        let task = &self.tasks[row_index];

                        row.col(|ui| {
                            ui.label(format!("{}", task.id));
                        });

                        // ✨ 変更ポイント: タイトルが長い場合に「...」にする
                        row.col(|ui| {
                            ui.add(Label::new(&task.title).truncate());
                        });

                        row.col(|ui| match task.priority {
                            Priority::High => {
                                ui.label(
                                    egui::RichText::new("High")
                                        .color(Color32::from_rgb(255, 60, 60)),
                                );
                            }
                            Priority::Medium => {
                                ui.label(
                                    egui::RichText::new("Medium")
                                        .color(Color32::from_rgb(255, 215, 0)),
                                );
                            }
                            Priority::Low => {
                                ui.label(
                                    egui::RichText::new("Low")
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
                            ui.add(
                                egui::ProgressBar::new(progress_fraction)
                                    .show_percentage()
                                    .text(format!("{:.1}% Done", task.progress)),
                            );
                        });
                        row.col(|ui| {
                            if ui.button("✏ Edit").clicked() {
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
