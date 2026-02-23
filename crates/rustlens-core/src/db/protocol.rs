#[derive(Debug)]
pub enum DbCmd {
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
}
