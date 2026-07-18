use crate::page::{Page, Pages};
use crate::widget::KanbanArea;
use crate::widget::MenuBar;
use crate::work::Work;
use core::prelude::*;
use core::{TaskFilterFlags, TaskOrderFlags};
use core::{TaskPriority, TaskStatus};
use egui::{Button, Ui, vec2};

// レイアウト用の定数
const SPINNER_SIZE: f32 = 64.0;
const BACK_BUTTON_SIZE: egui::Vec2 = vec2(120.0, 28.0);

pub struct KanbanPage {
    kanban_area: KanbanArea,
    back_page: Pages,
    kanban_ready: bool,
    menu_bar: MenuBar,
}

impl KanbanPage {
    pub fn new() -> Self {
        Self {
            kanban_area: KanbanArea::default(),
            back_page: Pages::Main,
            kanban_ready: false,
            menu_bar: MenuBar::new(),
        }
    }

    fn render_top_panel(&self, ui: &mut Ui) {
        egui::Panel::top("kanban_top_panel").show(ui, |ui| {
            ui.heading(fl!("kanban"));
        });
    }

    fn render_central_panel(&mut self, ui: &mut Ui, work: &mut Work) {
        egui::CentralPanel::default().show(ui, |ui| {
            // ローディング中の表示
            if !work.outputs.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.push_id("main-page-spinner", |ui| {
                            ui.add(egui::Spinner::new().size(SPINNER_SIZE));
                        });
                    },
                );
                return;
            }

            // タスクが取得できたらカンバンエリアにセット
            if !self.kanban_ready {
                if let Some(tasks) = &work.tasks {
                    self.kanban_area.set_tasks(tasks);
                    self.kanban_ready = true;
                }
            }

            // カンバンのメインコンテンツを描画
            self.kanban_area.show(ui, work);
        });
    }

    fn render_bottom_panel(&self, ui: &mut Ui, work: &mut Work) {
        egui::Panel::bottom("kanban_bottom_panel").show(ui, |ui| {
            egui::containers::Sides::new().show(
                ui,
                |_ui| {}, // 左側は空
                |ui| {
                    let back_btn = Button::new(fl!("back")).min_size(BACK_BUTTON_SIZE);
                    if ui.add(back_btn).clicked() {
                        work.next_page = self.back_page;
                    }
                    let create_task_btn =
                        Button::new(fl!("create-task")).min_size(BACK_BUTTON_SIZE);
                    if ui.add(create_task_btn).clicked() {
                        work.next_page = Pages::CreateTask;
                    }
                },
            );
        });
    }

    /// デフォルトのタスク取得フラグを生成するヘルパー
    fn default_fetch_flags() -> (TaskFilterFlags, TaskOrderFlags) {
        let filter_flag = TaskFilterFlags::All & !TaskFilterFlags::Inactive;
        let order_flag = TaskOrderFlags::OrderByPriority
            | TaskOrderFlags::OrderByDueDate
            | TaskOrderFlags::Reversed;
        (filter_flag, order_flag)
    }
}

impl Page for KanbanPage {
    fn on_entry(&mut self, work: &mut Work) {
        self.kanban_area.pop_columns();
        self.kanban_ready = false;
        work.tasks = None;
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        // アプリ起動時など、アイドルかつタスク未取得なら自動フェッチを実行
        if work.outputs.is_empty() && work.tasks.is_none() {
            let (filter_flag, order_flag) = Self::default_fetch_flags();
            work.outputs
                .push(work.core.fetch_all_task(filter_flag, order_flag));
        }
        self.menu_bar.show(ui, work);

        // eguiの原則通り Top -> Bottom -> Central の順でパネルを配置
        self.render_top_panel(ui);
        self.render_bottom_panel(ui, work);
        self.render_central_panel(ui, work);
    }

    fn on_exit(&mut self, work: &mut Work) {
        let mut columns = self.kanban_area.pop_columns();
        let _ = columns[0]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::High;
                task.status = TaskStatus::Pending;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[1]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::Medium;
                task.status = TaskStatus::Pending;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[2]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::Low;
                task.status = TaskStatus::Pending;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();

        let _ = columns[3]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::High;
                task.status = TaskStatus::WorkInProgress;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[4]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::Medium;
                task.status = TaskStatus::WorkInProgress;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[5]
            .iter_mut()
            .map(|task| {
                task.priority = TaskPriority::Low;
                task.status = TaskStatus::WorkInProgress;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[6]
            .iter_mut()
            .map(|task| {
                task.status = TaskStatus::Complete;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();
        let _ = columns[7]
            .iter_mut()
            .map(|task| {
                task.status = TaskStatus::Canceled;
                work.outputs.push(work.core.upsert_task(task));
            })
            .collect::<Vec<_>>();

        /* Request update task list */
        work.tasks = None;
    }
}
