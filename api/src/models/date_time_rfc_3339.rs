use chrono::DateTime;
use chrono::FixedOffset;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Decode;
use sqlx::Encode;
use sqlx::Postgres;
use sqlx::Type;
use sqlx::postgres::PgTypeInfo;
use sqlx::postgres::PgValueRef;

/// A newtype wrapper around [DateTime<FixedOffset>] that (de)serializes
/// timestamps to RFC 3339 and ISO 8601 formatting standard.
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct DateTimeRfc3339(DateTime<FixedOffset>);

impl DateTimeRfc3339 {
	/// Create a new [DateTimeRfc3339] from a [DateTime<FixedOffset>].
	pub fn new(dt: DateTime<FixedOffset>) -> Self {
		DateTimeRfc3339(dt)
	}

	/// Access the inner [DateTime<FixedOffset>].
	pub fn inner(&self) -> &DateTime<FixedOffset> {
		&self.0
	}

	/// Consume and return the inner [DateTime<FixedOffset>].
	pub fn into_inner(self) -> DateTime<FixedOffset> {
		self.0
	}
}

impl From<DateTime<FixedOffset>> for DateTimeRfc3339 {
	fn from(dt: DateTime<FixedOffset>) -> Self {
		DateTimeRfc3339(dt)
	}
}

impl From<DateTimeRfc3339> for DateTime<FixedOffset> {
	fn from(dt: DateTimeRfc3339) -> Self {
		dt.0
	}
}

impl Serialize for DateTimeRfc3339 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.inner().to_rfc3339())
	}
}

impl<'de> Deserialize<'de> for DateTimeRfc3339 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;

		DateTime::parse_from_rfc3339(&s)
			.map(DateTimeRfc3339)
			.map_err(serde::de::Error::custom)
	}
}

impl<'q> Encode<'q, Postgres> for DateTimeRfc3339 {
	fn encode_by_ref(
		&self,
		buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
	) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
		self.inner().encode_by_ref(buf)
	}
}

impl<'r> Decode<'r, Postgres> for DateTimeRfc3339 {
	fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
		let dt = <DateTime<FixedOffset> as Decode<'r, Postgres>>::decode(value)?;
		Ok(DateTimeRfc3339(dt))
	}
}

impl Type<Postgres> for DateTimeRfc3339 {
	fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
		PgTypeInfo::with_name("TIMESTAMPTZ")
	}
}

#[cfg(test)]
mod tests {
	use chrono::TimeZone;
	use chrono::Utc;
	use proptest::prelude::*;
	use serde_json::from_str;
	use serde_json::to_string;

	use super::*;

	#[test]
	fn test_from_implementations() {
		let now = Utc::now().fixed_offset();

		// From<DateTime<FixedOffset>> for DateTimeRfc3339.
		let wrapper: DateTimeRfc3339 = now.into();
		assert_eq!(wrapper.inner(), &now);

		// From<DateTimeRfc3339> for DateTime<FixedOffset>.
		let dt: DateTime<FixedOffset> = wrapper.into();
		assert_eq!(dt, now);
	}

	#[test]
	fn test_serialize() {
		let fixed_dt = FixedOffset::east_opt(3600)
			.unwrap()
			.with_ymd_and_hms(2023, 6, 15, 12, 30, 45)
			.unwrap();

		let wrapper = DateTimeRfc3339::new(fixed_dt);
		let serialized = to_string(&wrapper).unwrap();

		assert_eq!(serialized, "\"2023-06-15T12:30:45+01:00\"");
	}

	#[test]
	fn test_deserialize() {
		let json_str = "\"2023-06-15T12:30:45+01:00\"";
		let wrapper: DateTimeRfc3339 = from_str(json_str).unwrap();

		let expected = FixedOffset::east_opt(3600)
			.unwrap()
			.with_ymd_and_hms(2023, 6, 15, 12, 30, 45)
			.unwrap();

		assert_eq!(wrapper.inner(), &expected);
	}

	#[test]
	fn test_serialize_deserialize_roundtrip() {
		// Create a datetime.
		let dt = Utc::now().fixed_offset();
		let original = DateTimeRfc3339::new(dt);

		// Serialize, and then deserialize.
		let serialized = to_string(&original).unwrap();
		let deserialized: DateTimeRfc3339 = from_str(&serialized).unwrap();

		assert_eq!(original, deserialized);
	}

	#[test]
	fn test_deserialize_invalid_format() {
		// Test invalid format.
		let result = from_str::<DateTimeRfc3339>("\"not-a-date\"");
		assert!(result.is_err());

		// Test valid ISO 8601, but not RFC 3339.
		let result = from_str::<DateTimeRfc3339>("\"20230615T123045Z\"");
		assert!(result.is_err());
	}

	proptest! {
		#[test]
		fn prop_serialize_deserialize_roundtrip(
			year in 2000i32..2050,
			month in 1u32..=12,
			// Avoid month end edge cases.
			day in 1u32..28,
			hour in 0u32..24,
			minute in 0u32..60,
			second in 0u32..60,
			// ±12 hours in seconds.
			offset in -43200i32..43200
		) {
			// Create a fixed offset in seconds (rounded to minute).
			let offset = offset - (offset % 60);
			let tz = FixedOffset::east_opt(offset).unwrap();

			// Create date time.
			let dt = tz.with_ymd_and_hms(year, month, day, hour, minute, second).unwrap();
			let original = DateTimeRfc3339::new(dt);

			// Serialize, and then deserialize.
			let serialized = to_string(&original).unwrap();
			let deserialized: DateTimeRfc3339 = from_str(&serialized).unwrap();

			prop_assert_eq!(original, deserialized);
		}

		#[test]
		fn prop_serialized_format_is_valid_rfc3339(
			year in 2000i32..2050,
			month in 1u32..=12,
			day in 1u32..28,
			hour in 0u32..24,
			minute in 0u32..60,
			second in 0u32..60,
			offset in -43200i32..43200
		) {
			// Create a fixed offset in seconds (rounded to minute).
			let offset = offset - (offset % 60);
			let tz = FixedOffset::east_opt(offset).unwrap();

			// Create date time.
			let dt = tz.with_ymd_and_hms(year, month, day, hour, minute, second).unwrap();
			let wrapper = DateTimeRfc3339::new(dt);

			// Serialize.
			let serialized = to_string(&wrapper).unwrap();

			// Remove the quotes from the JSON string.
			let date_str = &serialized[1..serialized.len()-1];

			// Validate the RFC 3339 format with a regex.
			let rfc3339_regex = regex::Regex::new(
				r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$"
			).unwrap();

			prop_assert!(rfc3339_regex.is_match(date_str));

			// Parse it back as a DateTime.
			let parsed = DateTime::parse_from_rfc3339(date_str).unwrap();

			// Original and parsed should be the same moment in time.
			prop_assert_eq!(dt.timestamp_nanos_opt(), parsed.timestamp_nanos_opt());
		}
	}
}
