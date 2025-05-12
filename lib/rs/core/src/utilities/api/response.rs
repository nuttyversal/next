use serde::Deserialize;
use serde::Serialize;

/// The structure of an API response.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
				// At best, this will yield the error variant name.
				//
				// In the future, when `std::error::Report` is stabilized,
				// we might be able to parse the enum name from the report.
				//
				// See https://doc.rust-lang.org/std/error/struct.Report.html.
				let type_name = format!("{source_err:?}")
					.split(['(', ' '])
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
	use thiserror::Error;

	use super::*;

	#[derive(Debug, Error)]
	#[error("An OuterError occurred: {cause}")]
	pub struct OuterError {
		#[from]
		cause: EnumError,
	}

	#[derive(Debug, Error)]
	pub enum EnumError {
		#[error("An EnumError message: Variant: {cause}")]
		Variant {
			#[from]
			cause: InnerError,
		},
	}

	#[derive(Debug, Error)]
	#[error("An InnerError occurred")]
	pub struct InnerError;

	#[test]
	fn test_error_unwinding() {
		// Arrange.
		let nested_error = OuterError::from(EnumError::from(InnerError));

		// Act.
		let api_error = Error::from_error(&nested_error).with_summary("An error occurred.");

		// Assert.
		assert_eq!(api_error.code, Some("OuterError".to_string()));
		assert_eq!(api_error.summary, Some("An error occurred.".to_string()));

		assert_eq!(
			api_error.trace,
			vec![
				"OuterError".to_string(),
				"Variant".to_string(),
				"InnerError".to_string()
			]
		);

		assert_eq!(
			api_error.message,
			Some(
				[
					"An OuterError occurred:",
					"An EnumError message: Variant:",
					"An InnerError occurred"
				]
				.join(" ")
			)
		);
	}
}
