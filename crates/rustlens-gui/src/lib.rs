mod dialogs;
mod events;
mod helpers;
mod profile_flow;
mod profiles;
mod schema_studio;
mod sidebar;
mod sql_editor;
mod tabs;
mod theme;
mod wizard;

use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rustlens_core::db;
use rustlens_core::model::connection::{ConnectionProfile, Driver, ProfileSource};
use rustlens_core::provision::capabilities::{SystemCapabilities, ToolStatus};
use rustlens_core::provision::installer::InstallTarget;
use rustlens_core::sql::completion::CompletionResult;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

const DEFAULT_SCHEMA: &str = "public";
const DEFAULT_PAGE_SIZE: i64 = 200;

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustLens GUI")
            .with_inner_size([1200.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RustLens GUI",
        options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)?))),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Browse,
    Sql,
    Schema,
}

struct GuiApp {
    _runtime: Runtime,
    db_cmd_tx: mpsc::Sender<db::DbCmd>,
    db_evt_rx: mpsc::Receiver<db::DbEvt>,
    provision_evt_rx: std::sync::mpsc::Receiver<ProvisionEvt>,
    provision_evt_tx: std::sync::mpsc::Sender<ProvisionEvt>,

    profiles: Vec<profiles::Profile>,
    selected_profile: Option<usize>,
    wizard: SetupWizard,
    capabilities: SystemCapabilities,
    provisioning: bool,

    status: String,
    detail_status: String,
    connected: bool,
    schema: String,
    page_size: i64,

    tab: Tab,
    tables: Vec<String>,
    selected_table: Option<String>,
    page: i64,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    selected_cell: Option<SelectedCell>,
    next_request_id: u64,
    latest_result_request_id: Option<u64>,
    request_contexts: HashMap<u64, RequestContext>,
    page_cache: HashMap<PageCacheKey, CachedPage>,
    loading_status: Option<String>,

    sql_text: String,
    sql_cursor: usize,
    sql_completion: Option<CompletionResult>,
    sql_completion_selected: usize,
    sql_columns: HashMap<String, Vec<String>>,
    pending_sql_confirmation: Option<String>,
    pending_schema_confirmation: Option<Vec<String>>,
    pending_profile_edit: Option<EditProfileForm>,
    pending_profile_delete: Option<profiles::Profile>,
    schema_designer: schema_studio::SchemaDesignerState,
}

pub(crate) enum ProvisionEvt {
    Created(Result<ConnectionProfile, String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WizardStep {
    Welcome,
    AddExisting,
    CreateLocal,
    Install,
}

#[derive(Debug)]
struct SetupWizard {
    visible: bool,
    step: WizardStep,
    add_existing: AddExistingForm,
    create_local: CreateLocalForm,
    install_confirmation: Option<InstallConfirmation>,
}

#[derive(Debug)]
pub(crate) struct AddExistingForm {
    name: String,
    host: String,
    port: String,
    database: String,
    user: String,
    password: String,
    schema: String,
    page_size: String,
    advanced_url: bool,
    database_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateEngine {
    PostgresDocker,
    Sqlite,
    Mysql,
}

#[derive(Debug, Clone)]
pub(crate) struct CreateLocalForm {
    engine: CreateEngine,
    profile_name: String,
    database: String,
    user: String,
    password: String,
    image: String,
    schema: String,
    page_size: String,
    sqlite_path: String,
}

#[derive(Debug, Clone)]
struct InstallConfirmation {
    target: InstallTarget,
    commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct EditProfileForm {
    id: uuid::Uuid,
    name: String,
    driver: Driver,
    database_url: String,
    schema: String,
    page_size: String,
    source: ProfileSource,
    dbnest_instance_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedCell {
    column: String,
    row_index: usize,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PageCacheKey {
    schema: String,
    table: String,
    page: i64,
    page_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CachedPage {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequestContext {
    TablePage(PageCacheKey),
    Sql,
    SchemaApply,
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self {
            visible: false,
            step: WizardStep::Welcome,
            add_existing: AddExistingForm::default(),
            create_local: CreateLocalForm::default(),
            install_confirmation: None,
        }
    }
}

impl Default for AddExistingForm {
    fn default() -> Self {
        Self {
            name: "local".to_string(),
            host: "127.0.0.1".to_string(),
            port: "5432".to_string(),
            database: "appdb".to_string(),
            user: "app".to_string(),
            password: "app".to_string(),
            schema: DEFAULT_SCHEMA.to_string(),
            page_size: DEFAULT_PAGE_SIZE.to_string(),
            advanced_url: false,
            database_url: "postgres://app:app@127.0.0.1:5432/appdb".to_string(),
        }
    }
}

impl Default for CreateLocalForm {
    fn default() -> Self {
        Self {
            engine: CreateEngine::PostgresDocker,
            profile_name: "local".to_string(),
            database: "appdb".to_string(),
            user: "app".to_string(),
            password: "app".to_string(),
            image: "postgres:16-alpine".to_string(),
            schema: DEFAULT_SCHEMA.to_string(),
            page_size: DEFAULT_PAGE_SIZE.to_string(),
            sqlite_path: String::new(),
        }
    }
}

impl GuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        theme::apply(&cc.egui_ctx);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let (db_cmd_tx, db_cmd_rx) = mpsc::channel::<db::DbCmd>(64);
        let (db_evt_tx, db_evt_rx) = mpsc::channel::<db::DbEvt>(256);
        let (provision_evt_tx, provision_evt_rx) = std::sync::mpsc::channel();

        runtime.spawn(async move {
            if let Err(err) = db::worker::run(db_cmd_rx, db_evt_tx).await {
                eprintln!("db worker crashed: {err:#}");
            }
        });

        let (profiles, status) = match profiles::load_profiles() {
            Ok(profiles) => (profiles, "Profiles loaded.".to_string()),
            Err(err) => (Vec::new(), format!("Profile load error: {err}")),
        };
        let wizard = SetupWizard {
            visible: profiles.is_empty(),
            ..Default::default()
        };

        Ok(Self {
            _runtime: runtime,
            db_cmd_tx,
            db_evt_rx,
            provision_evt_rx,
            provision_evt_tx,
            profiles,
            selected_profile: None,
            wizard,
            capabilities: rustlens_core::provision::capabilities::detect_system_capabilities(),
            provisioning: false,
            status,
            detail_status: profiles::profiles_path_text(),
            connected: false,
            schema: DEFAULT_SCHEMA.to_string(),
            page_size: DEFAULT_PAGE_SIZE,
            tab: Tab::Browse,
            tables: Vec::new(),
            selected_table: None,
            page: 0,
            columns: Vec::new(),
            rows: Vec::new(),
            selected_cell: None,
            next_request_id: 1,
            latest_result_request_id: None,
            request_contexts: HashMap::new(),
            page_cache: HashMap::new(),
            loading_status: None,
            sql_text: String::new(),
            sql_cursor: 0,
            sql_completion: None,
            sql_completion_selected: 0,
            sql_columns: HashMap::new(),
            pending_sql_confirmation: None,
            pending_schema_confirmation: None,
            pending_profile_edit: None,
            pending_profile_delete: None,
            schema_designer: schema_studio::SchemaDesignerState::default(),
        })
    }

    fn send_cmd(&mut self, cmd: db::DbCmd) {
        if let Err(err) = self.db_cmd_tx.try_send(cmd) {
            self.status = format!("Could not queue DB command: {err}");
        }
    }

    fn begin_tracked_request(&mut self, context: RequestContext, loading_status: String) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1).max(1);
        self.latest_result_request_id = Some(request_id);
        self.request_contexts.insert(request_id, context);
        self.loading_status = Some(loading_status);
        request_id
    }

    fn open_table(&mut self, table: String) {
        self.selected_table = Some(table.clone());
        self.page = 0;
        self.load_current_page();
    }

    fn load_current_page(&mut self) {
        let Some(table) = self.selected_table.clone() else {
            return;
        };

        let key = self.current_page_cache_key(table);
        if let Some(cached) = self.page_cache.get(&key).cloned() {
            self.columns = cached.columns;
            self.rows = cached.rows;
            self.selected_cell = None;
            self.detail_status = format!("Loaded page {} from cache.", self.page + 1);
            return;
        }

        self.load_page_uncached(key);
    }

    fn refresh_current_page(&mut self) {
        let Some(table) = self.selected_table.clone() else {
            return;
        };

        let key = self.current_page_cache_key(table);
        self.page_cache.remove(&key);
        self.load_page_uncached(key);
    }

    fn current_page_cache_key(&self, table: String) -> PageCacheKey {
        PageCacheKey {
            schema: self.schema.clone(),
            table,
            page: self.page,
            page_size: self.page_size,
        }
    }

    fn load_page_uncached(&mut self, key: PageCacheKey) {
        let request_id = self.begin_tracked_request(
            RequestContext::TablePage(key.clone()),
            format!("Loading page {}...", key.page + 1),
        );
        self.send_cmd(db::DbCmd::LoadTablePage {
            request_id: Some(request_id),
            schema: key.schema,
            table: key.table,
            page: key.page,
            page_size: key.page_size,
        });
    }

    fn execute_sql(&mut self) {
        let sql = self.sql_text.trim().to_string();
        if sql.is_empty() {
            self.detail_status = "SQL is empty.".to_string();
            return;
        }

        if rustlens_core::sql::safety::requires_confirmation(&sql) {
            self.pending_sql_confirmation = Some(sql);
            return;
        }

        let request_id = self.begin_tracked_request(RequestContext::Sql, "Executing SQL...".into());
        self.send_cmd(db::DbCmd::ExecuteSql {
            request_id: Some(request_id),
            sql,
        });
    }

    fn confirm_pending_sql(&mut self) {
        if let Some(sql) = self.pending_sql_confirmation.take() {
            let request_id = self
                .begin_tracked_request(RequestContext::Sql, "Executing confirmed SQL...".into());
            self.detail_status = "Executing confirmed SQL.".to_string();
            self.send_cmd(db::DbCmd::ExecuteSql {
                request_id: Some(request_id),
                sql,
            });
        }
    }

    fn confirm_pending_schema(&mut self) {
        let Some(statements) = self.pending_schema_confirmation.take() else {
            return;
        };

        let statement_count = statements.len();
        self.page_cache.clear();
        let request_id = self.begin_tracked_request(
            RequestContext::SchemaApply,
            format!("Applying {statement_count} schema statement(s)..."),
        );
        self.send_cmd(db::DbCmd::ExecuteSqlBatch {
            request_id: Some(request_id),
            statements,
            refresh_schema: Some(self.schema.clone()),
        });
        self.detail_status = format!("Applying {statement_count} schema statement(s) as a batch.");
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_db_events(ctx);
        self.drain_provision_events(ctx);
        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("top_bar")
            .frame(theme::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("RustLens");
                    ui.separator();
                    selectable_tab(ui, &mut self.tab, Tab::Browse, "Browse");
                    selectable_tab(ui, &mut self.tab, Tab::Sql, "SQL");
                    selectable_tab(ui, &mut self.tab, Tab::Schema, "Schema Studio");
                    ui.separator();
                    ui.label(format!("Schema: {}", self.schema));
                    ui.separator();
                    if ui.button("Setup").clicked() {
                        self.wizard.visible = true;
                        self.wizard.step = WizardStep::Welcome;
                    }
                });
            });

        egui::SidePanel::left("sidebar")
            .frame(theme::side_panel_frame())
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| self.draw_sidebar(ui));

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Browse => self.draw_browse(ui),
            Tab::Sql => self.draw_sql(ui),
            Tab::Schema => self.draw_schema_studio(ui),
        });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(theme::status_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.status);
                    if !self.detail_status.is_empty() {
                        ui.separator();
                        ui.label(&self.detail_status);
                    }
                    if let Some(loading_status) = &self.loading_status {
                        ui.separator();
                        ui.label(loading_status);
                    }
                });
            });

        self.draw_confirmation(ctx);
        self.draw_schema_confirmation(ctx);
        self.draw_profile_edit(ctx);
        self.draw_profile_delete_confirmation(ctx);
        self.draw_setup_wizard(ctx);
    }
}

impl GuiApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) && self.tab == Tab::Sql {
            self.execute_sql();
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
            && self.tab == Tab::Sql
        {
            self.execute_sql();
        }
    }
}

fn selectable_tab(ui: &mut egui::Ui, current: &mut Tab, tab: Tab, label: &str) {
    if ui.selectable_label(*current == tab, label).clicked() {
        *current = tab;
    }
}

fn grid_form(ui: &mut egui::Ui, id: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, add_contents);
}

fn driver_label(driver: Driver) -> &'static str {
    match driver {
        Driver::Postgres => "PostgreSQL",
        Driver::Sqlite => "SQLite",
        Driver::Mysql => "MySQL",
        Driver::Mariadb => "MariaDB",
    }
}

fn install_target_label(target: InstallTarget) -> &'static str {
    match target {
        InstallTarget::Postgres => "PostgreSQL",
        InstallTarget::Docker => "Docker",
        InstallTarget::Sqlite => "SQLite",
        InstallTarget::Mysql => "MariaDB/MySQL",
    }
}

fn draw_capabilities(ui: &mut egui::Ui, capabilities: &SystemCapabilities) {
    ui.label(format!(
        "Package manager: {}",
        capabilities
            .package_manager
            .map(|pm| format!("{pm:?}"))
            .unwrap_or_else(|| "none detected".to_string())
    ));
    ui.label(format_tool("Docker", &capabilities.docker));
    ui.label(format_tool("psql", &capabilities.postgres_client));
    ui.label(format_tool("createdb", &capabilities.postgres_createdb));
    ui.label(format_tool("postgres", &capabilities.postgres_server));
    ui.label(format_tool("sqlite3", &capabilities.sqlite_cli));
    ui.label(format_tool("mysql", &capabilities.mysql_client));
}

fn format_tool(label: &str, status: &ToolStatus) -> String {
    match status {
        ToolStatus::Available { path } => format!("{label}: available ({})", path.display()),
        ToolStatus::Missing => format!("{label}: missing"),
        ToolStatus::PresentButUnavailable { path, reason } => {
            format!(
                "{label}: installed at {}, but unavailable ({reason})",
                path.display()
            )
        }
    }
}

fn launch_install_commands(commands: &[String]) -> anyhow::Result<()> {
    let script = format!(
        "{}; printf '\\nPress Enter to close...'; read _",
        commands.join(" && ")
    );

    #[cfg(target_os = "linux")]
    {
        let candidates: &[(&str, &[&str])] = &[
            ("x-terminal-emulator", &["-e", "sh", "-lc"]),
            ("gnome-terminal", &["--", "sh", "-lc"]),
            ("konsole", &["-e", "sh", "-lc"]),
            ("kitty", &["sh", "-lc"]),
            ("alacritty", &["-e", "sh", "-lc"]),
            ("wezterm", &["start", "--", "sh", "-lc"]),
            ("foot", &["sh", "-lc"]),
        ];

        for (terminal, args) in candidates {
            let mut command = std::process::Command::new(terminal);
            command.args(*args).arg(&script);
            if command.spawn().is_ok() {
                return Ok(());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", &commands.join(" && ")])
            .spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let osa = format!(
            "tell application \"Terminal\" to do script {:?}",
            commands.join(" && ")
        );
        std::process::Command::new("osascript")
            .args(["-e", &osa])
            .spawn()?;
        return Ok(());
    }

    anyhow::bail!(
        "No supported terminal launcher found. Run these commands manually: {}",
        commands.join(" && ")
    )
}

fn draw_result_table(
    ui: &mut egui::Ui,
    columns: &[String],
    rows: &[Vec<String>],
) -> Option<SelectedCell> {
    if columns.is_empty() {
        ui.label("No result columns.");
        return None;
    }

    let mut selected_cell = None;

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::auto().at_least(80.0).clip(true), columns.len())
        .min_scrolled_height(320.0);

    table
        .header(24.0, |mut header| {
            for column in columns {
                header.col(|ui| {
                    ui.strong(column);
                });
            }
        })
        .body(|body| {
            body.rows(22.0, rows.len(), |mut row| {
                let row_index = row.index();
                for (column_index, column) in columns.iter().enumerate() {
                    row.col(|ui| {
                        let value = rows
                            .get(row_index)
                            .and_then(|cells| cells.get(column_index))
                            .cloned()
                            .unwrap_or_default();
                        let text = rows
                            .get(row_index)
                            .and_then(|cells| cells.get(column_index))
                            .map(|cell| truncate_cell(cell, 256))
                            .unwrap_or_default();
                        if ui
                            .selectable_label(false, egui::RichText::new(text).monospace())
                            .clicked()
                        {
                            selected_cell = Some(SelectedCell {
                                column: column.clone(),
                                row_index,
                                value,
                            });
                        }
                    });
                }
            });
        });

    selected_cell
}

fn truncate_cell(cell: &str, max_chars: usize) -> String {
    if cell.chars().count() <= max_chars {
        return cell.to_string();
    }

    let mut out: String = cell.chars().take(max_chars).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::truncate_cell;

    #[test]
    fn leaves_short_cells_unchanged() {
        assert_eq!(truncate_cell("abc", 10), "abc");
    }

    #[test]
    fn truncates_long_cells_by_char_boundary() {
        assert_eq!(truncate_cell("åβçδε", 3), "åβç…");
    }
}
