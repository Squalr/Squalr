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

#[cfg(target_os = "android")]
pub use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    if let Err(error) = run_gui_android(android_app) {
        log::error!("Fatal Android GUI bootstrap failure: {error:?}");
    }
}

/// Runs the Squalr GUI with the provided engine mode.
pub fn run_gui(engine_mode: EngineMode) -> Result<()> {
    let native_options = create_native_options();
    run_gui_with_native_options(engine_mode, native_options)
}

#[cfg(target_os = "android")]
/// Runs the Squalr GUI on Android using a platform-provided app handle.
pub fn run_gui_android(android_app: AndroidApp) -> Result<()> {
    let native_options = create_android_native_options(android_app);
    run_gui_with_native_options(EngineMode::UnprivilegedHost, native_options)
}

fn create_native_options() -> NativeOptions {
    let app_icon = image::load_from_memory(ICON_APP)
        .unwrap_or_default()
        .into_rgba8();
    let icon_width = app_icon.width();
    let icon_height = app_icon.height();

    let mut viewport_builder = ViewportBuilder::default()
        .with_icon(IconData {
            rgba: app_icon.into_raw(),
            width: icon_width,
            height: icon_height,
        })
        .with_inner_size([1280.0, 840.0])
        .with_min_inner_size([512.0, 256.0]);

    #[cfg(not(target_os = "android"))]
    {
        viewport_builder = viewport_builder.with_decorations(false).with_transparent(true);
    }

    NativeOptions {
        viewport: viewport_builder,
        ..NativeOptions::default()
    }
}

#[cfg(target_os = "android")]
fn create_android_native_options(android_app: AndroidApp) -> NativeOptions {
    let mut native_options = create_native_options();
    native_options.android_app = Some(android_app);

    native_options
}

fn run_gui_with_native_options(
    engine_mode: EngineMode,
    native_options: NativeOptions,
) -> Result<()> {
    let mut squalr_engine = SqualrEngine::new(engine_mode).context("Fatal error initializing Squalr engine.")?;

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

                squalr_engine.initialize();

                Ok(Box::new(app))
            } else {
                Err("Failed to start Squalr engine.".into())
            }
        }),
    )
    .map_err(|error| anyhow!(error.to_string()))
    .context("Fatal error in Squalr event loop.")?;

    Ok(())
}
