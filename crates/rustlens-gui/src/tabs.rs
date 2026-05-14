use eframe::egui;

use crate::{draw_result_table, schema_studio, sql_editor, theme, GuiApp};

impl GuiApp {
    pub(crate) fn draw_browse(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                self.selected_table
                    .as_deref()
                    .unwrap_or("Select a table from the sidebar"),
            );
            ui.separator();
            if theme::secondary_button(ui, "Prev").clicked() {
                self.page = (self.page - 1).max(0);
                self.load_current_page();
            }
            ui.label(format!("Page {}", self.page + 1));
            if theme::secondary_button(ui, "Next").clicked() {
                self.page += 1;
                self.load_current_page();
            }
            if theme::secondary_button(ui, "Refresh").clicked() {
                self.refresh_current_page();
            }
        });

        ui.separator();
        self.draw_results_with_inspector(ui);
    }

    pub(crate) fn draw_sql(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if theme::primary_button(ui, "Execute (F5)").clicked() {
                self.execute_sql();
            }
            ui.label("Ctrl+Enter also executes. Mutating SQL requires confirmation.");
        });

        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut job = sql_editor::highlight_job(text);
            job.wrap.max_width = wrap_width;
            ui.fonts(|fonts| fonts.layout_job(job))
        };

        let output = egui::TextEdit::multiline(&mut self.sql_text)
            .font(egui::TextStyle::Monospace)
            .desired_rows(10)
            .lock_focus(true)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter)
            .show(ui);

        if let Some(range) = output.cursor_range {
            self.sql_cursor = range.primary.ccursor.index;
        } else {
            self.sql_cursor = self.sql_text.len();
        }

        if output.response.changed() || output.response.has_focus() {
            self.refresh_sql_completion();
        }

        self.handle_completion_keys(ui);
        self.draw_sql_completion(ui);

        ui.separator();
        self.draw_results_with_inspector(ui);
    }

    pub(crate) fn draw_schema_studio(&mut self, ui: &mut egui::Ui) {
        let schema = self.schema.clone();
        if let Some(action) =
            schema_studio::draw(ui, &mut self.schema_designer, self.connected, &schema)
        {
            match action {
                schema_studio::SchemaStudioAction::Apply(statements) => {
                    if statements.is_empty() {
                        self.detail_status =
                            "Schema design has no statements to apply.".to_string();
                    } else {
                        self.pending_schema_confirmation = Some(statements);
                    }
                }
            }
        }
    }

    fn refresh_sql_completion(&mut self) {
        let result = rustlens_core::sql::completion::complete(
            &self.sql_text,
            self.sql_cursor,
            &self.tables,
            &self.sql_columns,
        );
        self.sql_completion_selected = 0;
        self.sql_completion = result.visible.then_some(result);
    }

    fn draw_results_with_inspector(&mut self, ui: &mut egui::Ui) {
        if let Some(cell) = draw_result_table(ui, &self.columns, &self.rows) {
            self.selected_cell = Some(cell);
        }

        let Some(cell) = self.selected_cell.as_ref() else {
            return;
        };

        ui.separator();
        egui::Frame::none()
            .fill(theme::PANEL_ALT)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Cell Inspector");
                    ui.label(
                        egui::RichText::new(format!(
                            "row {}, column {}",
                            cell.row_index + 1,
                            cell.column
                        ))
                        .small()
                        .color(theme::MUTED),
                    );
                });
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.monospace(&cell.value);
                    });
            });
    }

    fn handle_completion_keys(&mut self, ui: &egui::Ui) {
        if self.sql_completion.is_none() {
            return;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.sql_completion = None;
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if let Some(result) = &self.sql_completion {
                self.sql_completion_selected =
                    (self.sql_completion_selected + 1).min(result.items.len().saturating_sub(1));
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.sql_completion_selected = self.sql_completion_selected.saturating_sub(1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.accept_sql_completion();
        }
    }

    fn draw_sql_completion(&mut self, ui: &mut egui::Ui) {
        let Some(result) = self.sql_completion.clone() else {
            return;
        };

        egui::Frame::none()
            .fill(theme::PANEL_ALT)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Completions");
                    ui.label(
                        egui::RichText::new("Tab to accept, Esc to close")
                            .small()
                            .color(theme::MUTED),
                    );
                });
                let max = result.items.len().min(8);
                for (idx, item) in result.items.iter().take(max).enumerate() {
                    let selected = idx == self.sql_completion_selected;
                    let label = format!(
                        "{}   {}",
                        item.label,
                        sql_editor::completion_kind_label(item.kind)
                    );
                    if ui.selectable_label(selected, label).clicked() {
                        self.sql_completion_selected = idx;
                        self.accept_sql_completion();
                    }
                }
            });
    }

    fn accept_sql_completion(&mut self) {
        let Some(result) = self.sql_completion.clone() else {
            return;
        };
        rustlens_core::sql::completion::apply_completion(
            &mut self.sql_text,
            &mut self.sql_cursor,
            &result,
            self.sql_completion_selected,
        );
        self.sql_completion = None;
    }
}
