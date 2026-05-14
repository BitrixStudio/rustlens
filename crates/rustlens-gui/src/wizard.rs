use eframe::egui;
use rustlens_core::db;
use rustlens_core::provision::capabilities::ToolStatus;
use rustlens_core::provision::installer::{install_commands, InstallTarget};

use crate::{
    draw_capabilities, grid_form, helpers, install_target_label, launch_install_commands, profiles,
    theme, CreateEngine, GuiApp, InstallConfirmation, WizardStep,
};

impl GuiApp {
    pub(crate) fn draw_setup_wizard(&mut self, ctx: &egui::Context) {
        if !self.wizard.visible {
            return;
        }

        let mut visible = self.wizard.visible;
        egui::Window::new("RustLens Setup")
            .open(&mut visible)
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .show(ctx, |ui| match self.wizard.step {
                WizardStep::Welcome => self.draw_wizard_welcome(ui),
                WizardStep::AddExisting => self.draw_wizard_add_existing(ui),
                WizardStep::CreateLocal => self.draw_wizard_create_local(ui),
                WizardStep::Install => self.draw_wizard_install(ui),
            });
        self.wizard.visible = visible;
    }

    fn draw_wizard_welcome(&mut self, ui: &mut egui::Ui) {
        ui.heading("Welcome to RustLens");
        ui.label("Create or add a database profile to get started.");
        ui.small(profiles::profiles_path_text());
        ui.add_space(12.0);

        if theme::primary_button(ui, "Add existing PostgreSQL database").clicked() {
            self.wizard.step = WizardStep::AddExisting;
        }
        if theme::primary_button(ui, "Create local database").clicked() {
            self.wizard.step = WizardStep::CreateLocal;
        }
        if theme::secondary_button(ui, "Install database tools").clicked() {
            self.capabilities =
                rustlens_core::provision::capabilities::detect_system_capabilities();
            self.wizard.step = WizardStep::Install;
        }
    }

    fn draw_wizard_add_existing(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add Existing PostgreSQL Database");
        ui.checkbox(
            &mut self.wizard.add_existing.advanced_url,
            "Use connection URL",
        );

        ui.horizontal(|ui| {
            ui.label("Profile name");
            ui.text_edit_singleline(&mut self.wizard.add_existing.name);
        });

        if self.wizard.add_existing.advanced_url {
            ui.label("Database URL");
            ui.text_edit_singleline(&mut self.wizard.add_existing.database_url);
        } else {
            grid_form(ui, "add_existing_grid", |ui| {
                ui.label("Host");
                ui.text_edit_singleline(&mut self.wizard.add_existing.host);
                ui.end_row();
                ui.label("Port");
                ui.text_edit_singleline(&mut self.wizard.add_existing.port);
                ui.end_row();
                ui.label("Database");
                ui.text_edit_singleline(&mut self.wizard.add_existing.database);
                ui.end_row();
                ui.label("User");
                ui.text_edit_singleline(&mut self.wizard.add_existing.user);
                ui.end_row();
                ui.label("Password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.wizard.add_existing.password)
                        .password(true),
                );
                ui.end_row();
            });
        }

        grid_form(ui, "add_existing_options_grid", |ui| {
            ui.label("Schema");
            ui.text_edit_singleline(&mut self.wizard.add_existing.schema);
            ui.end_row();
            ui.label("Page size");
            ui.text_edit_singleline(&mut self.wizard.add_existing.page_size);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Back").clicked() {
                self.wizard.step = WizardStep::Welcome;
            }
            if theme::secondary_button(ui, "Test connection").clicked() {
                match helpers::build_manual_profile(&self.wizard.add_existing) {
                    Ok(profile) => {
                        self.schema = profile.schema_or_default();
                        self.page_size = profile.page_size_or_default();
                        self.status = "Testing connection...".to_string();
                        self.send_cmd(db::DbCmd::Connect {
                            database_url: profile.database_url,
                        });
                    }
                    Err(err) => self.status = err,
                }
            }
            if theme::secondary_button(ui, "Save").clicked() {
                self.save_manual_profile(false);
            }
            if theme::primary_button(ui, "Save and connect").clicked() {
                self.save_manual_profile(true);
            }
        });
    }

    fn draw_wizard_create_local(&mut self, ui: &mut egui::Ui) {
        ui.heading("Create Local Database");
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.wizard.create_local.engine,
                CreateEngine::PostgresDocker,
                "PostgreSQL Docker",
            );
            ui.radio_value(
                &mut self.wizard.create_local.engine,
                CreateEngine::Sqlite,
                "SQLite",
            );
            ui.add_enabled_ui(false, |ui| {
                ui.radio_value(
                    &mut self.wizard.create_local.engine,
                    CreateEngine::Mysql,
                    "MySQL/MariaDB",
                );
            });
        });

        ui.add_space(8.0);
        match self.wizard.create_local.engine {
            CreateEngine::PostgresDocker => self.draw_create_postgres_form(ui),
            CreateEngine::Sqlite => self.draw_create_sqlite_form(ui),
            CreateEngine::Mysql => {
                ui.label("MySQL/MariaDB provisioning is planned but not implemented yet.");
            }
        }
    }

    fn draw_create_postgres_form(&mut self, ui: &mut egui::Ui) {
        match &self.capabilities.docker {
            ToolStatus::Available { .. } => ui.label("Docker is available."),
            ToolStatus::Missing => {
                ui.colored_label(egui::Color32::YELLOW, "Docker is not installed.")
            }
            ToolStatus::PresentButUnavailable { reason, .. } => ui.colored_label(
                egui::Color32::YELLOW,
                format!("Docker is installed but unavailable: {reason}"),
            ),
        };

        grid_form(ui, "create_pg_grid", |ui| {
            ui.label("Profile name");
            ui.text_edit_singleline(&mut self.wizard.create_local.profile_name);
            ui.end_row();
            ui.label("Database");
            ui.text_edit_singleline(&mut self.wizard.create_local.database);
            ui.end_row();
            ui.label("User");
            ui.text_edit_singleline(&mut self.wizard.create_local.user);
            ui.end_row();
            ui.label("Password");
            ui.add(
                egui::TextEdit::singleline(&mut self.wizard.create_local.password).password(true),
            );
            ui.end_row();
            ui.label("Image");
            ui.text_edit_singleline(&mut self.wizard.create_local.image);
            ui.end_row();
            ui.label("Schema");
            ui.text_edit_singleline(&mut self.wizard.create_local.schema);
            ui.end_row();
            ui.label("Page size");
            ui.text_edit_singleline(&mut self.wizard.create_local.page_size);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Back").clicked() {
                self.wizard.step = WizardStep::Welcome;
            }
            let can_create = matches!(self.capabilities.docker, ToolStatus::Available { .. })
                && !self.provisioning;
            if ui
                .add_enabled(can_create, egui::Button::new("Create and connect"))
                .clicked()
            {
                self.create_local_database();
            }
        });
    }

    fn draw_create_sqlite_form(&mut self, ui: &mut egui::Ui) {
        ui.colored_label(
            egui::Color32::YELLOW,
            "SQLite creation is supported through dbnest, but RustLens SQLite browsing is not implemented yet.",
        );
        grid_form(ui, "create_sqlite_grid", |ui| {
            ui.label("Profile name");
            ui.text_edit_singleline(&mut self.wizard.create_local.profile_name);
            ui.end_row();
            ui.label("Path (blank = dbnest managed)");
            ui.text_edit_singleline(&mut self.wizard.create_local.sqlite_path);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Back").clicked() {
                self.wizard.step = WizardStep::Welcome;
            }
            if ui
                .add_enabled(
                    !self.provisioning,
                    egui::Button::new("Create SQLite profile"),
                )
                .clicked()
            {
                self.create_local_database();
            }
        });
    }

    fn draw_wizard_install(&mut self, ui: &mut egui::Ui) {
        ui.heading("Install Database Tools");
        ui.label("RustLens can launch installer commands after confirmation. It never captures sudo passwords.");
        ui.add_space(8.0);
        draw_capabilities(ui, &self.capabilities);
        ui.separator();

        if let Some(manager) = self.capabilities.package_manager {
            for target in [
                InstallTarget::Postgres,
                InstallTarget::Docker,
                InstallTarget::Sqlite,
                InstallTarget::Mysql,
            ] {
                let commands = install_commands(manager, target);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("Install {}", install_target_label(target)));
                        if theme::secondary_button(ui, "Show command").clicked() {
                            self.wizard.install_confirmation = Some(InstallConfirmation {
                                target,
                                commands: commands.clone(),
                            });
                        }
                    });
                    for command in &commands {
                        ui.monospace(command);
                    }
                });
            }
        } else {
            ui.label("No supported package manager detected. Install PostgreSQL/Docker manually.");
        }

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Refresh detection").clicked() {
                self.capabilities =
                    rustlens_core::provision::capabilities::detect_system_capabilities();
            }
            if theme::secondary_button(ui, "Back").clicked() {
                self.wizard.step = WizardStep::Welcome;
            }
        });

        if let Some(confirm) = self.wizard.install_confirmation.clone() {
            ui.separator();
            ui.heading(format!(
                "Confirm {} install",
                install_target_label(confirm.target)
            ));
            ui.label("These commands will be launched in an external terminal:");
            for command in &confirm.commands {
                ui.monospace(command);
            }
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, "Cancel").clicked() {
                    self.wizard.install_confirmation = None;
                }
                if theme::danger_button(ui, "Launch installer").clicked() {
                    match launch_install_commands(&confirm.commands) {
                        Ok(()) => self.status = "Installer launched.".to_string(),
                        Err(err) => self.status = format!("Could not launch installer: {err}"),
                    }
                    self.wizard.install_confirmation = None;
                }
            });
        }
    }
}
