use crate::models::NuttyId;

/// A link between two blocks of content.
#[derive(Debug, Clone)]
pub struct ContentLink {
	pub nutty_id: NuttyId,
	pub source_id: NuttyId,
	pub target_id: NuttyId,
}

impl ContentLink {
	/// Create a new content link.
	pub fn new(nutty_id: NuttyId, source_id: NuttyId, target_id: NuttyId) -> Self {
		Self {
			nutty_id,
			source_id,
			target_id,
		}
	}

	/// Create a new content link with a generated identifier (UUIDv7).
	pub fn now(source_id: NuttyId, target_id: NuttyId) -> Self {
		Self::new(NuttyId::now(), source_id, target_id)
	}
}
