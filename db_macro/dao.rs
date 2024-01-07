use std::{fs, path};
use std::io::Write;
use std::ops::Add;

use proc_macro_roids::IdentExt;
use quote::{format_ident, quote};

use super::db_meta::TableMeta;
use super::kits::to_snake_name;

pub fn generate_dao(tm: &TableMeta) {
    let de = tm.derive_input.as_ref().expect("the derive_input is None");
    let m_name = de.ident.clone();
    let dao_name = m_name.clone().append("Dao");
    let table_name = to_snake_name(&tm.type_name);
    let test_name = syn::Ident::new(&to_snake_name(&dao_name.to_string()), m_name.span());
    let db_name = to_snake_name(&dao_name.to_string()).add(".db");
    let no_id: Vec<proc_macro2::TokenStream> = {
        tm.col_names[1..]
            .iter()
            .map(|col| {
                let id = format_ident!("T_{}", col.to_uppercase());
                // let id = syn::Ident::new(&v, m_name.span());
                quote! { #dao_name::#id}
            })
            .collect()
    };
    let no_id_len = no_id.len();

    let const_col_names: Vec<proc_macro2::TokenStream> = {
        tm.col_names
            .iter()
            .map(|col| {
                let id = format_ident!("T_{}", col.to_uppercase());
                // let id = syn::Ident::new(&v, m_name.span());
                quote! {pub const #id : &'static str = #col}
            })
            .collect()
    };

    let add_bind = {
        let mut t = Vec::new();
        let id = syn::Ident::new(&tm.col_names[0], m_name.span());
        t.push(quote!(q.bind(&m.#id)));
        tm.col_names[1..].iter().for_each(|col| {
            // let id = syn::Ident::new(col, m_name.span());
            let id = format_ident!("{}", col);
            t.push(quote!(.bind(&m.#id)))
        });
        t
    };

    let update_bind = {
        let mut t = Vec::new();
        let id = syn::Ident::new(&tm.col_names[1], m_name.span());
        t.push(quote!(q.bind(&m.#id)));
        tm.col_names[2..].iter().for_each(|col| {
            // let id = syn::Ident::new(col, m_name.span());
            let id = format_ident!("{}", col);
            t.push(quote!(.bind(&m.#id)));
        });
        // let id = syn::Ident::new(&tm.col_names[0], m_name.span());
        let id = format_ident!("{}", &tm.col_names[0]);
        t.push(quote!(.bind(&m.#id)));
        t
    };
    let update_bind_ol = {
        let mut t = Vec::new();
        // let id = syn::Ident::new(&tm.col_names[1], m_name.span());
        let id = format_ident!("{}", &tm.col_names[1]);
        t.push(quote!(q.bind(&m.#id)));
        tm.col_names[2..].iter().for_each(|col| {
            if col != "version" {
                // let id = syn::Ident::new(col, m_name.span());
                let id = format_ident!("{}", col);
                t.push(quote!(.bind(&m.#id)));
            }
        });
        // let id = syn::Ident::new(&tm.col_names[0], m_name.span());
        let id = format_ident!("{}", &tm.col_names[0]);
        t.push(quote!(.bind(&m.#id)));
        // let id = syn::Ident::new("version", m_name.span());
        let id = format_ident!("version");
        t.push(quote!(.bind(&m.#id)));
        t
    };

    let gen = quote!(
use std::sync::Arc;

use sqlx::{Pool, Sqlite, SqlitePool};
use sqlx::query::Query;
use sqlx::sqlite::SqliteArguments;

use db_code::dao::{Dao, KitsDb, Times};

use crate::#m_name;

#[derive(Debug)]
pub struct #dao_name {
    pool: Arc<Pool<Sqlite>>,
}

impl #dao_name {
    pub const TT: &'static str = #table_name;
    #(#const_col_names;)*

    pub(super) const fn columns_no_id() -> [&'static str; #no_id_len] {
        [
            #(#no_id,)*
        ]
    }

    pub(super) fn columns() -> String {
        format!("{},{}", #dao_name::T_ID, #dao_name::columns_no_id().join(","))
    }

    fn sql_get() -> String {
        format!("SELECT {} FROM {} WHERE {} = ?", #dao_name::columns(), #dao_name::TT, #dao_name::T_ID)
    }

    fn sql_add() -> String {
        let vs = ["?"; #dao_name::columns_no_id().len() + 1];
        format!("insert into {}({}) values ({})",#dao_name::TT, #dao_name::columns(), vs.join(","))
    }

    fn sql_remove() -> String {
        format!("delete from {} where {} = ?", #dao_name::TT, #dao_name::T_ID)
    }

    fn sql_remove_all() -> String {
        format!("delete from {}", #dao_name::TT)
    }

    fn sql_update() -> String {
        let vs = #dao_name::columns_no_id().join(" = ?, ") + " = ?";
        format!("update {} set {} where {} = ?", #dao_name::TT, vs, #dao_name::T_ID)
    }

    fn sql_update_ol() -> String {
        let mut vs = #dao_name::columns_no_id().join(" = ?, ") + " = ?";
        vs = vs.replace("version = ?","version = version + 1");
        format!("update {} set {} where {} = ? and {} = ?", #dao_name::TT, vs, #dao_name::T_ID, #dao_name::T_VERSION)
    }

    fn sql_list() -> String {
        format!("select {} from {}", #dao_name::columns(), #dao_name::TT)
    }
    pub(super) fn _bind_add<'a>(m: &'a #m_name, q: Query<'a, Sqlite, SqliteArguments<'a>>) -> Query<'a, Sqlite, SqliteArguments<'a>> {
        #(#add_bind)*
    }
    pub(super) fn _bind_update<'a>(m: &'a #m_name, q: Query<'a, Sqlite, SqliteArguments<'a>>) -> Query<'a, Sqlite, SqliteArguments<'a>> {
        #(#update_bind)*
    }
    pub(super) fn _bind_update_ol<'a>(m: &'a #m_name, q: Query<'a, Sqlite, SqliteArguments<'a>>) -> Query<'a, Sqlite, SqliteArguments<'a>> {
        #(#update_bind_ol)*
    }
}

impl Dao<#m_name> for #dao_name  {
    fn pool(&self) -> &SqlitePool {
        self.pool.as_ref()
    }

    fn new(pool: Arc<SqlitePool>) -> Self {
        Self  {
            pool
        }
    }

    async fn add(&self, m: &mut #m_name) -> Result<u64, sqlx::Error> {
        if m.id.is_empty() {
            m.id = KitsDb::uuid();
        }
        if m.update_ts < 1 {
            m.update_ts = Times::ts_now();
        }
        let sql = Self::sql_add();
        let re = Self::_bind_add(m, sqlx::query(&sql)).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }

    async fn remove(&self, id: &str) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_remove();
        let re = sqlx::query(&sql).bind(id).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }

    async fn remove_all(&self) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_remove_all();
        let re = sqlx::query(&sql).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }

    async fn update(&self, m: &mut #m_name) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_update();
        m.update_ts = Times::ts_now();
        let re = Self::_bind_update(m, sqlx::query(&sql)).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }

    async fn update_ol(&self, m: &mut #m_name) -> Result<u64, sqlx::Error> {
        let sql = Self::sql_update_ol();
        m.update_ts = Times::ts_now();
        let re = Self::_bind_update_ol(m, sqlx::query(&sql)).execute(self.pool()).await?;
        Ok(re.rows_affected())
    }

    async fn get(&self, id: &str) -> Result<Option<#m_name>, sqlx::Error> {
        let sql = Self::sql_get();
        let condition = sqlx::query_as::<_, #m_name>(&sql).bind(id).fetch_optional(self.pool()).await?;
        Ok(condition)
    }

    async fn list(&self) -> Result<Vec<#m_name>, sqlx::Error> {
        let sql = Self::sql_list();
        let rows = sqlx::query_as(&sql).fetch_all(self.pool()).await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use self::super::#dao_name;
    use crate::#m_name;
    use db_code::dao::{Dao, KitsDb};

    #[tokio::test]
    async fn #test_name() {
        let pool = KitsDb::new_with_name(#db_name, "init/sql.sql")
            .await
            .expect("");
        let dao_ = #dao_name::new(pool);
        let mut m = #m_name::default();
        {
            //清除数据，可以反复测试
            dao_.remove_all().await.expect("");
            dao_.remove(&m.id).await.expect("");
        }
        {
            let get_m = dao_.get(&m.id).await.expect("");
            assert_eq!(true, get_m.is_none());
        }

        m.update_ts = 1;

        {
            //add
            let re = dao_.add(&mut m).await.expect("");
            assert_eq!(re, 1);
            let get_m = dao_.get(&m.id).await.expect("").expect("");
            assert_eq!(m.id, get_m.id);
            assert_eq!(m.version, get_m.version);
            assert_eq!(m.update_ts, get_m.update_ts);
        }

        {
            //update
            let re = dao_.update(&mut m).await.expect("");
            assert_eq!(1, re);
            let get_m = dao_.get(&m.id).await.expect("").expect("");
            assert_eq!(m.id, get_m.id);
            assert_eq!(m.version, get_m.version);
            assert_eq!(m.update_ts, get_m.update_ts);
        }

        {
            //update ol
            let re = dao_.update(&mut m).await.expect("");
            assert_eq!(1, re);
            let get_m = dao_.get(&m.id).await.expect("").expect("");
            assert_eq!(m.id, get_m.id);
            assert_eq!(m.version + 1, get_m.version);
            assert_eq!(m.update_ts, get_m.update_ts);
        }

        {
            //list
            let ms = dao_.list().await.expect("");
            assert_eq!(1, ms.len());
            let new_m = &ms[0];
            assert_eq!(m.id, new_m.id);
        }

        {
            //remove
            let re = dao_.remove(&m.id).await.expect("");
            assert_eq!(1, re);
            let old = dao_.get(&m.id).await.expect("");
            assert_eq!(true, old.is_none());
        }
    }
}
    );

    let file_name = get_dap_path(&to_snake_name(&tm.type_name).add("_dao.rs"));
    if fs::metadata(file_name.clone()).is_err() {
        let mut file = fs::File::create(file_name).expect("fs::File::create(file_name)");
        let _ = file.write_all(gen.to_string().as_bytes());
    } else {
        //file exist, do nothing
    }
}

fn get_dap_path(short_name: &str) -> String {
    const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
    let mut cur = "dao".to_owned();
    if let Ok(p) = std::env::var(CARGO_MANIFEST_DIR) {
        let p = path::Path::new(p.as_str()).join("src").join(cur);
        cur = p.to_str().expect("cur = p.to_str().expect").to_owned();
    }

    if fs::metadata(cur.as_str()).is_err() {
        let _ = fs::create_dir(cur.as_str());
    }
    let full = path::Path::new(cur.as_str()).join(short_name);
    return full.to_str().expect("full.to_str().").to_owned();
}
