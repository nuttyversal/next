use std::fmt;

use regex::Regex;
use thiserror::Error;

use crate::models::AnyNuttyId;
use crate::models::nutty_id::NuttyIdError;

/// A NuttyTag represents a wikilink-style tag containing a Nutty ID.
///
/// It can be in the form [[abcdefg]] or [[abcdefg|Display Text]],
/// where "abcdefg" is a valid Nutty ID and "Display Text" is
/// optional text to render in the UI instead of the Nutty ID.
#[derive(Debug)]
pub struct NuttyTag {
	nutty_id: AnyNuttyId,
	display_text: Option<String>,
}

impl NuttyTag {
	/// Create a new `NuttyTag` from a Nutty ID and optional display text.
	pub fn new(nutty_id: AnyNuttyId, display_text: Option<String>) -> Self {
		Self {
			nutty_id,
			display_text,
		}
	}

	/// Parse a tag string like [[abcdefg]] or [[abcdefg|Display Text]].
	pub fn parse(value: &str) -> Result<Self, NuttyTagError> {
		// Check for opening and closing brackets.
		if !value.starts_with("[[") || !value.ends_with("]]") {
			return Err(NuttyTagError::MissingBrackets(value.to_string()));
		}

		// Strip the opening and closing brackets.
		let content = &value[2..value.len() - 2];

		// Split by pipe character, if present.
		let parts: Vec<&str> = content.split('|').collect();

		match parts.len() {
			// Format: [[abcdefg]]
			1 => {
				let id_str = parts[0].trim();
				let nutty_id = AnyNuttyId::new(id_str)?;

				Ok(Self {
					nutty_id,
					display_text: None,
				})
			}

			// Format: [[abcdefg|Display Text]]
			2 => {
				let id_str = parts[0].trim();
				let display = parts[1].trim();
				let nutty_id = AnyNuttyId::new(id_str)?;

				Ok(Self {
					nutty_id,
					display_text: Some(display.to_string()),
				})
			}

			// Format: (╯‵Д′)╯彡┻━┻
			_ => Err(NuttyTagError::InvalidTag(value.to_string())),
		}
	}

	/// Parse a given string and extracts a [NuttyTag] list.
	pub fn parse_all(value: &str) -> Vec<Self> {
		let mut tags = Vec::new();

		// Matches [[…]] where … is any character(s) except ]].
		let re = Regex::new(r"\[\[([^]]+)\]\]").unwrap();

		for capture in re.captures_iter(value) {
			let tag_str = capture.get(0).unwrap().as_str();

			if let Ok(tag) = Self::try_from(tag_str) {
				tags.push(tag);
			}
		}

		tags
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &AnyNuttyId {
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
			Some(display) => write!(f, "[[{}|{}]]", self.nutty_id().nid(), display),
			None => write!(f, "[[{}]]", self.nutty_id().nid()),
		}
	}
}

impl TryFrom<&str> for NuttyTag {
	type Error = NuttyTagError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::parse(value)
	}
}

#[derive(Debug, Error)]
pub enum NuttyTagError {
	#[error("Missing surrounding [[ and ]]: {0}")]
	MissingBrackets(String),

	#[error("Invalid tag format: '{0}'")]
	InvalidTag(String),

	#[error("Invalid Nutty ID: {0}")]
	InvalidNuttyId(#[from] NuttyIdError),
}

#[cfg(test)]
mod tests {
	use nutty_id::valid_nutty_id;
	use proptest::collection::vec;
	use proptest::option;
	use proptest::prelude::*;

	use crate::models::*;

	#[test]
	fn test_parse_simple_tag() {
		let tag = NuttyTag::parse("[[abcdefg]]").unwrap();
		assert_eq!(tag.nutty_id().nid(), "abcdefg");
		assert_eq!(tag.display_text(), None);
	}

	#[test]
	fn test_parse_tag_with_display_text() {
		let tag = NuttyTag::parse("[[abcdefg|Display Text]]").unwrap();
		assert_eq!(tag.nutty_id().nid(), "abcdefg");
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
		let nutty_id = AnyNuttyId::new("abcdefg").unwrap();
		let tag = NuttyTag::new(nutty_id, None);
		assert_eq!(format!("{tag}"), "[[abcdefg]]");

		// A tag with display text.
		let nutty_id = AnyNuttyId::new("abcdefg").unwrap();
		let tag = NuttyTag::new(nutty_id, Some("Display Text".to_string()));
		assert_eq!(format!("{tag}"), "[[abcdefg|Display Text]]");
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
	}

	#[test]
	fn test_edge_cases() {
		// Can we trim whitespace?
		let tag = NuttyTag::parse("[[  abcdefg  ]]").unwrap();
		assert_eq!(tag.nutty_id().nid(), "abcdefg");

		let tag = NuttyTag::parse("[[ abcdefg |   Display Text   ]]").unwrap();
		assert_eq!(tag.nutty_id().nid(), "abcdefg");
		assert_eq!(tag.display_text(), Some("Display Text"));

		// Can we have empty strings as display text.
		let tag = NuttyTag::parse("[[abcdefg|]]").unwrap();
		assert_eq!(tag.nutty_id().nid(), "abcdefg");
		assert_eq!(tag.display_text(), Some(""));
	}

	#[test]
	fn test_parse_all() {
		// Test empty string.
		let tags = NuttyTag::parse_all("");
		assert!(tags.is_empty());

		// Test single valid tag.
		let tags = NuttyTag::parse_all("[[abcdefg]]");
		assert_eq!(tags.len(), 1);
		assert_eq!(tags[0].nutty_id().nid(), "abcdefg");
		assert_eq!(tags[0].display_text(), None);

		// Test multiple valid tags.
		let tags = NuttyTag::parse_all("[[abcdefg]] [[1234567|Display Text]]");
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].nutty_id().nid(), "abcdefg");
		assert_eq!(tags[0].display_text(), None);
		assert_eq!(tags[1].nutty_id().nid(), "1234567");
		assert_eq!(tags[1].display_text(), Some("Display Text"));

		// Test mixed content with invalid tags.
		let tags = NuttyTag::parse_all("Hello [[abcdefg]] World [[invalid]] [[1234567|Display]]");
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].nutty_id().nid(), "abcdefg");
		assert_eq!(tags[0].display_text(), None);
		assert_eq!(tags[1].nutty_id().nid(), "1234567");
		assert_eq!(tags[1].display_text(), Some("Display"));

		// Test nested tags (should be treated as invalid).
		let tags = NuttyTag::parse_all("[[abcdefg[[1234567]]]]");
		assert!(tags.is_empty());

		// Test tags with whitespace.
		let tags = NuttyTag::parse_all("  [[  abcdefg  ]] & [[  1234567  |  Display  ]]  ");
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].nutty_id().nid(), "abcdefg");
		assert_eq!(tags[0].display_text(), None);
		assert_eq!(tags[1].nutty_id().nid(), "1234567");
		assert_eq!(tags[1].display_text(), Some("Display"));
	}

	proptest! {
		 #[test]
		 fn test_parse_valid_tag_property(id in valid_nutty_id()) {
			  let tag_str = format!("[[{id}]]");
			  let result = NuttyTag::parse(&tag_str);
			  assert!(result.is_ok());

			  let parsed = result.unwrap();
			  assert_eq!(parsed.nutty_id().nid(), id);
			  assert_eq!(parsed.display_text(), None);
		 }

		 #[test]
		 fn test_parse_valid_tag_with_display_property(
			  id in valid_nutty_id(),
			  display in "[^|]{1,100}"
		 ) {
			  let tag_str = format!("[[{id}|{display}]]");
			  let result = NuttyTag::parse(&tag_str);
			  assert!(result.is_ok());

			  let parsed = result.unwrap();
			  assert_eq!(parsed.nutty_id().nid(), id);
			  assert_eq!(parsed.display_text(), Some(display.trim()));
		 }

		 #[test]
		 fn test_roundtrip_property(
			  id in valid_nutty_id(),
			  display_option in proptest::option::of("[^|]{1,100}")
		 ) {
			  let nutty_id = AnyNuttyId::new(&id).unwrap();
			  let tag = NuttyTag::new(nutty_id, display_option.clone().map(|s| s.trim().to_string()));
			  let tag_str = format!("{tag}");
			  let parsed = NuttyTag::parse(&tag_str).unwrap();

			  assert_eq!(parsed.nutty_id().nid(), id);
			  assert_eq!(parsed.display_text(), display_option.as_deref().map(str::trim));
		 }

		#[test]
		fn test_parse_all_property(
			ids in vec(valid_nutty_id(), 1..10),
			displays in vec(option::of("[^|\\]]{1,100}"), 1..10),
			random_text in vec("[^\\[]*", 1..10)
		) {
			// Create a string with alternating tags and random text.
			let mut text = String::new();

			for ((id, display), random) in ids.iter().zip(displays.iter()).zip(random_text.iter()) {
				// Add random text before each tag.
				text.push_str(random);

				// Add the tag.
				match display {
					Some(display) => text.push_str(&format!("[[{id}|{display}]]")),
					None => text.push_str(&format!("[[{id}]]")),
				}
			}

			// Parse all tags.
			let tags = NuttyTag::parse_all(&text);

			// Verify each tag.
			for (tag, (id, display)) in tags.iter().zip(ids.iter().zip(displays.iter())) {
				assert_eq!(tag.nutty_id().nid(), *id);
				assert_eq!(tag.display_text(), display.as_deref().map(str::trim));
			}
		}
	}
}
