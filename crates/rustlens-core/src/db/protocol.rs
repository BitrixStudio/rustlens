#[derive(Debug)]
pub enum DbCmd {
    Connect {
        database_url: String,
    },
    LoadTables {
        schema: String,
    },
    LoadTablePage {
        request_id: Option<u64>,
        schema: String,
        table: String,
        page: i64,
        page_size: i64,
    },
    ExecuteSql {
        request_id: Option<u64>,
        sql: String,
    },
    ExecuteSqlBatch {
        request_id: Option<u64>,
        statements: Vec<String>,
        refresh_schema: Option<String>,
    },
    LoadSqlMeta {
        schema: String,
    },
}

#[derive(Debug)]
pub enum DbEvt {
    Status(String),
    Connected,
    Error(String),

    TablesLoaded {
        tables: Vec<String>,
    },

    QueryResult {
        request_id: Option<u64>,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        info: String,
    },

    SqlExecuted {
        request_id: Option<u64>,
        info: String,
    },

    SqlMetaLoaded {
        schema: String,
        tables: Vec<String>,
        columns: Vec<(String, Vec<String>)>,
    },
}
