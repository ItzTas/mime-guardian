use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 86_400;

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

pub fn utc(millis: u64) -> String {
    let seconds = millis / 1_000;
    let (year, month, day) = civil(seconds / SECONDS_PER_DAY);
    let of_day = seconds % SECONDS_PER_DAY;

    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60
    )
}

fn civil(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_format_in_utc() {
        assert_eq!(utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(utc(951_782_400_000), "2000-02-29 00:00:00 UTC");
        assert_eq!(utc(1_793_309_445_000), "2026-10-29 21:30:45 UTC");
    }
}
