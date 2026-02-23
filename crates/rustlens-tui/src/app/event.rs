use crate::term;
use rustlens_core::db;

#[derive(Debug)]
pub enum AppEvent {
    Input(term::input::UiEvent),
    Db(db::DbEvt),
}
