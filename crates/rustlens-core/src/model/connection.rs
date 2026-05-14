use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    #[serde(default = "new_profile_id")]
    pub id: Uuid,
    pub name: String,
    pub driver: Driver,

    pub database_url: String,

    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub page_size: Option<i64>,

    #[serde(default)]
    pub source: ProfileSource,
    #[serde(default)]
    pub dbnest_instance_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Postgres,
    Sqlite,
    Mysql,
    Mariadb,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileSource {
    #[default]
    Manual,
    Dbnest,
    Imported,
}

impl ConnectionProfile {
    pub fn schema_or_default(&self) -> String {
        self.schema.clone().unwrap_or_else(|| "public".to_string())
    }

    pub fn page_size_or_default(&self) -> i64 {
        self.page_size.unwrap_or(200)
    }

    pub fn supports_browsing(&self) -> bool {
        self.driver == Driver::Postgres
    }
}

fn new_profile_id() -> Uuid {
    Uuid::new_v4()
}
