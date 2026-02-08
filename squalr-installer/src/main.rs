#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod installer_runtime;
mod logging;
mod theme;
mod ui_assets;
mod ui_state;
mod views;

use crate::app::InstallerApp;
use crate::logging::initialize_logger;
use crate::ui_assets::{APP_NAME, load_app_icon_data};
use crate::ui_state::InstallerUiState;
use anyhow::{Context, Result, anyhow};
use eframe::NativeOptions;
use eframe::egui::ViewportBuilder;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    let ui_state = Arc::new(Mutex::new(InstallerUiState::new()));

    let mut viewport_builder = ViewportBuilder::default()
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size([640.0, 640.0])
        .with_min_inner_size([480.0, 420.0]);

    if let Some(icon_data) = load_app_icon_data() {
        viewport_builder = viewport_builder.with_icon(icon_data);
    }

    let native_options = NativeOptions {
        viewport: viewport_builder,
        ..NativeOptions::default()
    };

    // Run the gui.
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |creation_context| {
            initialize_logger(ui_state.clone(), creation_context.egui_ctx.clone()).context("Failed to initialize installer logger")?;

            Ok(Box::new(InstallerApp::new(&creation_context.egui_ctx, ui_state.clone())))
        }),
    )
    .map_err(|error| anyhow!(error.to_string()))
    .context("Fatal error starting installer GUI")?;

    Ok(())
}
