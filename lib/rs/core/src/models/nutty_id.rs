use sqlx::types::Uuid;

/// Base-58 alphabet — the ₿ encoding.
/// Satoshi Nakamoto came up with it.
const BASE58_ALPHABET: &[char] = &[
	'1', '2', '3', '4', '5', '6', '7', '8', '9', // …
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', // …
	'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', // …
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', // …
	'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// A Nutty ID is a short identifier for a Nuttyverse resource.
///
/// It can be derived from a UUID, but isn't a UUID itself.
/// It is a base-58 encoded string of 7 characters derived
/// from the last 41 bits of a UUID. Useful for generating
/// permanent URL links like https://nuttyver.se/abcdefg.
///
/// Why 41 bits? Because 2^42 > 58^7 > 2^41.
/// Why base-58? Because '0', 'O', 'I', and 'l' are ambiguous.
/// Why do this at all? Because it's a fun idea.
#[derive(Debug)]
pub struct NuttyId(String);

impl NuttyId {
	/// Create a new Nutty ID from a string without validation.
	/// [WARN] This should only be used for trusted inputs.
	pub fn new(id: String) -> Self {
		Self(id)
	}

	/// Derive a Nutty ID from a UUID.
	pub fn from_uuid(id: Uuid) -> Self {
		let last_41_bits = extract_last_41_bits(id);
		let nutty_id = encode_base58(last_41_bits);

		Self(nutty_id)
	}

	/// Check if a string is a valid Nutty ID.
	pub fn is_valid(id: &str) -> bool {
		// Assert: It must be exactly 7 characters.
		if id.len() != 7 {
			return false;
		}

		// Assert: It must contain only valid characters.
		id.chars().all(|c| BASE58_ALPHABET.contains(&c))
	}

	/// Get the string representation of the Nutty ID.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl TryFrom<String> for NuttyId {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if Self::is_valid(&value) {
			Ok(Self(value))
		} else {
			Err(format!("Invalid Nutty ID format: '{}'", value))
		}
	}
}

/// Extract the last 41 bits of a UUID.
fn extract_last_41_bits(uuid: Uuid) -> u64 {
	let bytes = uuid.as_bytes();

	// Take the last bit (LSB) from the 6th byte from the end.
	let mut value = (bytes[bytes.len() - 6] & 0x01) as u64;

	// Then, take the last 5 bytes (40 bits) from MSB to LSB.
	(bytes.len() - 5..bytes.len()).for_each(|i| {
		value = (value << 8) | bytes[i] as u64;
	});

	value
}

/// Encode a u64 value to a base-58 string (big-endian).
fn encode_base58(value: u64) -> String {
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
		result.push(BASE58_ALPHABET[0]);
	}

	// Encode the digits in big-endian order.
	for digit in digits.iter().rev() {
		result.push(BASE58_ALPHABET[*digit]);
	}

	result
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

		let id = NuttyId::from_uuid(uuid);
		assert_eq!(id.as_str(), "qfWLRgy");
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
			let id = NuttyId::from_uuid(uuid.0);
			assert_eq!(id.as_str().len(), 7);
		}

		#[test]
		fn test_nutty_id_chars(uuid in any::<TestUuid>()) {
			let id = NuttyId::from_uuid(uuid.0);
			assert!(id.as_str().chars().all(|c| BASE58_ALPHABET.contains(&c)));
		}

		#[test]
		fn test_nutty_id_padding(uuid in any::<TestUuid>()) {
			let id = NuttyId::from_uuid(uuid.0);
			let value = extract_last_41_bits(uuid.0);

			// If the value is small enough,
			// then it should be padded.
			if value < 58u64.pow(6) {
				let leading_ones = id.as_str().chars().take_while(|&c| c == '1').count();
				assert!(leading_ones > 0);
			}
		}

		#[test]
		fn test_nutty_id_ordering(uuid1 in any::<TestUuid>(), uuid2 in any::<TestUuid>()) {
			let id1 = NuttyId::from_uuid(uuid1.0);
			let id2 = NuttyId::from_uuid(uuid2.0);
			let bits1 = extract_last_41_bits(uuid1.0);
			let bits2 = extract_last_41_bits(uuid2.0);
			assert_eq!(id1.as_str() < id2.as_str(), bits1 < bits2);
		}

		#[test]
		fn test_nutty_id_uniqueness(uuid1 in any::<TestUuid>(), uuid2 in any::<TestUuid>()) {
			prop_assume!(uuid1.0 != uuid2.0);
			let id1 = NuttyId::from_uuid(uuid1.0);
			let id2 = NuttyId::from_uuid(uuid2.0);
			assert_ne!(id1.as_str(), id2.as_str());
		}
	}

	#[test]
	fn test_zero_value() {
		// Create a UUID with all bits set to 0.
		let uuid = Uuid::from_bytes([0; 16]);
		let id = NuttyId::from_uuid(uuid);
		assert_eq!(id.as_str(), "1111111");
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
		let id = NuttyId::from_uuid(uuid);

		// Calculate the expected maximum value.
		let max_value = (1u64 << 41) - 1;
		let extracted = extract_last_41_bits(uuid);

		// The extracted value should match the maximum value.
		assert_eq!(extracted, max_value);

		// The ID should be exactly 7 characters.
		assert_eq!(id.as_str().len(), 7);

		// The value should be stable.
		assert_eq!(id.as_str(), "zmM9z4E");
	}

	#[test]
	fn test_is_valid() {
		// Valid cases.
		assert!(NuttyId::is_valid("1111111"));
		assert!(NuttyId::is_valid("abcdefg"));
		assert!(NuttyId::is_valid("ABCDEFG"));
		assert!(NuttyId::is_valid("1234567"));
		assert!(NuttyId::is_valid("zmM9z4E"));

		// Invalid cases.
		assert!(!NuttyId::is_valid("")); // Too short.
		assert!(!NuttyId::is_valid("123456")); // Too short.
		assert!(!NuttyId::is_valid("12345678")); // Too long.
		assert!(!NuttyId::is_valid("abcdef0")); // Contains '0'.
		assert!(!NuttyId::is_valid("abcdefO")); // Contains 'O'.
		assert!(!NuttyId::is_valid("abcdefI")); // Contains 'I'.
		assert!(!NuttyId::is_valid("abcdefl")); // Contains 'l'.
		assert!(!NuttyId::is_valid("abcdef!")); // Contains invalid character.
	}

	#[test]
	fn test_try_from() {
		// Valid cases.
		assert!(NuttyId::try_from("1111111".to_string()).is_ok());
		assert!(NuttyId::try_from("abcdefg".to_string()).is_ok());
		assert!(NuttyId::try_from("ABCDEFG".to_string()).is_ok());
		assert!(NuttyId::try_from("1234567".to_string()).is_ok());
		assert!(NuttyId::try_from("zmM9z4E".to_string()).is_ok());

		// Invalid cases.
		assert!(NuttyId::try_from("".to_string()).is_err());
		assert!(NuttyId::try_from("123456".to_string()).is_err());
		assert!(NuttyId::try_from("12345678".to_string()).is_err());
		assert!(NuttyId::try_from("abcdef0".to_string()).is_err());
		assert!(NuttyId::try_from("abcdefO".to_string()).is_err());
		assert!(NuttyId::try_from("abcdefI".to_string()).is_err());
		assert!(NuttyId::try_from("abcdefl".to_string()).is_err());
		assert!(NuttyId::try_from("abcdef!".to_string()).is_err());

		// Test error messages.
		let err = NuttyId::try_from("".to_string()).unwrap_err();
		assert!(err.contains("Invalid Nutty ID format"));
		assert!(err.contains("''"));

		let err = NuttyId::try_from("abcdef0".to_string()).unwrap_err();
		assert!(err.contains("Invalid Nutty ID format"));
		assert!(err.contains("'abcdef0'"));
	}

	proptest! {
		#[test]
		fn test_try_from_property(id in "[1-9A-HJ-NP-Za-km-z]{7}") {
			assert!(NuttyId::try_from(id).is_ok());
		}

		#[test]
		fn test_try_from_invalid_property(id in "[^1-9A-HJ-NP-Za-km-z]+") {
			assert!(NuttyId::try_from(id).is_err());
		}
	}
}
