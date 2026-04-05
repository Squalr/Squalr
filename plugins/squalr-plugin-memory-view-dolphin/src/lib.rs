pub mod address_space;

mod constants;
mod discovery;
mod instance;
mod plugin;
mod process_detection;

pub use plugin::DolphinMemoryViewPlugin;

#[cfg(test)]
mod tests;
