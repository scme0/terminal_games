use uuid::Uuid;

pub trait HasCloseAndRefreshActions {
    fn get_close_action(id: Uuid) -> Self;
    fn get_refresh_action() -> Self;
}