use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::{collections::HashMap, io::Cursor};

static SUCCESS_WAV: &[u8] = include_bytes!("../../audio/Success.wav");
static NOTIFICATION_WAV: &[u8] = include_bytes!("../../audio/Notification.wav");
static WARN_WAV: &[u8] = include_bytes!("../../audio/Warn.wav");

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SoundType {
    Success,
    Warn,
    Notification,
}

pub struct AudioPlayer {
    output_stream: Option<OutputStream>,
    sounds: HashMap<SoundType, &'static [u8]>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let output_stream = match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Some(stream),
            Err(error) => {
                log::error!("Failed to initialize audio player: {}", error);
                None
            }
        };

        let sounds = Self::load_sounds();

        Self { output_stream, sounds }
    }

    pub fn play_sound(
        &self,
        sound: SoundType,
    ) {
        if let Some(data) = self.sounds.get(&sound) {
            let cursor = Cursor::new(*data);
            match Decoder::new(cursor) {
                Ok(source) => {
                    if let Some(output_stream) = &self.output_stream {
                        let sink = Sink::connect_new(output_stream.mixer());
                        sink.append(source);
                        sink.detach();
                    }
                }
                Err(error) => {
                    log::error!("Error creating audio decoder: {}", error);
                }
            }
        }
    }

    fn load_sounds() -> HashMap<SoundType, &'static [u8]> {
        let mut sounds = HashMap::new();

        // Directly store references to the static byte slices instead of making copies.
        sounds.insert(SoundType::Success, SUCCESS_WAV);
        sounds.insert(SoundType::Warn, WARN_WAV);
        sounds.insert(SoundType::Notification, NOTIFICATION_WAV);

        sounds
    }
}
