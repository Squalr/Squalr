use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::{collections::HashMap, io::Cursor};

static SUCCESS_WAV: &[u8] = include_bytes!("../../../ui/audio/Success.wav");
static NOTIFICATION_WAV: &[u8] = include_bytes!("../../../ui/audio/Notification.wav");
static WARN_WAV: &[u8] = include_bytes!("../../../ui/audio/Warn.wav");

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SoundType {
    Success,
    Warn,
    Notification,
}

pub struct AudioPlayer {
    stream_handle: Option<OutputStreamHandle>,
    sounds: HashMap<SoundType, &'static [u8]>,
}

static mut INSTANCE: Option<OutputStream> = None;

impl AudioPlayer {
    pub fn new() -> Self {
        let stream_handle = match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                // Keep the stream alive without tying it to the audio player instance, as it does not implement Send + Sync.
                unsafe {
                    INSTANCE = Some(stream);
                }

                Some(stream_handle)
            }
            Err(err) => {
                log::error!("Failed to initialize audio player: {}", err);
                None
            }
        };

        let sounds = Self::load_sounds();

        Self { stream_handle, sounds }
    }

    pub fn play_sound(
        &self,
        sound: SoundType,
    ) {
        if let Some(data) = self.sounds.get(&sound) {
            let cursor = Cursor::new(*data);
            match Decoder::new(cursor) {
                Ok(source) => {
                    if let Some(stream_handle) = &self.stream_handle {
                        match Sink::try_new(stream_handle) {
                            Ok(sink) => {
                                sink.append(source);
                                sink.detach();
                            }
                            Err(err) => {
                                log::error!("Error creating audio sink: {}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error creating audio decoder: {}", err);
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
