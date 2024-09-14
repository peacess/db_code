use std::{fs, fs::File, path, str::FromStr, sync::Arc};

use log::LevelFilter;
use rust_kits::Executor;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Pool, Sqlite,
};

pub struct KitsDb {}

impl KitsDb {
    pub fn uuid() -> String {
        xid::new().to_string()
        // uuid::Uuid::new_v4().to_string()
    }

    pub async fn new_with_name(db_name: &str, sql_file: &str) -> Result<Arc<Pool<Sqlite>>, sqlx::Error> {
        let mut p = path::PathBuf::from(format!("temp/data/{}", db_name));
        let mut init_table = false;
        if !p.exists() {
            // let e = fs::create_dir_all(p.parent().expect(""));
            // let temp = fs::canonicalize(&p.parent().expect("")).expect("");
            // log::info!("{}", temp.to_str().expect(""));
            p = Executor::path();
            log::info!("exe path: {}", p.to_str().expect(""));
            p = p.join("data");
            if !p.exists() {
                fs::create_dir_all(p.as_path()).expect("");
            }
            p = p.join(db_name);
            if !p.exists() {
                File::create(p.as_path()).expect("");
                init_table = true;
            }
        }
        let str = format!("sqlite:{}", p.to_str().expect(""));
        let options = SqlitePoolOptions::new().max_connections(30);
        let mut con_options = SqliteConnectOptions::from_str(&str)?;
        //设置显示sql语句的log level（并不是日志文件的level）, 如果把它设为info， 那么日志文件的输出要设置为info及更高级别（如warn等）才会输出
        con_options = con_options.log_statements(LevelFilter::Debug);
        //语句运行超过一秒，就输出日志
        con_options = con_options.log_slow_statements(LevelFilter::Warn, std::time::Duration::from_secs(1));
        let pool = options.connect_with(con_options).await?;
        let pool = Arc::new(pool);
        if init_table {
            let sql = {
                match fs::read_to_string(sql_file) {
                    Err(err) => {
                        log::error!("{}", err);
                        String::new()
                    }
                    Ok(data) => data,
                }
            };
            sqlx::query(&sql).execute(pool.as_ref()).await?;
        }
        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Utc};

    use crate::dao::{KitsDb, Times};

    #[test]
    fn test_day() {
        let d = Times::day_of_year();
        assert_eq!(d, Local::now().ordinal() as i32);
    }

    #[test]
    fn test_ts() {
        {
            let u = Utc::now();
            let l = u.with_timezone(&Local);
            //时间戳，都是以utc来的
            assert_eq!(u.timestamp_millis(), l.timestamp_millis());

            let ts = u.timestamp_millis();

            let u = Utc.timestamp_millis_opt(ts).unwrap();
            let l = Local.timestamp_millis_opt(ts).unwrap();
            let ustr = u.to_string();
            let lstr = l.to_string();
            assert_ne!(ustr, lstr);
        }
        {
            let f = "%Y-%m-%d %H:%M:%S%.3f";
            let u = Utc::now();
            let l = u.with_timezone(&Local);
            let c = u.with_timezone(&Times::china_offset());

            // let u_str = u.format(f).to_string();
            let c_str = c.format(f).to_string();
            // let l_str = l.format(f).to_string();

            let naive = NaiveDateTime::parse_from_str(&c_str, f).unwrap();
            let c2 = Times::naive_to_china_date(naive);
            let c2_str = c2.format(f).to_string();
            assert_eq!(c2_str, c_str);
            assert_eq!(c.timestamp_millis(), c2.timestamp_millis());
            assert_eq!(c.timestamp_millis(), u.timestamp_millis());
            assert_eq!(c.timestamp_millis(), l.timestamp_millis());
        }
    }

    #[test]
    fn test_id() {
        let id = KitsDb::uuid();
        assert_eq!(20, id.len());
    }
}
