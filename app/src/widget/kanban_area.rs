use crate::page::Pages;
use crate::work::Work;
use core::{Task, TaskPriority, TaskStatus};
use egui::{Color32, Frame, Id, Response, Stroke, Ui, vec2};
use tracing::{info, warn};

pub struct KanbanArea {
    columns: Vec<Vec<Task>>,
}

impl Default for KanbanArea {
    fn default() -> Self {
        Self {
            columns: vec![vec![]; 8],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}

impl KanbanArea {
    pub(crate) fn set_tasks(&mut self, tasks: &Vec<Task>) {
        let _ = tasks
            .iter()
            .map(|task| match (task.priority, task.status) {
                (TaskPriority::High, TaskStatus::Pending) => self.columns[0].push(task.clone()),
                (TaskPriority::Medium, TaskStatus::Pending) => self.columns[1].push(task.clone()),
                (TaskPriority::Low, TaskStatus::Pending) => self.columns[2].push(task.clone()),
                (TaskPriority::High, TaskStatus::WorkInProgress) => {
                    self.columns[3].push(task.clone())
                }
                (TaskPriority::Medium, TaskStatus::WorkInProgress) => {
                    self.columns[4].push(task.clone())
                }
                (TaskPriority::Low, TaskStatus::WorkInProgress) => {
                    self.columns[5].push(task.clone())
                }
                (_, TaskStatus::Complete) => self.columns[6].push(task.clone()),
                (_, TaskStatus::Canceled) => self.columns[7].push(task.clone()),
                _ => warn!("Undefined priority and status"),
            })
            .collect::<Vec<_>>();
    }

    pub(crate) fn pop_columns(&mut self) -> Vec<Vec<Task>> {
        let empty_columns = vec![vec![]; 8];
        std::mem::replace(&mut self.columns, empty_columns)
    }

    /// 【新規】カードがダブルクリックされたときに呼ばれるハンドラ関数
    fn on_card_double_click(&mut self, loc: Location, work: &mut Work) {
        if let Some(task) = self.columns[loc.col].get(loc.row) {
            info!(
                "Task card double clicked: '{}' (Column: {}, Row: {})",
                task.title, loc.col, loc.row
            );
            work.task = task.clone();
            work.next_page = Pages::EditTask;
        }
    }

    /// マスの中にある各カード（ドラッグソース）の描画
    /// 戻り値: (Response, double_clicked: bool)
    fn render_card(&self, ui: &mut Ui, task: &Task, loc: Location) -> (Response, bool) {
        let item_id = Id::new(("kanban_item", loc.col, loc.row));
        let available_width = ui.available_width();
        let mut is_double_clicked = false;

        let dnd_res = ui.dnd_drag_source(item_id, loc, |ui| {
            let card_frame = Frame::window(ui.style()).inner_margin(egui::Margin::symmetric(2, 2));

            card_frame.show(ui, |ui| {
                let desired_size = vec2(available_width, 18.0);
                let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true);

                ui.allocate_ui_with_layout(desired_size, layout, |ui| {
                    ui.set_max_width(available_width);
                    ui.add(egui::Label::new(task.title.clone()).truncate());
                });
            });
        });

        // 【ここを修正：イベント消費に邪魔されない幾何学的判定】
        // 1. このフレーム中に、画面のどこかで左マウスボタンのダブルクリックが発生したか？
        if ui.input(|i| {
            i.pointer
                .button_double_clicked(egui::PointerButton::Primary)
        }) {
            // 2. その時のマウスカーソルの絶対座標を取得
            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                // 3. カードが描画された物理的な矩形（rect）の中に、カーソルが含まれているか？
                if dnd_res.response.rect.contains(pointer_pos) {
                    is_double_clicked = true;
                }
            }
        }

        (dnd_res.response, is_double_clicked)
    }

    /// ドロップされたアイテムのデータ移動ロジック
    fn move_item(&mut self, from: Location, mut to: Location) {
        if from.col == to.col {
            to.row -= (from.row < to.row) as usize;
        }

        if let Some(item) = self.columns[from.col].get(from.row).cloned() {
            self.columns[from.col].remove(from.row);
            let column = &mut self.columns[to.col];
            to.row = to.row.min(column.len());
            column.insert(to.row, item);
        }
    }

    /// メインのエントリポイント
    pub(crate) fn show(&mut self, ui: &mut Ui, work: &mut Work) {
        let mut from = None;
        let mut to = None;
        let mut double_clicked_loc = None; // ダブルクリックされた位置を記録する変数

        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);

        egui_extras::StripBuilder::new(ui)
            .sizes(egui_extras::Size::remainder(), 3)
            .horizontal(|mut col_strip| {
                // 1列目 (縦3分割)
                col_strip.cell(|ui| {
                    egui_extras::StripBuilder::new(ui)
                        .sizes(egui_extras::Size::remainder(), 3)
                        .vertical(|mut row_strip| {
                            for row_idx in 0..3 {
                                row_strip.cell(|ui| {
                                    self.render_area_cell(
                                        ui,
                                        row_idx,
                                        &mut from,
                                        &mut to,
                                        &mut double_clicked_loc,
                                    );
                                });
                            }
                        });
                });

                // 2列目 (縦3分割)
                col_strip.cell(|ui| {
                    egui_extras::StripBuilder::new(ui)
                        .sizes(egui_extras::Size::remainder(), 3)
                        .vertical(|mut row_strip| {
                            for row_idx in 0..3 {
                                row_strip.cell(|ui| {
                                    self.render_area_cell(
                                        ui,
                                        3 + row_idx,
                                        &mut from,
                                        &mut to,
                                        &mut double_clicked_loc,
                                    );
                                });
                            }
                        });
                });

                // 3列目 (縦2分割: 完了・キャンセル)
                col_strip.cell(|ui| {
                    egui_extras::StripBuilder::new(ui)
                        .sizes(egui_extras::Size::remainder(), 2)
                        .vertical(|mut row_strip| {
                            for row_idx in 0..2 {
                                row_strip.cell(|ui| {
                                    self.render_area_cell(
                                        ui,
                                        6 + row_idx,
                                        &mut from,
                                        &mut to,
                                        &mut double_clicked_loc,
                                    );
                                });
                            }
                        });
                });
            });

        // 1. ドラッグ＆ドロップの移動処理
        if let (Some(from), Some(to)) = (from, to) {
            self.move_item(from, to);
        }

        // 2. 【新規】ダブルクリックイベントが検知されていたら、安全にハンドラ関数を実行
        if let Some(loc) = double_clicked_loc {
            self.on_card_double_click(loc, work);
        }
    }

    /// 各セルの内部マージン調整と描画処理の仲介
    fn render_area_cell(
        &self,
        ui: &mut Ui,
        col_idx: usize,
        from: &mut Option<Location>,
        to: &mut Option<Location>,
        double_clicked_loc: &mut Option<Location>,
    ) {
        ui.style_mut().spacing.item_spacing = vec2(4.0, 4.0);

        if let Some((f, t)) = self.render_cell(ui, col_idx, double_clicked_loc) {
            *from = Some(f);
            *to = Some(t);
        }
    }

    /// エリア（ドロップゾーン）のレンダリング
    fn render_cell(
        &self,
        ui: &mut Ui,
        col_idx: usize,
        double_clicked_loc: &mut Option<Location>,
    ) -> Option<(Location, Location)> {
        let mut drop_event = None;

        let fill_color = match col_idx {
            0 | 3 => Color32::from_rgb(80, 0, 0),
            1 | 4 => Color32::from_rgb(80, 80, 0),
            2 | 5 => Color32::from_rgb(0, 80, 0),
            6 => Color32::from_rgb(0, 80, 80),
            _ => Color32::from_rgb(40, 40, 40),
        };

        let frame = Frame::canvas(ui.style())
            .outer_margin(0.0)
            .inner_margin(0.0);
        let total_cell_rect = ui.max_rect();

        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
            ui.painter().rect_filled(total_cell_rect, 0.0, fill_color);

            let available = ui.available_size();
            ui.set_min_size(available);

            ui.vertical(|ui| {
                let label_text = match col_idx {
                    0 => format!("{} {}", fl!("pending"), fl!("high")).to_string(),
                    1 => format!("{} {}", fl!("pending"), fl!("medium")).to_string(),
                    2 => format!("{} {}", fl!("pending"), fl!("low")).to_string(),
                    3 => format!("{} {}", fl!("work-in-progress"), fl!("high")).to_string(),
                    4 => format!("{} {}", fl!("work-in-progress"), fl!("medium")).to_string(),
                    5 => format!("{} {}", fl!("work-in-progress"), fl!("low")).to_string(),
                    6 => fl!("complete").to_string(),
                    7 => fl!("cancel").to_string(),
                    _ => format!("Area {}", col_idx + 1),
                };
                ui.label(label_text);

                egui::ScrollArea::vertical()
                    .id_salt(("kanban_scroll", col_idx))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (row_idx, item) in self.columns[col_idx].iter().enumerate() {
                            let current_loc = Location {
                                col: col_idx,
                                row: row_idx,
                            };

                            // render_card からダブルクリックの有無を受け取る
                            let (response, is_double_clicked) =
                                self.render_card(ui, item, current_loc);

                            // ダブルクリックされていたら、位置情報を上層へ書き戻す
                            if is_double_clicked {
                                *double_clicked_loc = Some(current_loc);
                            }

                            if let Some(insert_row) = handle_card_hover(ui, &response, current_loc)
                            {
                                if let Some(dragged) = response.dnd_release_payload() {
                                    drop_event = Some((
                                        *dragged,
                                        Location {
                                            col: col_idx,
                                            row: insert_row,
                                        },
                                    ));
                                }
                            }
                        }
                    });
            });
        });

        if let Some(dragged) = dropped_payload {
            drop_event = Some((
                *dragged,
                Location {
                    col: col_idx,
                    row: usize::MAX,
                },
            ));
        }

        drop_event
    }
}

/// カード上にホバーした際の位置判定とインジケータ（線）の描画
fn handle_card_hover(ui: &mut Ui, response: &Response, current_loc: Location) -> Option<usize> {
    let pointer = ui.input(|i| i.pointer.interact_pos())?;
    let _hovered = response.dnd_hover_payload::<Location>()?;

    let rect = response.rect;
    let stroke = Stroke::new(1.0, Color32::WHITE);

    if *_hovered == current_loc {
        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
        Some(current_loc.row)
    } else if pointer.y < rect.center().y {
        ui.painter().hline(rect.x_range(), rect.top(), stroke);
        Some(current_loc.row)
    } else {
        ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
        Some(current_loc.row + 1)
    }
}
