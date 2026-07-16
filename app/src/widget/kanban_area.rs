//! Kanban board widget: groups tasks into columns by priority and status,
//! and supports dragging cards between columns or double-clicking a card
//! to edit it.

use crate::page::Pages;
use crate::work::Work;
use core::{Task, TaskPriority, TaskStatus};
use egui::{Color32, Frame, Id, Rect, Response, Stroke, Ui, vec2};
use tracing::info;

/// Number of columns on the board: `Pending` and `WorkInProgress` each get
/// three (High / Medium / Low), plus one each for `Complete` and
/// `Canceled`.
const COLUMN_COUNT: usize = 8;

const COL_PENDING_HIGH: usize = 0;
const COL_PENDING_MEDIUM: usize = 1;
const COL_PENDING_LOW: usize = 2;
const COL_WIP_HIGH: usize = 3;
const COL_WIP_MEDIUM: usize = 4;
const COL_WIP_LOW: usize = 5;
const COL_COMPLETE: usize = 6;
const COL_CANCELED: usize = 7;

pub struct KanbanArea {
    columns: Vec<Vec<Task>>,
}

impl Default for KanbanArea {
    fn default() -> Self {
        Self {
            columns: vec![vec![]; COLUMN_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}

/// Drag-and-drop and double-click events gathered while rendering one
/// frame. Applied to the board only after rendering has finished, so we
/// never mutate `columns` while it's still being iterated over.
#[derive(Default)]
struct FrameEvents {
    drag_from: Option<Location>,
    drop_to: Option<Location>,
    double_clicked: Option<Location>,
}

impl KanbanArea {
    /// Sorts `tasks` into columns by priority and status.
    pub(crate) fn set_tasks(&mut self, tasks: &[Task]) {
        for task in tasks {
            let column = match (task.priority, task.status) {
                (TaskPriority::High, TaskStatus::Pending) => COL_PENDING_HIGH,
                (TaskPriority::Medium, TaskStatus::Pending) => COL_PENDING_MEDIUM,
                (TaskPriority::Low, TaskStatus::Pending) => COL_PENDING_LOW,
                (TaskPriority::High, TaskStatus::WorkInProgress) => COL_WIP_HIGH,
                (TaskPriority::Medium, TaskStatus::WorkInProgress) => COL_WIP_MEDIUM,
                (TaskPriority::Low, TaskStatus::WorkInProgress) => COL_WIP_LOW,
                (_, TaskStatus::Complete) => COL_COMPLETE,
                (_, TaskStatus::Canceled) => COL_CANCELED,
            };
            self.columns[column].push(task.clone());
        }
    }

    /// Empties every column, returning their previous contents.
    pub(crate) fn pop_columns(&mut self) -> Vec<Vec<Task>> {
        std::mem::replace(&mut self.columns, vec![vec![]; COLUMN_COUNT])
    }

    /// Renders the whole board, then applies whatever drag-and-drop move or
    /// double-click edit that rendering pass detected.
    pub(crate) fn show(&mut self, ui: &mut Ui, work: &mut Work) {
        let mut events = FrameEvents::default();
        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);

        egui_extras::StripBuilder::new(ui)
            .sizes(egui_extras::Size::remainder(), 3)
            .horizontal(|mut col_strip| {
                // Pending: High / Medium / Low.
                col_strip.cell(|ui| self.render_column_strip(ui, COL_PENDING_HIGH, 3, &mut events));
                // Work in progress: High / Medium / Low.
                col_strip.cell(|ui| self.render_column_strip(ui, COL_WIP_HIGH, 3, &mut events));
                // Complete / Canceled.
                col_strip.cell(|ui| self.render_column_strip(ui, COL_COMPLETE, 2, &mut events));
            });

        if let (Some(from), Some(to)) = (events.drag_from, events.drop_to) {
            self.move_item(from, to);
        }
        if let Some(loc) = events.double_clicked {
            self.on_card_double_click(loc, work);
        }
    }

    /// Renders `row_count` stacked drop zones, for columns
    /// `start_col..start_col + row_count`.
    fn render_column_strip(
        &self,
        ui: &mut Ui,
        start_col: usize,
        row_count: usize,
        events: &mut FrameEvents,
    ) {
        egui_extras::StripBuilder::new(ui)
            .sizes(egui_extras::Size::remainder(), row_count)
            .vertical(|mut row_strip| {
                for offset in 0..row_count {
                    row_strip.cell(|ui| self.render_cell(ui, start_col + offset, events));
                }
            });
    }

    /// Renders one column: background, header label, and task list. Also
    /// records a drop event when a card is released on the column's empty
    /// background (rather than directly on another card).
    fn render_cell(&self, ui: &mut Ui, col_idx: usize, events: &mut FrameEvents) {
        ui.style_mut().spacing.item_spacing = vec2(4.0, 4.0);

        let frame = Frame::canvas(ui.style())
            .outer_margin(0.0)
            .inner_margin(0.0);
        let total_cell_rect = ui.max_rect();
        let fill_color = column_fill_color(col_idx);

        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
            ui.painter().rect_filled(total_cell_rect, 0.0, fill_color);

            let available_size = ui.available_size();
            ui.set_min_size(available_size);

            ui.vertical(|ui| {
                ui.label(column_label(col_idx));
                self.render_task_list(ui, col_idx, events);
            });
        });

        if let Some(dragged) = dropped_payload {
            events.drag_from = Some(*dragged);
            events.drop_to = Some(Location {
                col: col_idx,
                row: usize::MAX,
            });
        }
    }

    /// Renders every card in `col_idx`'s column, inside a scroll area.
    fn render_task_list(&self, ui: &mut Ui, col_idx: usize, events: &mut FrameEvents) {
        egui::ScrollArea::vertical()
            .id_salt(("kanban_scroll", col_idx))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (row, task) in self.columns[col_idx].iter().enumerate() {
                    let loc = Location { col: col_idx, row };
                    self.render_task_row(ui, task, loc, events);
                }
            });
    }

    /// Renders a single card and records any double-click or drop event it
    /// produced this frame.
    fn render_task_row(&self, ui: &mut Ui, task: &Task, loc: Location, events: &mut FrameEvents) {
        let (response, is_double_clicked) = render_card(ui, task, loc);

        if is_double_clicked {
            events.double_clicked = Some(loc);
        }

        if let Some((dragged, insert_row)) = resolve_card_drop(ui, &response, loc) {
            events.drag_from = Some(dragged);
            events.drop_to = Some(Location {
                col: loc.col,
                row: insert_row,
            });
        }
    }

    /// Moves a card from `from` to `to`, adjusting `to.row` to account for
    /// the source item's removal when both locations share a column.
    fn move_item(&mut self, from: Location, mut to: Location) {
        if from.col == to.col && from.row < to.row {
            to.row -= 1;
        }

        let Some(item) = self.columns[from.col].get(from.row).cloned() else {
            return;
        };
        self.columns[from.col].remove(from.row);

        let target = &mut self.columns[to.col];
        to.row = to.row.min(target.len());
        target.insert(to.row, item);
    }

    /// Looks up the double-clicked card and hands an owned clone of it to
    /// `work` for editing.
    fn on_card_double_click(&self, loc: Location, work: &mut Work) {
        let Some(task) = self.columns[loc.col].get(loc.row) else {
            return;
        };

        info!(
            "Task card double clicked: '{}' (Column: {}, Row: {})",
            task.title, loc.col, loc.row
        );
        work.task = task.clone();
        work.next_page = Pages::EditTask;
    }
}

/// Renders a single draggable card, returning its response and whether it
/// was double-clicked this frame.
fn render_card(ui: &mut Ui, task: &Task, loc: Location) -> (Response, bool) {
    let item_id = Id::new(("kanban_item", loc.col, loc.row));
    let available_width = ui.available_width();

    let dnd_res = ui.dnd_drag_source(item_id, loc, |ui| {
        draw_card_content(ui, task, available_width);
    });

    let is_double_clicked = is_double_click_inside(ui, dnd_res.response.rect);
    (dnd_res.response, is_double_clicked)
}

/// Draws the card's window frame and its truncated, single-line title.
fn draw_card_content(ui: &mut Ui, task: &Task, available_width: f32) {
    let card_frame = Frame::window(ui.style()).inner_margin(egui::Margin::symmetric(2, 2));

    card_frame.show(ui, |ui| {
        let desired_size = vec2(available_width, 18.0);
        let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true);

        ui.allocate_ui_with_layout(desired_size, layout, |ui| {
            ui.set_max_width(available_width);
            ui.add(egui::Label::new(task.title.clone()).truncate());
        });
    });
}

/// Whether the primary button double-clicked inside `rect` this frame.
///
/// This is a plain geometric hit-test (pointer position vs. rect) rather
/// than the response's own click state, since drag-and-drop consumes that
/// before a double-click can register on it.
fn is_double_click_inside(ui: &Ui, rect: Rect) -> bool {
    let double_clicked = ui.input(|i| {
        i.pointer
            .button_double_clicked(egui::PointerButton::Primary)
    });

    double_clicked
        && ui
            .input(|i| i.pointer.interact_pos())
            .is_some_and(|pos| rect.contains(pos))
}

/// If a dragged card was released on top of `loc` this frame, returns the
/// card's origin location and the row it should be inserted at.
fn resolve_card_drop(ui: &mut Ui, response: &Response, loc: Location) -> Option<(Location, usize)> {
    let insert_row = handle_card_hover(ui, response, loc)?;
    let dragged = response.dnd_release_payload::<Location>()?;
    Some((*dragged, insert_row))
}

/// Draws a highlight line marking where a dragged card would land if
/// dropped on `current_loc`, and returns the row index it would land at.
fn handle_card_hover(ui: &mut Ui, response: &Response, current_loc: Location) -> Option<usize> {
    let pointer = ui.input(|i| i.pointer.interact_pos())?;
    let hovered = response.dnd_hover_payload::<Location>()?;

    let rect = response.rect;
    let stroke = Stroke::new(1.0, Color32::WHITE);

    if *hovered == current_loc {
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

/// Background color for a column, grouped by priority/status.
fn column_fill_color(col_idx: usize) -> Color32 {
    match col_idx {
        COL_PENDING_HIGH | COL_WIP_HIGH => Color32::from_rgb(80, 0, 0),
        COL_PENDING_MEDIUM | COL_WIP_MEDIUM => Color32::from_rgb(80, 80, 0),
        COL_PENDING_LOW | COL_WIP_LOW => Color32::from_rgb(0, 80, 0),
        COL_COMPLETE => Color32::from_rgb(0, 80, 80),
        _ => Color32::from_rgb(40, 40, 40),
    }
}

/// Header label for a column.
fn column_label(col_idx: usize) -> String {
    match col_idx {
        COL_PENDING_HIGH => format!("{} {}", fl!("pending"), fl!("high")),
        COL_PENDING_MEDIUM => format!("{} {}", fl!("pending"), fl!("medium")),
        COL_PENDING_LOW => format!("{} {}", fl!("pending"), fl!("low")),
        COL_WIP_HIGH => format!("{} {}", fl!("work-in-progress"), fl!("high")),
        COL_WIP_MEDIUM => format!("{} {}", fl!("work-in-progress"), fl!("medium")),
        COL_WIP_LOW => format!("{} {}", fl!("work-in-progress"), fl!("low")),
        COL_COMPLETE => fl!("complete").to_string(),
        COL_CANCELED => fl!("cancel").to_string(),
        _ => format!("Area {}", col_idx + 1),
    }
}
