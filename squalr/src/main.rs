#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod app_context;
pub mod models;
pub mod ui;
pub mod views;

use anyhow::{Context, Result, anyhow};
use app::App;
use eframe::NativeOptions;
use eframe::egui::{IconData, ViewportBuilder};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

static ICON_APP: &[u8] = include_bytes!("../images/app/app_icon.png");
static APP_NAME: &str = "Squalr";

pub fn main() -> Result<()> {
    // Create a standalone engine (same process for gui and engine).
    let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).context("Fatal error initializing Squalr engine.")?;

    let icon = image::load_from_memory(ICON_APP)
        .unwrap_or_default()
        .into_rgba8();
    let icon_width = icon.width();
    let icon_height = icon.height();

    // Register app icon, set window size, and disable default window border so that we can add a custom one.
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([512.0, 256.0])
            .with_decorations(false)
            .with_transparent(true),
        ..NativeOptions::default()
    };

    // Run the gui.
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|creation_context| {
            if let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state() {
                let app = App::new(
                    &creation_context.egui_ctx,
                    engine_unprivileged_state.clone(),
                    squalr_engine.get_dependency_container(),
                    APP_NAME.to_string(),
                );

                // Now that gui dependencies are registered, start the engine fully.
                squalr_engine.initialize();

                Ok(Box::new(app))
            } else {
                Err("Failed to start Squalr engine!".into())
            }
        }),
    )
    .map_err(|error| anyhow!(error.to_string()))
    .context("Fatal error in Squalr event loop.")?;

    Ok(())
}
