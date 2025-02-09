use uuid::Uuid;

pub trait CommandHandler {
    fn handle(
        &self,
        uuid: Uuid,
    );
}
