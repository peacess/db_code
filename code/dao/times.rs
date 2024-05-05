use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDateTime, TimeZone, Utc};

pub struct Times;

impl Times {
    /// 毫秒
    #[inline]
    pub fn ts_now() -> i64 {
        Utc::now().timestamp_millis()
    }

    #[inline]
    pub fn to_ts(d: &DateTime<Local>) -> i64 {
        d.timestamp_millis()
    }

    pub fn to_local_date(ts: i64) -> DateTime<Local> {
        Local.timestamp_millis_opt(ts).single().expect("")
    }

    pub fn to_utc_date(ts: i64) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(ts).single().expect("")
    }

    pub fn to_china_date(ts: i64) -> DateTime<FixedOffset> {
        let u = Utc.timestamp_millis_opt(ts).single().expect("");
        let of = FixedOffset::east_opt(8 * 3600).unwrap();
        u.with_timezone(&of)
    }

    pub fn naive_to_china_date(n: NaiveDateTime) -> DateTime<FixedOffset> {
        let of = Self::china_offset();
        of.timestamp_millis_opt(n.and_utc().timestamp_millis() - of.local_minus_utc() as i64 * 1000)
            .unwrap()
    }

    pub fn china_offset() -> FixedOffset {
        FixedOffset::east_opt(8 * 3600).unwrap()
    }

    ///
    pub fn day_of_year() -> i32 {
        let t = Local::now().ordinal();
        t as i32
    }

    ///
    pub fn day_of_year_(d: &DateTime<Local>) -> i32 {
        let t = d.ordinal();
        t as i32
    }
}
