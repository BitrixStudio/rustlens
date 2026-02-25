#[derive(Debug)]
pub enum DbCmd {
    Connect {
        database_url: String,
    },
    LoadTables {
        schema: String,
    },
    LoadTablePage {
        schema: String,
        table: String,
        page: i64,
        page_size: i64,
    },
    ExecuteSql {
        sql: String,
    },
    LoadSqlMeta {
        schema: String,
    },
}

#[derive(Debug)]
pub enum DbEvt {
    Status(String),
    Error(String),

    TablesLoaded {
        tables: Vec<String>,
    },

    QueryResult {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        info: String,
    },

    SqlExecuted {
        info: String,
    },

    SqlMetaLoaded {
        schema: String,
        tables: Vec<String>,
        columns: Vec<(String, Vec<String>)>,
    },
}
