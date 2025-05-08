use crate::services::ContentService;
use crate::services::NavigatorService;

pub struct AppState {
	pub content_service: ContentService,
	pub navigator_service: NavigatorService,
}
