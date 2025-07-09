use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

use crate::models::NuttyId;
use crate::models::date_time_rfc_3339::DateTimeRfc3339;

/// A permission that can be granted to roles.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
	name: String,
	description: String,
}

/// A role that groups permissions together.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
	name: String,
	description: String,
}

/// Associates a navigator with a global role.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NavigatorRole {
	#[sqlx(rename = "id")]
	nutty_id: NuttyId,
	navigator_id: NuttyId,
	role_name: String,
	created_at: DateTimeRfc3339,
	updated_at: DateTimeRfc3339,
}

/// Associates a navigator with a role on a specific resource.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceRole {
	#[sqlx(rename = "id")]
	nutty_id: NuttyId,
	navigator_id: Option<NuttyId>,
	role_name: String,
	resource_type: String,
	resource_id: NuttyId,
	created_at: DateTimeRfc3339,
	updated_at: DateTimeRfc3339,
}

impl ResourceRole {
	pub fn role_name(&self) -> &str {
		&self.role_name
	}

	pub fn resource_id(&self) -> &NuttyId {
		&self.resource_id
	}
}

/// A permission check request.
#[derive(Debug, Clone)]
pub struct PermissionCheck {
	navigator_id: Option<NuttyId>,
	permission: String,
	resource_type: Option<String>,
	resource_id: Option<NuttyId>,
}

/// The result of a permission check.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResult {
	/// A permission granted through global role.
	GrantedGlobal,

	/// A permission granted through resource role.
	GrantedResource,

	/// A permission granted through ownership.
	GrantedOwnership,

	/// A permission denied.
	Denied,
}

/// Builder for permission checks.
#[derive(Default)]
pub struct PermissionCheckBuilder {
	navigator_id: Option<NuttyId>,
	permission: Option<String>,
	resource_type: Option<String>,
	resource_id: Option<NuttyId>,
}

impl PermissionCheckBuilder {
	pub fn navigator(mut self, navigator_id: NuttyId) -> Self {
		self.navigator_id = Some(navigator_id);
		self
	}

	pub fn permission(mut self, permission: String) -> Self {
		self.permission = Some(permission);
		self
	}

	pub fn resource(mut self, resource_type: String, resource_id: NuttyId) -> Self {
		self.resource_type = Some(resource_type);
		self.resource_id = Some(resource_id);
		self
	}

	pub fn try_build(self) -> Result<PermissionCheck, PermissionCheckError> {
		let permission = self
			.permission
			.ok_or(PermissionCheckError::MissingPermission)?;

		Ok(PermissionCheck {
			navigator_id: self.navigator_id,
			permission,
			resource_type: self.resource_type,
			resource_id: self.resource_id,
		})
	}
}

impl PermissionCheck {
	pub fn builder() -> PermissionCheckBuilder {
		PermissionCheckBuilder::default()
	}

	pub fn navigator_id(&self) -> Option<&NuttyId> {
		self.navigator_id.as_ref()
	}

	pub fn permission(&self) -> &str {
		&self.permission
	}

	pub fn resource_type(&self) -> Option<&str> {
		self.resource_type.as_deref()
	}

	pub fn resource_id(&self) -> Option<&NuttyId> {
		self.resource_id.as_ref()
	}
}

#[derive(Debug, Error)]
pub enum PermissionCheckError {
	#[error("Permission is required")]
	MissingPermission,
}

#[derive(Debug, Error)]
pub enum AccessError {
	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),

	#[error("Permission check error: {0}")]
	PermissionCheck(#[from] PermissionCheckError),

	#[error("Invalid permission format: {0}")]
	InvalidPermissionFormat(String),
}
