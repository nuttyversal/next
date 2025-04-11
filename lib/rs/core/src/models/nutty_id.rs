use sqlx::types::Uuid;

/// A Nutty ID is a short identifier for a Nuttyverse resource.
///
/// It can be derived from a UUID, but isn't a UUID itself.
/// It is a base-58 encoded string of 7 characters derived
/// from the last 41 bits of a UUID. Useful for generating
/// permanent URL links like https://nuttyver.se/abcdefg.
///
/// Why base-58? Because '0', 'O', 'I', and 'l' are ambiguous.
/// Why 41 bits? Because 2⁴² > 58⁷ > 2⁴¹.
pub struct NuttyId(String);

impl NuttyId {
	pub fn new(id: Uuid) -> Self {
		let last_41_bits = extract_last_41_bits(id);
		let encoded = encode_base58(last_41_bits);

		Self(encoded)
	}
}

/// Extract the last 41 bits of an UUIDv7.
fn extract_last_41_bits(uuid: Uuid) -> u64 {
	let bytes = uuid.as_bytes();
	let mut value = 0u64;

	// Take the last 5 bytes (40 bits) …
	(bytes.len() - 5..bytes.len()).for_each(|i| {
		value = (value << 8) | bytes[i] as u64;
	});

	// … + 1 bit from the 6ᵗʰ byte from the end.
	value = (value << 1) | ((bytes[bytes.len() - 6] >> 7) & 1) as u64;

	// That's 41 bits.
	value
}

/// Encode a u64 value to a base-58 string (big-endian).
fn encode_base58(value: u64) -> String {
	const FIXED_LENGTH: usize = 7;
	const BASE: u64 = 58;

	/// Base-58 alphabet — the ₿ encoding.
	/// Satoshi Nakamoto came up with it.
	const BASE58_ALPHABET: &[char] = &[
		'1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
		'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
		'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
		'w', 'x', 'y', 'z',
	];

	let mut digits = Vec::with_capacity(FIXED_LENGTH);
	let mut remaining = value;

	// Special case for 0.
	// Buy a lottery ticket if this happens.
	if value == 0 {
		return "1".repeat(FIXED_LENGTH);
	}

	// Convert to base-58.
	while remaining > 0 {
		digits.push((remaining % BASE) as usize);
		remaining /= BASE;
	}

	// Reverse to get big-endian order.
	digits.reverse();

	let padding_length = FIXED_LENGTH.saturating_sub(digits.len());
	let mut result = String::with_capacity(FIXED_LENGTH);

	// Left pad with ones — because 0 is ambiguous.
	// Notice how I didn't need a library for this?
	// https://en.wikipedia.org/wiki/Npm_left-pad_incident
	for _ in 0..padding_length {
		result.push(BASE58_ALPHABET[0]);
	}

	// Encode the digits.
	for digit in digits {
		result.push(BASE58_ALPHABET[digit]);
	}

	result
}
