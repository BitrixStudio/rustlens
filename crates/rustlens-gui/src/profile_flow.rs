use rustlens_core::db;
use rustlens_core::model::connection::ConnectionProfile;

use crate::{driver_label, helpers, profiles, schema_studio, GuiApp, ProvisionEvt, Tab};

impl GuiApp {
    pub(crate) fn connect_profile(&mut self, index: usize) {
        let Some(profile) = self.profiles.get(index).cloned() else {
            return;
        };

        if !profile.supports_browsing() {
            self.selected_profile = Some(index);
            self.status = format!(
                "{} profiles can be created, but browsing is not supported yet.",
                driver_label(profile.driver)
            );
            return;
        }

        self.selected_profile = Some(index);
        self.connected = false;
        self.schema = profile.schema;
        self.page_size = profile.page_size;
        self.selected_table = None;
        self.page = 0;
        self.columns.clear();
        self.rows.clear();
        self.selected_cell = None;
        self.page_cache.clear();
        self.request_contexts.clear();
        self.latest_result_request_id = None;
        self.loading_status = None;
        self.status = format!("Connecting to {}...", profile.name);
        self.detail_status.clear();

        self.send_cmd(db::DbCmd::Connect {
            database_url: profile.database_url,
        });
    }

    pub(crate) fn refresh_profiles(&mut self) {
        match profiles::load_profiles() {
            Ok(profiles) => {
                self.profiles = profiles;
                self.selected_profile = None;
                self.page_cache.clear();
                self.status = "Profiles reloaded.".to_string();
                self.detail_status = profiles::profiles_path_text();
            }
            Err(err) => {
                self.status = format!("Profile load error: {err}");
            }
        }
    }

    pub(crate) fn save_created_profile(&mut self, profile: ConnectionProfile) {
        match profiles::add_profile(profile) {
            Ok(profile) => {
                let should_connect = profile.supports_browsing();
                self.profiles.push(profile);
                let idx = self.profiles.len() - 1;
                self.selected_profile = Some(idx);
                self.wizard.visible = false;
                self.status = "Profile created.".to_string();
                if should_connect {
                    self.schema_designer = schema_studio::SchemaDesignerState::default();
                    self.connect_profile(idx);
                    self.tab = Tab::Schema;
                    self.detail_status =
                        "Database created. Design a starter schema, then apply it.".to_string();
                } else {
                    self.detail_status =
                        "SQLite profile saved. SQLite browsing is planned but not implemented yet."
                            .to_string();
                }
            }
            Err(err) => {
                self.status = format!("Could not save profile: {err}");
            }
        }
    }

    pub(crate) fn save_manual_profile(&mut self, connect: bool) {
        match helpers::build_manual_profile(&self.wizard.add_existing) {
            Ok(profile) => match profiles::add_profile(profile) {
                Ok(profile) => {
                    self.profiles.push(profile);
                    let idx = self.profiles.len() - 1;
                    self.selected_profile = Some(idx);
                    self.wizard.visible = false;
                    self.status = "Profile saved.".to_string();
                    if connect {
                        self.connect_profile(idx);
                    }
                }
                Err(err) => self.status = format!("Could not save profile: {err}"),
            },
            Err(err) => self.status = err,
        }
    }

    pub(crate) fn create_local_database(&mut self) {
        if self.provisioning {
            return;
        }

        let tx = self.provision_evt_tx.clone();
        let form = self.wizard.create_local.clone();
        self.provisioning = true;
        self.status = "Creating database...".to_string();

        std::thread::spawn(move || {
            let result = helpers::create_profile_with_dbnest(form).map_err(|err| err.to_string());
            let _ = tx.send(ProvisionEvt::Created(result));
        });
    }

    pub(crate) fn delete_profile(&mut self, profile: &profiles::Profile) {
        match profiles::delete_profile(profile.id) {
            Ok(()) => {
                let deleted_selected = self
                    .selected_profile
                    .and_then(|idx| self.profiles.get(idx))
                    .is_some_and(|selected| selected.id == profile.id);
                self.profiles.retain(|item| item.id != profile.id);
                if deleted_selected {
                    self.selected_profile = None;
                    self.connected = false;
                    self.tables.clear();
                    self.selected_table = None;
                    self.columns.clear();
                    self.rows.clear();
                    self.selected_cell = None;
                    self.page_cache.clear();
                    self.request_contexts.clear();
                    self.latest_result_request_id = None;
                    self.loading_status = None;
                }
                self.pending_profile_delete = None;
                self.status = format!("Deleted profile '{}'.", profile.name);
                self.detail_status = if profile.dbnest_instance_id.is_some() {
                    "Only the RustLens profile was deleted; the dbnest instance was not removed."
                        .to_string()
                } else {
                    profiles::profiles_path_text()
                };
            }
            Err(err) => {
                self.status = format!("Could not delete profile: {err}");
            }
        }
    }

    pub(crate) fn save_profile_edit(&mut self) {
        let Some(form) = self.pending_profile_edit.clone() else {
            return;
        };
        match helpers::build_profile_from_edit(&form) {
            Ok(profile) => match profiles::update_profile(profile) {
                Ok(updated) => {
                    if let Some(existing) =
                        self.profiles.iter_mut().find(|item| item.id == updated.id)
                    {
                        *existing = updated.clone();
                    }
                    let edited_selected = self
                        .selected_profile
                        .and_then(|idx| self.profiles.get(idx))
                        .is_some_and(|selected| selected.id == updated.id);
                    if edited_selected {
                        self.schema = updated.schema.clone();
                        self.page_size = updated.page_size;
                    }
                    self.pending_profile_edit = None;
                    self.status = format!("Saved profile '{}'.", updated.name);
                    self.detail_status =
                        "Reconnect if you changed the connection URL or credentials.".to_string();
                }
                Err(err) => self.status = format!("Could not save profile: {err}"),
            },
            Err(err) => self.status = err,
        }
    }
}
