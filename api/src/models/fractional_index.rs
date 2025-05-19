use std::cmp::Ordering;
use std::iter::repeat_n;

use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::Type;
use thiserror::Error;

/// A fractional index for ordering content blocks.
///
/// The index is stored as a base-94 string, where each character represents
/// a digit in the range [33, 126] (the set of visible ASCII characters),
/// which enables generation of new index between any two existing indices
/// by averaging their values together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Type)]
#[sqlx(transparent)]
pub struct FractionalIndex(String);

impl FractionalIndex {
	/// The minimum character value used in base-94 encoding.
	const MIN_CHAR: u8 = 33; // Exclamation mark (!)

	/// The maximum character value used in base-94 encoding.
	const MAX_CHAR: u8 = 126; // Tilde (~)

	/// The number of possible values per digit.
	const BASE: u32 = 94;

	/// Creates a new fractional index from a string.
	///
	/// # Arguments
	///
	/// * `index` - The string representation of the index.
	///
	/// # Errors
	///
	/// Returns an error if the string contains characters outside the valid range.
	pub fn new(index: String) -> Result<Self, FractionalIndexError> {
		for c in index.chars() {
			if !(Self::MIN_CHAR as char..=Self::MAX_CHAR as char).contains(&c) {
				return Err(FractionalIndexError::InvalidCharacter(c));
			}
		}

		Ok(Self(index))
	}

	/// Creates a new fractional index at the start of the sequence.
	pub fn start() -> Self {
		Self(String::from("!"))
	}

	/// Creates a new fractional index at the end of the sequence.
	pub fn end() -> Self {
		Self(String::from("~"))
	}

	/// Generates a new index between two existing indices.
	///
	/// # Arguments
	///
	/// * `before` - The index that should come before the new index.
	/// * `after` - The index that should come after the new index.
	///
	/// # Errors
	///
	/// Returns an error if the indices are identical or if the new index would
	/// require more precision than we can represent.
	pub fn between(before: &Self, after: &Self) -> Result<Self, FractionalIndexError> {
		if before == after {
			return Err(FractionalIndexError::IdenticalIndices);
		}

		let mut result = String::new();
		let mut carry = 0;

		// Pad the shorter string with minimum value characters.
		// This makes the strings lexicographically comparable.
		let max_len = before.0.len().max(after.0.len());

		let before_padded = {
			let mut s = before.0.clone();
			s.extend(repeat_n('!', max_len - before.0.len()));
			s
		};

		let after_padded = {
			let mut s = after.0.clone();
			s.extend(repeat_n('!', max_len - after.0.len()));
			s
		};

		for (b, a) in before_padded.chars().zip(after_padded.chars()) {
			// Convert the characters to their numeric values: [33, 126] ↦ [0, 93].
			let b_val = b as u32 - Self::MIN_CHAR as u32;
			let a_val = a as u32 - Self::MIN_CHAR as u32;

			// Calculate the sum of the two values and the carry.
			let sum = b_val + a_val + carry;
			let digit = sum / 2;
			carry = (sum % 2) * Self::BASE;

			// Convert the digit back to a character: [0, 93] ↦ [33, 126].
			let c = (digit as u8 + Self::MIN_CHAR) as char;
			result.push(c);
		}

		// If there's a carry, add an additional digit.
		if carry > 0 {
			// Convert the carry to a character: [0, 93] ↦ [33, 126].
			let c = ((carry / 2) as u8 + Self::MIN_CHAR) as char;
			result.push(c);
		}

		Ok(Self(result))
	}

	/// Returns the string representation of the index.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl PartialOrd for FractionalIndex {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for FractionalIndex {
	fn cmp(&self, other: &Self) -> Ordering {
		// Pad the shorter string with minimum value characters.
		// This makes the strings lexicographically comparable.
		let max_len = self.0.len().max(other.0.len());

		let self_padded = {
			let mut s = self.0.clone();
			s.extend(repeat_n('!', max_len - self.0.len()));
			s
		};

		let other_padded = {
			let mut s = other.0.clone();
			s.extend(repeat_n('!', max_len - other.0.len()));
			s
		};

		self_padded.cmp(&other_padded)
	}
}

#[derive(Debug, Error)]
pub enum FractionalIndexError {
	#[error("Invalid character in index: {0}")]
	InvalidCharacter(char),

	#[error("Cannot generate index between identical indices")]
	IdenticalIndices,
}

#[cfg(test)]
mod tests {
	use proptest::prelude::*;
	use proptest::proptest;

	use super::*;

	/// Generate a string that could be a valid index.
	fn valid_index() -> impl Strategy<Value = String> {
		prop::collection::vec(
			prop::char::range(
				FractionalIndex::MIN_CHAR as char,
				FractionalIndex::MAX_CHAR as char,
			),
			1..10,
		)
		.prop_map(|chars| chars.into_iter().collect())
	}

	#[test]
	fn test_between() {
		let start = FractionalIndex::start();
		let end = FractionalIndex::end();

		// Generate a new index between start and end.
		let middle = FractionalIndex::between(&start, &end).unwrap();
		assert!(start < middle);
		assert!(middle < end);

		// Generate another index between start and middle.
		let quarter = FractionalIndex::between(&start, &middle).unwrap();
		assert!(start < quarter);
		assert!(quarter < middle);

		// Generate another index between middle and end.
		let three_quarters = FractionalIndex::between(&middle, &end).unwrap();
		assert!(middle < three_quarters);
		assert!(three_quarters < end);
	}

	#[test]
	fn test_ordering() {
		let indices = [
			FractionalIndex::start(),
			FractionalIndex::between(&FractionalIndex::start(), &FractionalIndex::end()).unwrap(),
			FractionalIndex::end(),
		];

		for i in 0..indices.len() {
			for j in 0..indices.len() {
				match i.cmp(&j) {
					Ordering::Less => assert!(indices[i] < indices[j]),
					Ordering::Greater => assert!(indices[i] > indices[j]),
					Ordering::Equal => assert!(indices[i] == indices[j]),
				}
			}
		}
	}

	#[test]
	fn test_identical_indices() {
		let index = FractionalIndex::start();
		assert!(FractionalIndex::between(&index, &index).is_err());
	}

	#[test]
	fn test_invalid_characters() {
		// Test non-printable ASCII.
		assert!(FractionalIndex::new("\x00".to_string()).is_err());
		assert!(FractionalIndex::new("\x1F".to_string()).is_err());

		// Test space character.
		assert!(FractionalIndex::new(" ".to_string()).is_err());

		// Test characters outside our range.
		assert!(FractionalIndex::new("\x7F".to_string()).is_err());
		assert!(FractionalIndex::new("é".to_string()).is_err());
	}

	proptest! {
		#[test]
		fn test_ordering_property(a in valid_index(), b in valid_index(), c in valid_index()) {
			// Filter out invalid indices.
			let a = FractionalIndex::new(a).ok();
			let b = FractionalIndex::new(b).ok();
			let c = FractionalIndex::new(c).ok();

			if let (Some(a), Some(b), Some(c)) = (a, b, c) {
				if a < b && b < c {
					let between = FractionalIndex::between(&a, &c).unwrap();
					prop_assert!(a < between);
					prop_assert!(between < c);

					// Sanity check: Are we testing something?
					// Run with `cargo test -- --nocapture` to see the output.
					println!("In test_ordering_property: {} {} {}", a.as_str(), b.as_str(), c.as_str());
				}
			}
		}

		#[test]
		fn test_monotonicity(a in valid_index(), b in valid_index()) {
			// Filter out invalid indices.
			let a = FractionalIndex::new(a).ok();
			let b = FractionalIndex::new(b).ok();

			if let (Some(a), Some(b)) = (a, b) {
				if a < b {
					let between = FractionalIndex::between(&a, &b).unwrap();
					prop_assert!(a < between);
					prop_assert!(between < b);

					// Sanity check: Are we testing something?
					// Run with `cargo test -- --nocapture` to see the output.
					println!("In test_monotonicity: {} {} {}", a.as_str(), b.as_str(), between.as_str());
				}
			}
		}

		#[test]
		fn test_uniqueness(a in valid_index(), b in valid_index(), c in valid_index(), d in valid_index()) {
			// Filter out invalid indices.
			let a = FractionalIndex::new(a).ok();
			let b = FractionalIndex::new(b).ok();
			let c = FractionalIndex::new(c).ok();
			let d = FractionalIndex::new(d).ok();

			if let (Some(ref a), Some(ref b), Some(ref c), Some(ref d)) = (a, b, c, d) {
				if a < b && c < d && (a, b) != (c, d) {
					let between1 = FractionalIndex::between(a, b).unwrap();
					let between2 = FractionalIndex::between(c, d).unwrap();
					prop_assert_ne!(between1, between2);

					// Sanity check: Are we testing something?
					// Run with `cargo test -- --nocapture` to see the output.
					println!("In test_uniqueness: {} {} {} {}", a.as_str(), b.as_str(), c.as_str(), d.as_str());
				}
			}
		}

		#[test]
		fn test_round_trip(a in valid_index(), b in valid_index()) {
			// Filter out invalid indices.
			let a = FractionalIndex::new(a.clone()).ok();
			let b = FractionalIndex::new(b.clone()).ok();

			if let (Some(a), Some(b)) = (a, b) {
				// Test that string representation preserves ordering.
				prop_assert_eq!(a < b, a.as_str() < b.as_str());

				// Test that we can recreate the same index from its string representation.
				let a_recreated = FractionalIndex::new(a.as_str().to_string()).unwrap();
				let b_recreated = FractionalIndex::new(b.as_str().to_string()).unwrap();
				prop_assert_eq!(&a, &a_recreated);
				prop_assert_eq!(&b, &b_recreated);

				// Sanity check: Are we testing something?
				// Run with `cargo test -- --nocapture` to see the output.
				println!("In test_round_trip: {} {} {}", a.as_str(), b.as_str(), a_recreated.as_str());
			}
		}
	}
}
