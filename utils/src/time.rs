use std::time::{Duration, SystemTime};

pub fn unix_epoch_now() -> i64 {
    unix_epoch_now_as_duration().as_secs() as i64
}

pub fn unix_epoch_now_as_duration() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

pub fn duration_in_years(years: u64) -> Duration {
    Duration::from_secs(years * 365 * 24 * 60 * 60)
}
