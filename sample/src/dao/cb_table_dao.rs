use std::sync::Arc;

use db_code::dao::{Dao, KitsDb, Times};
use sqlx::{Pool, Sqlite, SqlitePool};
use sqlx::query::Query;
use sqlx::sqlite::SqliteArguments;

use crate::CbTable;

#[derive(Debug)]
pub struct
CbTableDao {
    pool: Arc<Pool<Sqlite>>,
}

impl CbTableDao {
    pub const TT: &'static str = "cb_table";
    pub const T_ID: &'static str = "id";
    pub const T_UPDATE_TS: &'static str = "update_ts";
    pub const T_VERSION: &'static str = "version";
    pub(super) const fn columns_no_id() -> [&'static str; 2usize] {
        [CbTableDao::T_UPDATE_TS, CbTableDao::T_VERSION, ]
    }
    pub(super) fn columns() -> String {
        format!("{},{}", CbTableDao::T_ID, CbTableDao::columns_no_id().join(","))
    }
    fn sql_get() -> String {
        format!("SELECT {} FROM {} WHERE {} = ?", CbTableDao::columns(), CbTableDao
        ::TT, CbTableDao::T_ID)
    }
    fn sql_add() -> String {
        let vs = ["?"; CbTableDao::columns_no_id().len() + 1];
        format!("insert into {}({}) values ({})", CbTableDao::TT, CbTableDao::
        columns(), vs.join(","))
    }
    fn sql_remove() -> String {
        format!("delete from {} where {} = ?", CbTableDao::TT, CbTableDao::T_ID)
    }
    fn sql_remove_all() -> String { format!("delete from {}", CbTableDao::TT) }
    fn sql_update() -> String {
        let vs = CbTableDao::columns_no_id().join(" = ?, ") + " = ?";
        format!("update {} set {} where {} = ?", CbTableDao::TT, vs, CbTableDao::
        T_ID)
    }
    fn sql_update_ol() -> String {
        let mut vs = CbTableDao::columns_no_id().join(" = ?, ") + " = ?";
        vs = vs.replace("version = ?", "version = version + 1");
        format!("update {} set {} where {} = ? and {} = ?", CbTableDao::TT, vs,
                CbTableDao::T_ID, CbTableDao::T_VERSION)
    }
    fn sql_list() -> String
    {
        format!("select {} from {}", CbTableDao::columns(), CbTableDao::TT)
    }
    pub(super) fn _bind_add<'a>
    (m: &'a CbTable, q: Query<'a, Sqlite, SqliteArguments<'a>>) ->
    Query<'a, Sqlite, SqliteArguments<'a>>
    { q.bind(&m.id).bind(&m.update_ts).bind(&m.version) }
    pub(super) fn
    _bind_update<'a>
    (m: &'a CbTable, q: Query<'a, Sqlite, SqliteArguments<'a>>) ->
    Query<'a, Sqlite, SqliteArguments<'a>>
    { q.bind(&m.update_ts).bind(&m.version).bind(&m.id) }
    pub(super) fn
    _bind_update_ol<'a>
    (m: &'a CbTable, q: Query<'a, Sqlite, SqliteArguments<'a>>) ->
    Query<'a, Sqlite, SqliteArguments<'a>>
    { q.bind(&m.update_ts).bind(&m.id).bind(&m.version) }
}

impl Dao<CbTable> for CbTableDao
{
    fn pool(&self) -> &SqlitePool { self.pool.as_ref() }
    fn
    new(pool: Arc<SqlitePool>) -> Self { Self { pool } }
    async fn
    add(&self, m: &mut CbTable) -> Result<u64, sqlx::Error> {
        if m.id.is_empty() { m.id = KitsDb::uuid(); }
        if m.update_ts < 1
        { m.update_ts = Times::ts_now(); }
        let sql = Self::sql_add();
        let re = Self::
        _bind_add(m, sqlx::query(&sql)).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }
    async fn remove(&self, id: &str) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_remove();
        let re = sqlx::
        query(&sql).bind(id).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }
    async fn remove_all(&self) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_remove_all();
        let re = sqlx::
        query(&sql).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }
    async fn update(&self, m: &mut CbTable) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_update();
        m.update_ts = Times::ts_now();
        let re = Self::_bind_update(m, sqlx::query(&sql)).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }
    async fn update_ol(&self, m: &mut CbTable) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_update_ol();
        m.update_ts = Times::ts_now();
        let re = Self::
        _bind_update_ol(m, sqlx::query(&sql)).execute(self.pool()).await?
            ;
        Ok(re.rows_affected())
    }
    async fn get(&self, id: &str) -> Result<Option<CbTable>, sqlx::Error> {
        let sql = Self::sql_get();
        let condition = sqlx::query_as::<_,
            CbTable>(&sql).bind(id).fetch_optional(self.pool()).await?;
        Ok(condition)
    }
    async fn list(&self) -> Result<Vec<CbTable>, sqlx::Error> {
        let sql = Self::sql_list();
        let rows = sqlx::
        query_as(&sql).fetch_all(self.pool()).await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests
{
    use db_code::dao::{Dao, KitsDb};

    use crate::CbTable;

    #[tokio::test]
    async fn cb_table_dao() {
        let pool = KitsDb::new_with_name("cb_table_dao.db", "init/sql.sql").await.expect("");
        let dao_ = CbTableDao::new(pool);
        let mut m = CbTable::default()
            ;
        {
            dao_.remove_all().await.expect("");
            dao_.remove(&m.id).await.expect("");
        }
        {
            let get_m = dao_.get(&m.id).await.expect("");
            assert_eq!(true, get_m.is_none());
        }
        m.update_ts = 1;
        {
            let re = dao_.add(&mut m).await.expect("");
            assert_eq!(re, 1);
            let get_m = dao_.get(&m.id).await.expect("").expect("");
            assert_eq!(m.id, get_m.id);
            assert_eq!(m.version, get_m.version);
            assert_eq!(m.update_ts, get_m.update_ts);
        }
        {
            let re = dao_.update(&mut m).await.expect("");
            assert_eq!(1, re);
            let get_m = dao_.get(&m.id).await.expect("").expect("")
                ;
            assert_eq!(m.id, get_m.id);
            assert_eq!(m.version + 1, get_m.version);
            assert_eq!(m.update_ts, get_m.update_ts);
        }
        {
            let ms = dao_.list().await.expect("");
            assert_eq!(1, ms.len());
            let new_m = &ms[0];
            assert_eq!(m.id, new_m.id);
        }
        {
            let re = dao_.remove(&m.id).await.expect("");
            assert_eq!(1, re)
            ;
            let old = dao_.get(&m.id).await.expect("");
            assert_eq!(true, old.is_none());
        }
    }
}