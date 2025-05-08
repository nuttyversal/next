use crate::models::Navigator;
use crate::models::navigator::NavigatorError;
use crate::repository::navigator::NavigatorRepository;
use crate::repository::navigator::NavigatorRepositoryError;

pub struct NavigatorService {
	repository: NavigatorRepository,
}

impl NavigatorService {
	/// Create a new navigator service with the given repository.
	pub fn new(repository: NavigatorRepository) -> Self {
		NavigatorService { repository }
	}

	/// Register a [Navigator].
	pub async fn register(
		&self,
		name: String,
		pass: String,
	) -> Result<Navigator, NavigatorServiceError> {
		let navigator = Navigator::new(name, &pass).map_err(NavigatorServiceError::Create)?;

		self
			.repository
			.create_navigator(navigator)
			.await
			.map_err(NavigatorServiceError::Insert)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum NavigatorServiceError {
	#[error("Failed to create navigator: {0}")]
	Create(#[from] NavigatorError),

	#[error("Failed to create navigator: {0}")]
	Insert(#[from] NavigatorRepositoryError),
}
