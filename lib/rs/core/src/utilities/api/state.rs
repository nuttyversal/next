use crate::content::service::ContentService;
use crate::navigator::service::NavigatorService;

#[derive(Clone)]
pub struct AppState {
	pub content_service: ContentService,
	pub navigator_service: NavigatorService,
}
