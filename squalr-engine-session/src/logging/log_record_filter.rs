use log::Record;

const ANDROID_INPUT_AVAILABLE_TARGET: &str = "android_activity::game_activity";
const ANDROID_INPUT_AVAILABLE_MESSAGE: &str = "Notifying Input Available";

pub fn should_suppress_record(record: &Record) -> bool {
    record.target().starts_with(ANDROID_INPUT_AVAILABLE_TARGET)
        && record
            .args()
            .to_string()
            .contains(ANDROID_INPUT_AVAILABLE_MESSAGE)
}
