use eframe::egui;

use crate::theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDesignerState {
    pub tables: Vec<TableDraft>,
    pub selected_table: Option<usize>,
    pub template: SchemaTemplateDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDraft {
    pub name: String,
    pub columns: Vec<ColumnDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDraft {
    pub name: String,
    pub logical_type: LogicalTypeDraft,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalTypeDraft {
    String,
    Int64,
    Bool,
    Uuid,
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaTemplateDraft {
    Users,
    Content,
    Tasks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaStudioAction {
    Apply(Vec<String>),
}

impl Default for SchemaDesignerState {
    fn default() -> Self {
        let template = SchemaTemplateDraft::Users;
        Self {
            tables: template.tables(),
            selected_table: Some(0),
            template,
        }
    }
}

impl SchemaDesignerState {
    fn replace_with_template(&mut self) {
        self.tables = self.template.tables();
        self.selected_table = if self.tables.is_empty() {
            None
        } else {
            Some(0)
        };
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    state: &mut SchemaDesignerState,
    connected: bool,
    schema: &str,
) -> Option<SchemaStudioAction> {
    let mut action = None;
    let errors = validate(state);

    ui.horizontal(|ui| {
        ui.heading("Schema Studio");
        ui.label(
            egui::RichText::new(
                "Design tables visually, preview SQL, then apply with confirmation.",
            )
            .color(theme::MUTED),
        );
    });
    ui.horizontal(|ui| {
        draw_template_picker(ui, state);
        ui.separator();
        let can_apply = connected && errors.is_empty() && !state.tables.is_empty();
        if ui
            .add_enabled(can_apply, egui::Button::new("Apply schema"))
            .clicked()
        {
            action = Some(SchemaStudioAction::Apply(generate_postgres_statements(
                state, schema,
            )));
        }
        if connected {
            ui.label(format!("Target schema: {schema}"));
        } else {
            ui.label("Connect to a PostgreSQL profile to apply schema.");
        }
    });
    ui.separator();

    ui.columns(3, |columns| {
        draw_table_list(&mut columns[0], state);
        draw_table_editor(&mut columns[1], state);
        draw_preview(&mut columns[2], state, schema, &errors);
    });

    action
}

fn draw_template_picker(ui: &mut egui::Ui, state: &mut SchemaDesignerState) {
    ui.label("Template");
    egui::ComboBox::from_id_source("schema-template")
        .selected_text(state.template.label())
        .show_ui(ui, |ui| {
            for template in SchemaTemplateDraft::all() {
                ui.selectable_value(&mut state.template, template, template.label());
            }
        });
    if theme::secondary_button(ui, "Replace draft").clicked() {
        state.replace_with_template();
    }
}

fn draw_table_list(ui: &mut egui::Ui, state: &mut SchemaDesignerState) {
    ui.heading("Tables");
    if theme::primary_button(ui, "Add table").clicked() {
        state.tables.push(TableDraft {
            name: next_table_name(&state.tables),
            columns: vec![ColumnDraft::default_id()],
        });
        state.selected_table = Some(state.tables.len() - 1);
    }
    ui.add_space(8.0);

    let mut remove = None;
    for (idx, table) in state.tables.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(state.selected_table == Some(idx), &table.name)
                .clicked()
            {
                state.selected_table = Some(idx);
            }
            if theme::danger_button(ui, "Delete").clicked() {
                remove = Some(idx);
            }
        });
    }

    if let Some(idx) = remove {
        state.tables.remove(idx);
        state.selected_table = if state.tables.is_empty() {
            None
        } else {
            Some(0)
        };
    }
}

fn draw_table_editor(ui: &mut egui::Ui, state: &mut SchemaDesignerState) {
    let Some(table_idx) = state.selected_table else {
        ui.heading("No table selected");
        return;
    };
    let Some(table) = state.tables.get_mut(table_idx) else {
        return;
    };

    ui.heading("Table");
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut table.name);
    });
    ui.separator();

    ui.horizontal(|ui| {
        ui.heading("Columns");
        if theme::primary_button(ui, "Add column").clicked() {
            table
                .columns
                .push(ColumnDraft::default_text(&table.columns));
        }
    });

    let mut remove = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, column) in table.columns.iter_mut().enumerate() {
            egui::Frame::none()
                .fill(theme::PANEL_ALT)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(8.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut column.name);
                        type_combo(ui, &mut column.logical_type);
                        if theme::danger_button(ui, "Remove").clicked() {
                            remove = Some(idx);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut column.primary_key, "Primary key");
                        ui.checkbox(&mut column.nullable, "Nullable");
                        ui.checkbox(&mut column.unique, "Unique");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Default");
                        ui.text_edit_singleline(&mut column.default_value);
                    });
                });
            ui.add_space(6.0);
        }
    });

    if let Some(idx) = remove {
        table.columns.remove(idx);
    }
}

fn draw_preview(ui: &mut egui::Ui, state: &SchemaDesignerState, schema: &str, errors: &[String]) {
    ui.heading("Preview");
    if errors.is_empty() {
        theme::badge(ui, "valid", theme::PRIMARY);
    } else {
        for err in errors {
            ui.colored_label(theme::DANGER, err);
        }
    }
    ui.separator();
    ui.heading("Generated SQL");
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.monospace(generate_postgres_sql(state, schema));
    });
}

fn type_combo(ui: &mut egui::Ui, value: &mut LogicalTypeDraft) {
    egui::ComboBox::from_id_source(ui.next_auto_id())
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            ui.selectable_value(value, LogicalTypeDraft::String, "string");
            ui.selectable_value(value, LogicalTypeDraft::Int64, "int64");
            ui.selectable_value(value, LogicalTypeDraft::Bool, "bool");
            ui.selectable_value(value, LogicalTypeDraft::Uuid, "uuid");
            ui.selectable_value(value, LogicalTypeDraft::Timestamp, "timestamp");
        });
}

pub fn validate(state: &SchemaDesignerState) -> Vec<String> {
    let mut errors = Vec::new();
    let mut tables = std::collections::HashSet::new();
    for table in &state.tables {
        if !is_ident(&table.name) {
            errors.push(format!("Invalid table name: {}", table.name));
        }
        if !tables.insert(table.name.clone()) {
            errors.push(format!("Duplicate table name: {}", table.name));
        }
        let mut columns = std::collections::HashSet::new();
        let mut primary_keys = 0;
        for column in &table.columns {
            if !is_ident(&column.name) {
                errors.push(format!(
                    "Invalid column name: {}.{}",
                    table.name, column.name
                ));
            }
            if !columns.insert(column.name.clone()) {
                errors.push(format!("Duplicate column: {}.{}", table.name, column.name));
            }
            if column.primary_key {
                primary_keys += 1;
            }
        }
        if primary_keys > 1 {
            errors.push(format!(
                "Table {} has more than one primary key column",
                table.name
            ));
        }
    }
    errors
}

pub fn generate_postgres_sql(state: &SchemaDesignerState, schema: &str) -> String {
    generate_postgres_statements(state, schema).join("\n\n")
}

pub fn generate_postgres_statements(state: &SchemaDesignerState, schema: &str) -> Vec<String> {
    let mut out = Vec::new();
    for table in &state.tables {
        let cols = table
            .columns
            .iter()
            .map(column_sql)
            .collect::<Vec<_>>()
            .join(",\n  ");
        out.push(format!(
            "CREATE TABLE IF NOT EXISTS {}.{} (\n  {}\n);",
            quote_ident(schema),
            quote_ident(&table.name),
            cols
        ));
    }
    out
}

fn column_sql(column: &ColumnDraft) -> String {
    let mut out = format!(
        "{} {}",
        quote_ident(&column.name),
        column.logical_type.postgres_type()
    );
    if column.primary_key {
        out.push_str(" PRIMARY KEY");
    }
    if column.unique && !column.primary_key {
        out.push_str(" UNIQUE");
    }
    if !column.nullable && !column.primary_key {
        out.push_str(" NOT NULL");
    }
    if !column.default_value.trim().is_empty() {
        out.push_str(" DEFAULT ");
        out.push_str(&map_default(column));
    }
    out
}

fn map_default(column: &ColumnDraft) -> String {
    let default = column.default_value.trim();
    if default.eq_ignore_ascii_case("now") && column.logical_type == LogicalTypeDraft::Timestamp {
        "NOW()".to_string()
    } else if default.ends_with(')') {
        default.to_string()
    } else if matches!(
        column.logical_type,
        LogicalTypeDraft::String | LogicalTypeDraft::Uuid | LogicalTypeDraft::Timestamp
    ) {
        format!("'{}'", default.replace('\'', "''"))
    } else {
        default.to_string()
    }
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn next_table_name(tables: &[TableDraft]) -> String {
    let mut idx = tables.len() + 1;
    loop {
        let candidate = format!("table_{idx}");
        if !tables.iter().any(|table| table.name == candidate) {
            return candidate;
        }
        idx += 1;
    }
}

impl ColumnDraft {
    fn new(
        name: &str,
        logical_type: LogicalTypeDraft,
        nullable: bool,
        primary_key: bool,
        unique: bool,
        default_value: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            logical_type,
            nullable,
            primary_key,
            unique,
            default_value: default_value.to_string(),
        }
    }

    fn default_id() -> Self {
        Self::new(
            "id",
            LogicalTypeDraft::Uuid,
            false,
            true,
            false,
            "gen_random_uuid()",
        )
    }

    fn default_text(existing: &[ColumnDraft]) -> Self {
        Self {
            name: format!("column_{}", existing.len() + 1),
            logical_type: LogicalTypeDraft::String,
            nullable: true,
            primary_key: false,
            unique: false,
            default_value: String::new(),
        }
    }
}

impl SchemaTemplateDraft {
    fn all() -> [Self; 3] {
        [Self::Users, Self::Content, Self::Tasks]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Users => "Users",
            Self::Content => "Content",
            Self::Tasks => "Tasks",
        }
    }

    fn tables(self) -> Vec<TableDraft> {
        match self {
            Self::Users => vec![TableDraft {
                name: "users".to_string(),
                columns: vec![
                    ColumnDraft::default_id(),
                    ColumnDraft::new("email", LogicalTypeDraft::String, false, false, true, ""),
                    ColumnDraft::new(
                        "created_at",
                        LogicalTypeDraft::Timestamp,
                        false,
                        false,
                        false,
                        "now",
                    ),
                ],
            }],
            Self::Content => vec![TableDraft {
                name: "posts".to_string(),
                columns: vec![
                    ColumnDraft::default_id(),
                    ColumnDraft::new("title", LogicalTypeDraft::String, false, false, false, ""),
                    ColumnDraft::new("slug", LogicalTypeDraft::String, false, false, true, ""),
                    ColumnDraft::new("body", LogicalTypeDraft::String, true, false, false, ""),
                    ColumnDraft::new(
                        "published",
                        LogicalTypeDraft::Bool,
                        false,
                        false,
                        false,
                        "false",
                    ),
                    ColumnDraft::new(
                        "created_at",
                        LogicalTypeDraft::Timestamp,
                        false,
                        false,
                        false,
                        "now",
                    ),
                ],
            }],
            Self::Tasks => vec![
                TableDraft {
                    name: "projects".to_string(),
                    columns: vec![
                        ColumnDraft::default_id(),
                        ColumnDraft::new("name", LogicalTypeDraft::String, false, false, true, ""),
                        ColumnDraft::new(
                            "created_at",
                            LogicalTypeDraft::Timestamp,
                            false,
                            false,
                            false,
                            "now",
                        ),
                    ],
                },
                TableDraft {
                    name: "tasks".to_string(),
                    columns: vec![
                        ColumnDraft::default_id(),
                        ColumnDraft::new(
                            "title",
                            LogicalTypeDraft::String,
                            false,
                            false,
                            false,
                            "",
                        ),
                        ColumnDraft::new(
                            "done",
                            LogicalTypeDraft::Bool,
                            false,
                            false,
                            false,
                            "false",
                        ),
                        ColumnDraft::new(
                            "created_at",
                            LogicalTypeDraft::Timestamp,
                            false,
                            false,
                            false,
                            "now",
                        ),
                    ],
                },
            ],
        }
    }
}

impl LogicalTypeDraft {
    fn as_str(self) -> &'static str {
        match self {
            LogicalTypeDraft::String => "string",
            LogicalTypeDraft::Int64 => "int64",
            LogicalTypeDraft::Bool => "bool",
            LogicalTypeDraft::Uuid => "uuid",
            LogicalTypeDraft::Timestamp => "timestamp",
        }
    }

    fn postgres_type(self) -> &'static str {
        match self {
            LogicalTypeDraft::String => "TEXT",
            LogicalTypeDraft::Int64 => "BIGINT",
            LogicalTypeDraft::Bool => "BOOLEAN",
            LogicalTypeDraft::Uuid => "UUID",
            LogicalTypeDraft::Timestamp => "TIMESTAMPTZ",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        generate_postgres_sql, generate_postgres_statements, validate, SchemaDesignerState,
        SchemaTemplateDraft,
    };

    #[test]
    fn default_schema_is_valid() {
        assert!(validate(&SchemaDesignerState::default()).is_empty());
    }

    #[test]
    fn generates_create_table_sql() {
        let sql = generate_postgres_sql(&SchemaDesignerState::default(), "public");
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"public\".\"users\""));
        assert!(sql.contains("\"email\" TEXT UNIQUE NOT NULL"));
    }

    #[test]
    fn generates_statements_per_table() {
        let statements = generate_postgres_statements(&SchemaDesignerState::default(), "public");
        assert_eq!(statements.len(), 1);
        assert!(statements[0].ends_with(';'));
    }

    #[test]
    fn detects_duplicate_tables() {
        let mut state = SchemaDesignerState::default();
        state.tables.push(state.tables[0].clone());
        assert!(validate(&state)
            .iter()
            .any(|err| err.contains("Duplicate table")));
    }

    #[test]
    fn starter_templates_are_valid() {
        for template in SchemaTemplateDraft::all() {
            let state = SchemaDesignerState {
                tables: template.tables(),
                selected_table: Some(0),
                template,
            };
            assert!(
                validate(&state).is_empty(),
                "invalid template: {template:?}"
            );
        }
    }

    #[test]
    fn tasks_template_generates_multiple_statements() {
        let state = SchemaDesignerState {
            tables: SchemaTemplateDraft::Tasks.tables(),
            selected_table: Some(0),
            template: SchemaTemplateDraft::Tasks,
        };
        let statements = generate_postgres_statements(&state, "public");
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("\"projects\""));
        assert!(statements[1].contains("\"tasks\""));
    }
}
