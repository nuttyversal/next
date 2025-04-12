use crate::models::NuttyId;
use std::fmt;

/// A NuttyTag represents a wikilink-style tag containing a Nutty ID.
///
/// It can be in the form [[abcdefg]] or [[abcdefg|Display Text]],
/// where "abcdefg" is a valid Nutty ID and "Display Text" is
/// optional text to render in the UI instead of the Nutty ID.
#[derive(Debug)]
pub struct NuttyTag {
	nutty_id: NuttyId,
	display_text: Option<String>,
}

impl NuttyTag {
	/// Create a new `NuttyTag` from a Nutty ID and optional display text.
	pub fn new(nutty_id: NuttyId, display_text: Option<String>) -> Self {
		Self {
			nutty_id,
			display_text,
		}
	}

	/// Parse a tag string like [[abcdefg]] or [[abcdefg|Display Text]].
	pub fn parse(value: &str) -> Result<Self, String> {
		// Check for opening and closing brackets.
		if !value.starts_with("[[") || !value.ends_with("]]") {
			return Err(format!("Invalid NuttyTag format: '{}'", value));
		}

		// Strip the opening and closing brackets.
		let content = &value[2..value.len() - 2];

		// Split by pipe character, if present.
		let parts: Vec<&str> = content.split('|').collect();

		match parts.len() {
			// Format: [[abcdefg]]
			1 => {
				let id_str = parts[0].trim();
				let nutty_id = NuttyId::try_from(id_str.to_string())?;

				Ok(Self {
					nutty_id,
					display_text: None,
				})
			}

			// Format: [[abcdefg|Display Text]]
			2 => {
				let id_str = parts[0].trim();
				let display = parts[1].trim();
				let nutty_id = NuttyId::try_from(id_str.to_string())?;

				Ok(Self {
					nutty_id,
					display_text: Some(display.to_string()),
				})
			}

			// Format: (╯‵Д′)╯彡┻━┻
			_ => Err(format!("Invalid NuttyTag format: '{}'", value)),
		}
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &NuttyId {
		&self.nutty_id
	}

	/// Get the display text, if any.
	pub fn display_text(&self) -> Option<&str> {
		self.display_text.as_deref()
	}
}

impl fmt::Display for NuttyTag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.display_text {
			Some(display) => write!(f, "[[{}|{}]]", self.nutty_id.as_str(), display),
			None => write!(f, "[[{}]]", self.nutty_id.as_str()),
		}
	}
}

impl TryFrom<&str> for NuttyTag {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::parse(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use proptest::prelude::*;

	#[test]
	fn test_parse_simple_tag() {
		let tag = NuttyTag::parse("[[abcdefg]]").unwrap();
		assert_eq!(tag.nutty_id().as_str(), "abcdefg");
		assert_eq!(tag.display_text(), None);
	}

	#[test]
	fn test_parse_tag_with_display_text() {
		let tag = NuttyTag::parse("[[abcdefg|Display Text]]").unwrap();
		assert_eq!(tag.nutty_id().as_str(), "abcdefg");
		assert_eq!(tag.display_text(), Some("Display Text"));
	}

	#[test]
	fn test_parse_invalid_tag() {
		// Missing opening brackets.
		assert!(NuttyTag::parse("abcdefg]]").is_err());

		// Missing closing brackets.
		assert!(NuttyTag::parse("[[abcdefg").is_err());

		// Invalid NuttyId (contains '0').
		assert!(NuttyTag::parse("[[abcdef0]]").is_err());

		// Invalid NuttyId (is 6 characters).
		assert!(NuttyTag::parse("[[abcdef]]").is_err());

		// Too many pipe characters.
		assert!(NuttyTag::parse("[[abcdefg|Display|Text]]").is_err());

		// Empty content.
		assert!(NuttyTag::parse("[[]]").is_err());
	}

	#[test]
	fn test_display_trait() {
		// A simple tag.
		let nutty_id = NuttyId::try_from("abcdefg".to_string()).unwrap();
		let tag = NuttyTag::new(nutty_id, None);
		assert_eq!(format!("{}", tag), "[[abcdefg]]");

		// A tag with display text.
		let nutty_id = NuttyId::try_from("abcdefg".to_string()).unwrap();
		let tag = NuttyTag::new(nutty_id, Some("Display Text".to_string()));
		assert_eq!(format!("{}", tag), "[[abcdefg|Display Text]]");
	}

	proptest! {
		 #[test]
		 fn test_parse_valid_tag_property(id in "[1-9A-HJ-NP-Za-km-z]{7}") {
			  let tag_str = format!("[[{}]]", id);
			  let result = NuttyTag::parse(&tag_str);
			  assert!(result.is_ok());

			  let parsed = result.unwrap();
			  assert_eq!(parsed.nutty_id().as_str(), id);
			  assert_eq!(parsed.display_text(), None);
		 }

		 #[test]
		 fn test_parse_valid_tag_with_display_property(
			  id in "[1-9A-HJ-NP-Za-km-z]{7}",
			  display in "[^|]{1,100}"
		 ) {
			  let tag_str = format!("[[{}|{}]]", id, display);
			  let result = NuttyTag::parse(&tag_str);
			  assert!(result.is_ok());

			  let parsed = result.unwrap();
			  assert_eq!(parsed.nutty_id().as_str(), id);
			  assert_eq!(parsed.display_text(), Some(display.trim()));
		 }

		 #[test]
		 fn test_roundtrip_property(
			  id in "[1-9A-HJ-NP-Za-km-z]{7}",
			  display_option in proptest::option::of("[^|]{1,100}")
		 ) {
			  let nutty_id = NuttyId::try_from(id.clone()).unwrap();
			  let tag = NuttyTag::new(nutty_id, display_option.clone().map(|s| s.trim().to_string()));
			  let tag_str = format!("{}", tag);
			  let parsed = NuttyTag::parse(&tag_str).unwrap();

			  assert_eq!(parsed.nutty_id().as_str(), id);
			  assert_eq!(parsed.display_text(), display_option.as_deref().map(str::trim));
		 }
	}

	#[test]
	fn test_try_from() {
		// Valid cases:
		assert!(NuttyTag::try_from("[[abcdefg]]").is_ok());
		assert!(NuttyTag::try_from("[[abcdefg|Display Text]]").is_ok());

		// Invalid cases:
		assert!(NuttyTag::try_from("abcdefg").is_err());
		assert!(NuttyTag::try_from("[[abcdefg").is_err());
		assert!(NuttyTag::try_from("abcdefg]]").is_err());
		assert!(NuttyTag::try_from("[[]]").is_err());
		assert!(NuttyTag::try_from("[[abcdef0]]").is_err());

		// Test error messages.
		let err = NuttyTag::try_from("abcdefg").unwrap_err();
		assert!(err.contains("Invalid NuttyTag format"));

		let err = NuttyTag::try_from("[[abcdef0]]").unwrap_err();
		assert!(err.contains("Invalid Nutty ID format"));
	}

	#[test]
	fn test_edge_cases() {
		// Can we trim whitespace?
		let tag = NuttyTag::parse("[[  abcdefg  ]]").unwrap();
		assert_eq!(tag.nutty_id().as_str(), "abcdefg");

		let tag = NuttyTag::parse("[[ abcdefg |   Display Text   ]]").unwrap();
		assert_eq!(tag.nutty_id().as_str(), "abcdefg");
		assert_eq!(tag.display_text(), Some("Display Text"));

		// Can we have empty strings as display text.
		let tag = NuttyTag::parse("[[abcdefg|]]").unwrap();
		assert_eq!(tag.nutty_id().as_str(), "abcdefg");
		assert_eq!(tag.display_text(), Some(""));
	}
}
