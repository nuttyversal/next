use proptest::prelude::Strategy;
use sqlx::types::Uuid;
use std::cmp::Ordering;
use thiserror::Error;

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
		encode_base_58(last_41_bits)
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

/// Either [NuttyId] or [DissociatedNuttyId].
///
/// Used in signatures for functions and methods that can
/// accept any Nutty ID, even the ones that are dissociated.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum AnyNuttyId {
	/// A Nutty ID associated with a UUID.
	Associated(NuttyId),

	/// A Nutty ID without a UUID — dissociated. 💀
	Dissociated(DissociatedNuttyId),
}

impl AnyNuttyId {
	/// Create a new Nutty ID from a string slice.
	pub fn new(nid: &str) -> Result<Self, NuttyIdError> {
		match DissociatedNuttyId::new(nid) {
			Ok(dissociated) => Ok(AnyNuttyId::Dissociated(dissociated)),
			Err(e) => Err(e),
		}
	}

	/// Get the Nutty ID.
	pub fn nid(&self) -> String {
		match self {
			AnyNuttyId::Associated(nutty_id) => nutty_id.nid(),
			AnyNuttyId::Dissociated(nutty_id) => nutty_id.nid(),
		}
	}
}

impl From<NuttyId> for AnyNuttyId {
	fn from(nutty_id: NuttyId) -> Self {
		AnyNuttyId::Associated(nutty_id)
	}
}

impl From<DissociatedNuttyId> for AnyNuttyId {
	fn from(nutty_id: DissociatedNuttyId) -> Self {
		AnyNuttyId::Dissociated(nutty_id)
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
fn extract_last_41_bits(uuid: &Uuid) -> u64 {
	let bytes = uuid.as_bytes();

	// Extract the last bit (41) from the 6th byte from the end,
	// plus all the bits (1..40) from the last 5 bytes.
	(((bytes[10] & 0x01) as u64) << 40)
		| ((bytes[11] as u64) << 32)
		| ((bytes[12] as u64) << 24)
		| ((bytes[13] as u64) << 16)
		| ((bytes[14] as u64) << 8)
		| (bytes[15] as u64)
}

/// Encode a u64 value to a base-58 string in big-endian order.
fn encode_base_58(value: u64) -> String {
	const FIXED_LENGTH: usize = 7;
	const BASE: u64 = 58;

	// Special case for 0.
	if value == 0 {
		return "1".repeat(FIXED_LENGTH);
	}

	// Convert to base-58 in little-endian order.
	let mut digits = Vec::with_capacity(FIXED_LENGTH);
	let mut remaining = value;

	while remaining > 0 {
		digits.push((remaining % BASE) as usize);
		remaining /= BASE;
	}

	let padding_length = FIXED_LENGTH.saturating_sub(digits.len());
	let mut result = String::with_capacity(FIXED_LENGTH);

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
	use super::*;
	use proptest::prelude::*;

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
			if value < 58u64.pow(6) {
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
		let max_value = (1u64 << 41) - 1;
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
}
