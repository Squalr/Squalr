use anyhow::Result;
use squalr_engine::engine_mode::EngineMode;

pub fn main() -> Result<()> {
    squalr::run_gui(EngineMode::Standalone)
}
