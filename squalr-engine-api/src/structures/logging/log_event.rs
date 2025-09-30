use log::Level;

#[derive(Clone, Debug)]
pub struct LogEvent {
    pub message: String,
    pub level: Level,
}
