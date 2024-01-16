use db_code_macro::{DbDao, DbSub};

mod dao;

#[derive(sqlx::FromRow, DbDao, DbSub)]
pub struct CbTable {
    pub id: String,
    pub update_ts: i64,
    pub version: i64,
}

// #[derive(DbDao)]
// pub struct DbTable3 {
//     #[db_sub(flatten)]
//     dd: DbTable,
// }
