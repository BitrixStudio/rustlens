use eframe::egui;
use rustlens_core::db;

use crate::{CachedPage, GuiApp, ProvisionEvt, RequestContext};

impl GuiApp {
    pub(crate) fn drain_db_events(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        while let Ok(evt) = self.db_evt_rx.try_recv() {
            changed = true;
            self.handle_db_event(evt);
        }

        if changed {
            ctx.request_repaint();
        }
    }

    pub(crate) fn drain_provision_events(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        while let Ok(evt) = self.provision_evt_rx.try_recv() {
            changed = true;
            self.provisioning = false;
            match evt {
                ProvisionEvt::Created(Ok(profile)) => self.save_created_profile(profile),
                ProvisionEvt::Created(Err(err)) => {
                    self.status = format!("Database creation failed: {err}");
                }
            }
        }

        if changed {
            ctx.request_repaint();
        }
    }

    fn handle_db_event(&mut self, evt: db::DbEvt) {
        match evt {
            db::DbEvt::Status(msg) => self.status = msg,
            db::DbEvt::Connected => {
                self.connected = true;
                self.send_cmd(db::DbCmd::LoadSqlMeta {
                    schema: self.schema.clone(),
                });
            }
            db::DbEvt::Error(err) => {
                self.status = format!("Error: {err}");
                self.detail_status.clear();
                self.loading_status = None;
            }
            db::DbEvt::TablesLoaded { tables } => {
                self.tables = tables;
                self.detail_status = format!("{} tables loaded.", self.tables.len());
            }
            db::DbEvt::QueryResult {
                request_id,
                columns,
                rows,
                info,
            } => {
                if !self.is_current_result_request(request_id) {
                    return;
                }
                if let Some(request_id) = request_id {
                    if let Some(RequestContext::TablePage(key)) =
                        self.request_contexts.remove(&request_id)
                    {
                        self.page_cache.insert(
                            key,
                            CachedPage {
                                columns: columns.clone(),
                                rows: rows.clone(),
                            },
                        );
                    }
                }
                self.columns = columns;
                self.rows = rows;
                self.selected_cell = None;
                self.detail_status = info;
                self.loading_status = None;
            }
            db::DbEvt::SqlExecuted { request_id, info } => {
                if !self.is_current_result_request(request_id) {
                    return;
                }
                if let Some(request_id) = request_id {
                    let _ = self.request_contexts.remove(&request_id);
                }
                self.detail_status = info;
                self.loading_status = None;
            }
            db::DbEvt::SqlMetaLoaded {
                schema,
                tables,
                columns,
            } => {
                self.schema = schema;
                self.tables = tables;
                self.sql_columns = columns.into_iter().collect();
                self.page_cache.clear();
                self.loading_status = None;
                self.status = format!("Connected to schema '{}'.", self.schema);
                self.detail_status = format!("{} tables loaded.", self.tables.len());

                if self.selected_table.is_none() {
                    if let Some(table) = self.tables.first().cloned() {
                        self.open_table(table);
                    }
                }
            }
        }
    }

    fn is_current_result_request(&self, request_id: Option<u64>) -> bool {
        request_id.is_none() || request_id == self.latest_result_request_id
    }
}
