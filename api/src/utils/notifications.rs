use chrono::{DateTime, Utc};

pub fn calculate_recipient_time(utc_time: DateTime<Utc>, offset: &str) -> DateTime<Utc> {
	let offset_int = offset_to_int(offset);
	let offset_duration = chrono::Duration::seconds(offset_int);
	utc_time + offset_duration
}

pub fn offset_to_int(offset: &str) -> i64 {
	let offset_parts: Vec<&str> = offset.split(':').collect();
	if offset_parts.len() == 2 {
		let hours: i64 = offset_parts[0].parse().unwrap_or(0);
		let minutes: i64 = offset_parts[1].parse().unwrap_or(0);
		hours * 3600 + minutes * 60
	} else {
		0
	}
}
