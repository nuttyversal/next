use std::cmp::Ordering;

use proptest::prelude::Strategy;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

/// A Nutty ID is a newtype wrapper around a UUID.
///
/// It can be used to derive a short base-58 encoded string
/// of 7 characters derived from the last 41 bits of a UUID.
/// For generating permalinks: https://nuttyver.se/abcdefg.
///
/// Why 41 bits? Because 2^42 > 58^7 > 2^41.
/// Why base-58? Because '0', 'O', 'I', and 'l' are ambiguous.
/// Why do this at all? Because it's a fun idea.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct NuttyId {
	uuid: Uuid,
}

impl NuttyId {
	/// Derive a Nutty ID from a UUID.
	pub fn new(uuid: Uuid) -> Self {
		Self { uuid }
	}

	/// Create a new Nutty ID from a UUIDv7.
	pub fn now() -> Self {
		let uuid = Uuid::now_v7();
		Self::new(uuid)
	}

	/// Get the UUID.
	pub fn uuid(&self) -> &Uuid {
		&self.uuid
	}

	/// Get the Nutty ID.
	pub fn nid(&self) -> String {
		let last_41_bits = extract_last_41_bits(&self.uuid);
		encode_base_58(last_41_bits, 7)
	}

	/// Get the Unix timestamp (in milliseconds) encoded in the UUIDv7.
	pub fn timestamp(&self) -> u64 {
		let bytes = self.uuid.as_bytes();

		// Extract the first 48 bits (6 bytes).
		((bytes[0] as u64) << 40)
			| ((bytes[1] as u64) << 32)
			| ((bytes[2] as u64) << 24)
			| ((bytes[3] as u64) << 16)
			| ((bytes[4] as u64) << 8)
			| (bytes[5] as u64)
	}

	/// Detach the NID from the UUID.
	pub fn dissociate(&self) -> DissociatedNuttyId {
		DissociatedNuttyId::new(&self.nid()).expect("the impossible")
	}
}

impl Serialize for NuttyId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let uuid = self.uuid.as_u128();
		let uuid = encode_base_58(uuid, 22);
		let nid = self.nid();
		serializer.serialize_str(&(uuid + ":" + &nid))
	}
}

impl<'de> serde::Deserialize<'de> for NuttyId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;

		// Split the string on the colon.
		let parts: Vec<&str> = s.split(':').collect();

		if parts.len() != 2 {
			return Err(serde::de::Error::custom(
				"Invalid format: expected <BASE58-UUID>:<NID>",
			));
		}

		// Decode the UUID from base-58.
		let uuid_u128 = decode_base_58(parts[0])
			.map_err(|e| serde::de::Error::custom(format!("Invalid UUID: {e}")))?;

		// Convert the u128 to a UUID.
		let uuid = Uuid::from_u128(uuid_u128);

		// Create the Nutty ID.
		let nutty_id = NuttyId::new(uuid);

		// Verify the NID as a checksum.
		if nutty_id.nid() != parts[1] {
			return Err(serde::de::Error::custom(format!(
				"NID mismatch: expected {}, got {}",
				nutty_id.nid(),
				parts[1]
			)));
		}

		Ok(nutty_id)
	}
}

/// This poor Nutty ID is traumatized.
///
/// Lost in the wild, with no UUID to call its own,
/// it wanders through memory, a nomad without a home.
///
/// Any Nutty ID can be derived from a UUID,
/// but the path backward is darkened, you see.
///
/// To find its ancestral UUID, you must decree
/// a solemn query to the content block tree.
///
/// tl;dr: it's surjective, but not injective.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DissociatedNuttyId {
	nid: [u8; 7],
}

impl DissociatedNuttyId {
	/// Create a new Nutty ID from a string slice.
	pub fn new(nid: &str) -> Result<Self, NuttyIdError> {
		if !is_valid_nutty_id(nid) {
			return Err(NuttyIdError::ValidationError(nid.to_string()));
		}

		let nid_bytes: [u8; 7] = match nid.as_bytes().try_into() {
			Ok(bytes) => bytes,
			Err(_) => return Err(NuttyIdError::ConversionError(nid.to_string())),
		};

		Ok(Self { nid: nid_bytes })
	}

	/// Get the Nutty ID.
	pub fn nid(&self) -> String {
		std::str::from_utf8(&self.nid)
			// Since we validated the bytes on creation,
			// this shouldn't fail. Smart constructors. 😎
			.expect("Nutty ID contains invalid UTF-8")
			.to_string()
	}
}

impl From<NuttyId> for DissociatedNuttyId {
	fn from(nutty_id: NuttyId) -> Self {
		nutty_id.dissociate()
	}
}

impl From<&NuttyId> for DissociatedNuttyId {
	fn from(nutty_id: &NuttyId) -> Self {
		nutty_id.dissociate()
	}
}

/// Base-58 alphabet — the ₿ encoding.
/// Satoshi Nakamoto came up with it.
const BASE_58_ALPHABET: &[char] = &[
	'1', '2', '3', '4', '5', '6', '7', '8', '9', // …
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', // …
	'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', // …
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', // …
	'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Check if a string is a valid Nutty ID.
fn is_valid_nutty_id(id: &str) -> bool {
	// Assert: It must be exactly 7 characters.
	if id.len() != 7 {
		return false;
	}

	// Assert: It must contain only valid characters.
	if !id.chars().all(|c| BASE_58_ALPHABET.contains(&c)) {
		return false;
	}

	// Assert: It must be derived from the last 41 bits of a UUID.
	let max_41_bit_value_base_58: &str = "zmM9z4E";
	id <= max_41_bit_value_base_58
}

/// Extract the last 41 bits of a UUID.
fn extract_last_41_bits(uuid: &Uuid) -> u128 {
	let bytes = uuid.as_bytes();

	// Extract the last bit (41) from the 6th byte from the end,
	// plus all the bits (1..40) from the last 5 bytes.
	(((bytes[10] & 0x01) as u128) << 40)
		| ((bytes[11] as u128) << 32)
		| ((bytes[12] as u128) << 24)
		| ((bytes[13] as u128) << 16)
		| ((bytes[14] as u128) << 8)
		| (bytes[15] as u128)
}

/// Encode a u128 value to a base-58 string in big-endian order.
///
/// If the resulting string is less than `pad_width`, then the
/// remaining space will be padded with "1"s.
fn encode_base_58(value: u128, pad_width: usize) -> String {
	const BASE: u128 = 58;

	// Special case for 0.
	if value == 0 {
		return "1".repeat(pad_width);
	}

	// Convert to base-58 in little-endian order.
	let mut digits = Vec::with_capacity(pad_width);
	let mut remaining = value;

	while remaining > 0 {
		digits.push((remaining % BASE) as usize);
		remaining /= BASE;
	}

	let padding_length = pad_width.saturating_sub(digits.len());
	let mut result = String::with_capacity(pad_width);

	// Left pad with ones. Look, no package needed. 👀
	// https://en.wikipedia.org/wiki/Npm_left-pad_incident
	for _ in 0..padding_length {
		result.push(BASE_58_ALPHABET[0]);
	}

	// Encode the digits in big-endian order.
	for digit in digits.iter().rev() {
		result.push(BASE_58_ALPHABET[*digit]);
	}

	result
}

/// Decode a base-58 string to a u128 value.
fn decode_base_58(input: &str) -> Result<u128, DecodeError> {
	const BASE: u128 = 58;

	if input.is_empty() {
		return Err(DecodeError::InvalidInput("<empty>".to_string()));
	}

	let mut result: u128 = 0;

	for c in input.chars() {
		let digit = match BASE_58_ALPHABET.iter().position(|&x| x == c) {
			Some(pos) => pos as u128,
			None => return Err(DecodeError::InvalidCharacter(c)),
		};

		result = result * BASE + digit;
	}

	Ok(result)
}

#[derive(Debug, Error)]
pub enum DecodeError {
	#[error("Invalid character '{0}' in base-58 string")]
	InvalidCharacter(char),

	#[error("Invalid input string: {0}")]
	InvalidInput(String),
}

#[derive(Debug, Error)]
pub enum NuttyIdError {
	#[error("Failed to convert '{0}' to 7 bytes")]
	ConversionError(String),

	#[error("Invalid Nutty ID format: '{0}'")]
	ValidationError(String),
}

// A proptest strategy for generating valid Nutty IDs.
pub fn valid_nutty_id() -> impl Strategy<Value = String> {
	// It must be base-58 encoded and 7 characters long.
	let regex_strategy = "[1-9A-HJ-NP-Za-km-z]{7}";

	// It must be derived from the last 41 bits of a UUID.
	regex_strategy.prop_filter("Nutty ID must be <= zmM9z4E", |s| {
		// Compare lexicographically with the maximum value.
		s.cmp(&"zmM9z4E".to_string()) != Ordering::Greater
	})
}

#[cfg(test)]
mod tests {
	use proptest::prelude::*;

	use super::*;

	#[test]
	fn test_hand_calculated_nutty_id() {
		let uuid = Uuid::from_bytes([
			0x01, 0x96, 0x23, 0x29, // -
			0xad, 0x5a, // -
			0x7f, 0xfd, // -
			0x83, 0x13, // -
			0x7f, 0xaf, 0x55, 0xd2, 0x91, 0xf6,
		]);

		// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
		// ┃  HOMEWORK ASSIGNMENT: Calculate the Nutty ID.   ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • UUIDv7: 01962329-ad5a-7ffd-8313-7faf55d291f6  ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Get the last six bytes in big-endian order.   ┃
		// ┃ • We want to grab the last 41 bits.             ┃
		// ┃ • 0x7faf55d291f6 & 0x01ffffffffff               ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Show the calculation in binary:               ┃
		// ┃ • 0x7f & 0x01 = 0b01111111 & 0b00000001         ┃
		// ┃ • 0xaf & 0xff = 0b10101111 & 0b11111111         ┃
		// ┃ • 0x55 & 0xff = 0b01010101 & 0b11111111         ┃
		// ┃ • 0xd2 & 0xff = 0b11010010 & 0b11111111         ┃
		// ┃ • 0x91 & 0xff = 0b10010001 & 0b11111111         ┃
		// ┃ • 0xf6 & 0xff = 0b11110110 & 0b11111111         ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Binary result (base-2):                       ┃
		// ┃ • 0b11010111101010101110100101001000111110110   ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Cheat & use a binary-to-decimal calculator:   ┃
		// ┃ • 1852570767862                                 ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Convert to base-58:                           ┃
		// ┃ • 1852570767862 mod 58 = 56 = y                 ┃
		// ┃ • 31940875307 mod 58 = 39 = g                   ┃
		// ┃ • 550704746 mod 58 = 24 = R                     ┃
		// ┃ • 9494909 mod 58 = 19 = L                       ┃
		// ┃ • 163705 mod 58 = 29 = W                        ┃
		// ┃ • 2822 mod 58 = 38 = f                          ┃
		// ┃ • 48 mod 58 = 48 = q                            ┃
		// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
		// ┃ • Nutty ID: qfWLRgy                             ┃
		// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

		let nutty_id = NuttyId::new(uuid);
		assert_eq!(nutty_id.nid(), "qfWLRgy");
	}

	#[test]
	fn test_deserialize() {
		// Create a known UUID and its associated NID.
		let nutty_id = NuttyId::now();
		let uuid = nutty_id.uuid();
		let correct_nid = nutty_id.nid();

		// Test case 0: Valid UUID and NID.
		let well_formed = format!("{}:{correct_nid}", encode_base_58(uuid.as_u128(), 22));
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{well_formed}\""));
		assert!(result.is_ok());

		// Test case 1: Wrong NID for the UUID.
		let wrong_nid = "1234567";
		let malformed = format!("{uuid}:{wrong_nid}");
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{malformed}\""));
		assert!(result.is_err());

		// Test case 2: Missing colon separator.
		let no_colon = format!("{uuid}{correct_nid}");
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{no_colon}\""));
		assert!(result.is_err());

		// Test case 3: Invalid UUID.
		let invalid_uuid = format!("not-a-uuid:{correct_nid}");
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{invalid_uuid}\""));
		assert!(result.is_err());

		// Test case 4: Invalid NID (wrong length).
		let invalid_nid = format!("{uuid}:12345");
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{invalid_nid}\""));
		assert!(result.is_err());

		// Test case 5: Invalid NID (illegal character).
		let invalid_nid = format!("{uuid}:abcdef0"); // contains '0' which is not in base58
		let result: Result<NuttyId, _> = serde_json::from_str(&format!("\"{invalid_nid}\""));
		assert!(result.is_err());
	}

	/// A newtype wrapper for [Uuid] to implement [Arbitrary].
	#[derive(Debug, Clone)]
	struct TestUuid(Uuid);

	impl Arbitrary for TestUuid {
		type Parameters = ();
		type Strategy = BoxedStrategy<Self>;

		fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
			any::<[u8; 16]>()
				.prop_map(|bytes| TestUuid(Uuid::from_bytes(bytes)))
				.boxed()
		}
	}

	proptest! {
		#[test]
		fn test_nutty_id_length(uuid in any::<TestUuid>()) {
			let nutty_id = NuttyId::new(uuid.0);
			assert_eq!(nutty_id.nid().len(), 7);
		}

		#[test]
		fn test_nutty_id_chars(uuid in any::<TestUuid>()) {
			let nutty_id = NuttyId::new(uuid.0);
			assert!(nutty_id.nid().chars().all(|c| BASE_58_ALPHABET.contains(&c)));
		}

		#[test]
		fn test_nutty_id_padding(uuid in any::<TestUuid>()) {
			let nutty_id = NuttyId::new(uuid.0);
			let value = extract_last_41_bits(&uuid.0);

			// If the value is small enough,
			// then it should be left-padded.
			if value < 58u128.pow(6) {
				let leading_ones = nutty_id.nid().chars().take_while(|&c| c == '1').count();
				assert!(leading_ones > 0);
			}
		}

		#[test]
		fn test_nutty_id_ordering(uuid1 in any::<TestUuid>(), uuid2 in any::<TestUuid>()) {
			let nutty_id1 = NuttyId::new(uuid1.0);
			let nutty_id2 = NuttyId::new(uuid2.0);
			let bits1 = extract_last_41_bits(&uuid1.0);
			let bits2 = extract_last_41_bits(&uuid2.0);
			assert_eq!(nutty_id1.nid() < nutty_id2.nid(), bits1 < bits2);
		}

		#[test]
		fn test_nutty_id_serde_roundtrip(uuid in any::<TestUuid>()) {
			// Create a Nutty ID.
			let original = NuttyId::new(uuid.0);

			// Serialize to JSON.
			let serialized = serde_json::to_string(&original)
				.expect("Failed to serialize NuttyId");

			// Deserialize to Nutty ID.
			let deserialized: NuttyId = serde_json::from_str(&serialized)
				.expect("Failed to deserialize NuttyId");

			// Check equality.
			assert_eq!(original, deserialized);
			assert_eq!(original.uuid(), deserialized.uuid());
			assert_eq!(original.nid(), deserialized.nid());
		}
	}

	#[test]
	fn test_zero_value() {
		// Create a UUID with all bits set to 0.
		let uuid = Uuid::from_bytes([0; 16]);
		let nutty_id = NuttyId::new(uuid);
		assert_eq!(nutty_id.nid(), "1111111");
	}

	#[test]
	fn test_max_value() {
		// Create a UUID with the last 41 bits set to 1.
		let mut bytes = [0; 16];

		// Set the last 5 bytes to all 1s (40 bits).
		(11..16).for_each(|i| {
			bytes[i] = 0xFF;
		});

		// Set the last bit in the 6th byte from the end (41st bit).
		bytes[10] |= 0x01;
		let uuid = Uuid::from_bytes(bytes);
		let nutty_id = NuttyId::new(uuid);

		// Calculate the expected maximum value.
		let max_value = (1u128 << 41) - 1;
		let extracted = extract_last_41_bits(&uuid);

		// The extracted value should match the maximum value.
		assert_eq!(extracted, max_value);

		// The ID should be exactly 7 characters.
		assert_eq!(nutty_id.nid().len(), 7);

		// The value should be stable.
		assert_eq!(nutty_id.nid(), "zmM9z4E");
	}

	#[test]
	fn test_is_valid_nutty_id() {
		// Valid cases.
		assert!(is_valid_nutty_id("1111111"));
		assert!(is_valid_nutty_id("abcdefg"));
		assert!(is_valid_nutty_id("ABCDEFG"));
		assert!(is_valid_nutty_id("1234567"));
		assert!(is_valid_nutty_id("zmM9z4E"));

		// Invalid cases.
		assert!(!is_valid_nutty_id("")); // Too short.
		assert!(!is_valid_nutty_id("123456")); // Too short.
		assert!(!is_valid_nutty_id("12345678")); // Too long.
		assert!(!is_valid_nutty_id("abcdef0")); // Contains '0'.
		assert!(!is_valid_nutty_id("abcdefO")); // Contains 'O'.
		assert!(!is_valid_nutty_id("abcdefI")); // Contains 'I'.
		assert!(!is_valid_nutty_id("abcdefl")); // Contains 'l'.
		assert!(!is_valid_nutty_id("abcdef!")); // Contains invalid character.
		assert!(!is_valid_nutty_id("zzzzzzz")); // Not derived from UUID.
	}

	#[test]
	fn test_timestamp_extraction() {
		// January 1, 2023 00:00:00 UTC = 1672531200000 milliseconds since epoch.
		// This is 0x0186741B0D80 in hexadecimal (48 bits).
		let mut bytes = [0; 16];
		bytes[0] = 0x01;
		bytes[1] = 0x86;
		bytes[2] = 0x74;
		bytes[3] = 0x1B;
		bytes[4] = 0x0D;
		bytes[5] = 0x80;

		// Set version to 7 (UUIDv7).
		bytes[6] = (bytes[6] & 0x0F) | 0x70;

		// Set variant to RFC 4122.
		bytes[8] = (bytes[8] & 0x3F) | 0x80;

		// Fill remaining bytes with some values.
		(9..16).for_each(|i| {
			bytes[i] = i as u8;
		});

		let uuid = Uuid::from_bytes(bytes);
		let nutty_id = NuttyId::new(uuid);

		let expected_timestamp = 0x0186741B0D80;
		assert_eq!(nutty_id.timestamp(), expected_timestamp);
	}

	#[test]
	fn test_now_timestamp_is_current() {
		// Get current timestamp in milliseconds.
		let before_ms = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis() as u64;

		// Create a new NuttyId with the current time.
		let nutty_id = NuttyId::now();

		// Get current timestamp after creating NuttyId.
		let after_ms = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis() as u64;

		// Extract the timestamp from the NuttyId.
		let extracted_ms = nutty_id.timestamp();

		// Verify the timestamp is within range.
		assert!(
			extracted_ms >= before_ms && extracted_ms <= after_ms,
			"Extracted timestamp ({extracted_ms}) should be between {before_ms} and {after_ms}"
		);
	}
}
