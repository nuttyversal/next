use sqlx::types::Uuid;

/// A link between two blocks of content.
#[derive(Debug, Copy, Clone)]
pub struct ContentLink {
	pub id: Uuid,
	pub source_id: Uuid,
	pub target_id: Uuid,
}

impl ContentLink {
	pub fn new(id: Uuid, source_id: Uuid, target_id: Uuid) -> Self {
		Self {
			id,
			source_id,
			target_id,
		}
	}
}

impl ContentLink {
	pub fn now(source_id: Uuid, target_id: Uuid) -> Self {
		Self::new(Uuid::now_v7(), source_id, target_id)
	}
}
