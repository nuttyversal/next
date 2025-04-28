use serde::{Deserialize, Serialize};

/// The structure of an API response.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
	/// A single resource object.
	Single { data: Option<T> },

	/// An array of resource objects.
	Multiple { data: Vec<T> },

	/// An array of errors.
	Error { errors: Vec<Error> },
}

impl<T> Response<T> {
	/// Extracts the resource object from the document.
	pub fn extract_object(&self) -> Option<&T> {
		match self {
			Response::Single { data } => data.as_ref(),
			_ => None,
		}
	}

	/// Extracts the resource objects from the document.
	pub fn extract_objects(&self) -> Vec<&T> {
		match self {
			Response::Single { data } => data.as_ref().into_iter().collect(),
			Response::Multiple { data } => data.iter().collect(),
			_ => vec![],
		}
	}
}

/// An error object.
#[derive(Serialize, Deserialize)]
pub struct Error {
	/// An application-specific error code (i.e., the error variant name).
	#[serde(skip_serializing_if = "Option::is_none")]
	code: Option<String>,

	/// An unwinding of the error chain from the outermost to the innermost.
	trace: Vec<String>,

	/// The contents of the error message.
	#[serde(skip_serializing_if = "Option::is_none")]
	message: Option<String>,

	/// A short, human-readable summary of the problem.
	#[serde(skip_serializing_if = "Option::is_none")]
	summary: Option<String>,
}

impl Error {
	/// Create an [Error] from an [std::error::Error].
	pub fn from_error<E>(error: &E) -> Self
	where
		E: std::error::Error + 'static,
	{
		// Extract the outermost error variant name as the code.
		// E.g., crate::errors::DatabaseQueryError → DatabaseQueryError.
		let code = Some(
			std::any::type_name::<E>()
				.split("::")
				.last()
				.unwrap_or("UnknownError")
				.to_string(),
		);

		// Build the trace by unwinding the error chain.
		let mut trace = Vec::new();
		let mut current_error: Option<&dyn std::error::Error> = Some(error);

		// Add the outermost error to the trace first.
		trace.push(code.clone().unwrap_or_else(|| "UnknownError".to_string()));

		// Unwind to collect inner errors.
		while let Some(err) = current_error {
			if let Some(source_err) = err.source() {
				// Since we lose concrete type information on a `&dyn Error`,
				// we use the {:?} representation to identify the error type.
				// E.g., QueryError { source: RowNotFound } → QueryError.
				let type_name = format!("{:?}", source_err)
					.split(' ')
					.next()
					.unwrap_or("UnknownError")
					.to_string();

				trace.push(type_name);
				current_error = Some(source_err);
			} else {
				current_error = None;
			}
		}

		// Get the full error message.
		let message = Some(error.to_string());

		Error {
			code,
			trace,
			message,
			summary: None,
		}
	}

	/// Attach a summary to this [Error].
	pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
		self.summary = Some(summary.into());
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fmt;

	#[derive(Debug)]
	struct RowNotFound;

	impl fmt::Display for RowNotFound {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(
				f,
				"No rows returned by a query that expected to return at least one row"
			)
		}
	}

	impl std::error::Error for RowNotFound {}

	#[derive(Debug)]
	struct DatabaseQueryError {
		source: RowNotFound,
	}

	impl fmt::Display for DatabaseQueryError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "Database query failed: {}", self.source)
		}
	}

	impl std::error::Error for DatabaseQueryError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			Some(&self.source)
		}
	}

	#[derive(Debug)]
	struct ArticleNotFoundError {
		slug: String,
	}

	impl fmt::Display for ArticleNotFoundError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "Article with slug '{}' not found", self.slug)
		}
	}

	impl std::error::Error for ArticleNotFoundError {}

	#[test]
	fn test_error_from_simple_error() {
		// Arrange: Create a simple error with no nested errors.
		let article_error = ArticleNotFoundError {
			slug: "getting-started".to_string(),
		};

		// Act: Create an Error from the simple error.
		let error = Error::from_error(&article_error);

		// Assert: The error fields match the expected values.
		assert_eq!(error.code, Some("ArticleNotFoundError".to_string()));

		assert_eq!(
			error.trace.first(),
			Some(&"ArticleNotFoundError".to_string())
		);

		assert_eq!(
			error.message,
			Some("Article with slug 'getting-started' not found".to_string())
		);

		assert_eq!(error.summary, None);
	}

	#[test]
	fn test_error_from_nested_error() {
		// Arrange: Create a nested error chain.
		let row_not_found = RowNotFound;

		let db_error = DatabaseQueryError {
			source: row_not_found,
		};

		// Act: Create an Error from the nested error.
		let error = Error::from_error(&db_error);

		// Assert: The error contains the expected information.
		assert_eq!(error.code, Some("DatabaseQueryError".to_string()));

		assert!(
			error.trace.len() >= 2,
			"Error trace should include at least two entries"
		);

		assert_eq!(
			error.message,
			Some("Database query failed: No rows returned by a query that expected to return at least one row".to_string())
		);
	}

	#[test]
	fn test_error_with_summary() {
		// Arrange: Create a simple error.
		let article_error = ArticleNotFoundError {
			slug: "getting-started".to_string(),
		};

		// Act: Create an Error with a summary.
		let error = Error::from_error(&article_error)
			.with_summary("The article you requested does not exist.");

		// Assert: The summary is correctly attached.
		assert_eq!(
			error.summary,
			Some("The article you requested does not exist.".to_string())
		);
	}

	#[test]
	fn test_error_serialization() {
		// Arrange: Create errors to serialize.
		let article_error = ArticleNotFoundError {
			slug: "getting-started".to_string(),
		};

		let row_not_found = RowNotFound;

		let db_error = DatabaseQueryError {
			source: row_not_found,
		};

		// Act: Create and serialize an Error with a summary.
		let error_with_summary = Error::from_error(&article_error)
			.with_summary("The article you requested does not exist.");
		let json_with_summary = serde_json::to_string(&error_with_summary).unwrap();

		// Act: Create and serialize a nested error.
		let nested_error = Error::from_error(&db_error);
		let json_nested = serde_json::to_string(&nested_error).unwrap();

		// Assert: The serialized JSON contains the expected fields.
		let parsed_with_summary: serde_json::Value =
			serde_json::from_str(&json_with_summary).unwrap();

		assert_eq!(
			parsed_with_summary["code"].as_str().unwrap(),
			"ArticleNotFoundError"
		);

		assert_eq!(
			parsed_with_summary["message"].as_str().unwrap(),
			"Article with slug 'getting-started' not found"
		);

		assert_eq!(
			parsed_with_summary["summary"].as_str().unwrap(),
			"The article you requested does not exist."
		);

		let expected_trace = vec!["ArticleNotFoundError"];
		let actual_trace: Vec<&str> = parsed_with_summary["trace"]
			.as_array()
			.unwrap()
			.iter()
			.map(|v| v.as_str().unwrap())
			.collect();

		assert_eq!(actual_trace, expected_trace);

		// Assert: The serialized JSON contains the expected fields.
		let parsed_nested: serde_json::Value = serde_json::from_str(&json_nested).unwrap();

		assert_eq!(
			parsed_nested["code"].as_str().unwrap(),
			"DatabaseQueryError"
		);

		assert_eq!(
			parsed_nested["message"].as_str().unwrap(),
			"Database query failed: No rows returned by a query that expected to return at least one row"
		);

		let expected_nested_trace = vec!["DatabaseQueryError", "RowNotFound"];
		let actual_nested_trace: Vec<&str> = parsed_nested["trace"]
			.as_array()
			.unwrap()
			.iter()
			.map(|v| v.as_str().unwrap())
			.collect();

		assert_eq!(actual_nested_trace, expected_nested_trace);
	}
}
