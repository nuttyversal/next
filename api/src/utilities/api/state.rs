use crate::access::service::AccessService;
use crate::content::service::ContentService;
use crate::navigator::service::NavigatorService;

#[derive(Clone)]
pub struct AppState {
	pub access_service: AccessService,
	pub content_service: ContentService,
	pub navigator_service: NavigatorService,
}
