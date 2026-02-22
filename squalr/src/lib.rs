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
#[cfg(target_os = "android")]
use log::LevelFilter;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

static ICON_APP: &[u8] = include_bytes!("../images/app/app_icon.png");
static APP_NAME: &str = "Squalr";

#[cfg(target_os = "android")]
pub use android_activity::AndroidApp;

#[cfg(target_os = "android")]
static ANDROID_LOGCAT_INIT: OnceLock<()> = OnceLock::new();

#[cfg(target_os = "android")]
fn initialize_android_logcat_logger() {
    ANDROID_LOGCAT_INIT.get_or_init(|| {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(LevelFilter::Info)
                .with_tag("Squalr"),
        );
    });
}

#[cfg(target_os = "android")]
fn log_android_startup_breadcrumb(message: &str) {
    log::info!("[android_bootstrap] {message}");
}

#[cfg(not(target_os = "android"))]
fn log_android_startup_breadcrumb(_message: &str) {}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: AndroidApp) {
    initialize_android_logcat_logger();
    log_android_startup_breadcrumb("android_main entered.");

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
    log_android_startup_breadcrumb("Preparing Android native options.");
    let native_options = create_android_native_options(android_app);
    log_android_startup_breadcrumb("Android native options prepared.");
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
    log_android_startup_breadcrumb("Before SqualrEngine::new.");
    let mut squalr_engine = SqualrEngine::new(engine_mode).context("Fatal error initializing Squalr engine.")?;
    log_android_startup_breadcrumb("After SqualrEngine::new.");
    log_android_startup_breadcrumb("Before eframe::run_native.");

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|creation_context| {
            log_android_startup_breadcrumb("run_native app creator entered.");
            if let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state() {
                log_android_startup_breadcrumb("Before App::new.");
                let app = App::new(
                    &creation_context.egui_ctx,
                    engine_unprivileged_state.clone(),
                    squalr_engine.get_dependency_container(),
                    APP_NAME.to_string(),
                );
                log_android_startup_breadcrumb("After App::new.");

                log_android_startup_breadcrumb("Before squalr_engine.initialize.");
                squalr_engine.initialize();
                log_android_startup_breadcrumb("After squalr_engine.initialize.");

                Ok(Box::new(app))
            } else {
                Err("Failed to start Squalr engine.".into())
            }
        }),
    )
    .map_err(|error| anyhow!(error.to_string()))
    .context("Fatal error in Squalr event loop.")?;

    log_android_startup_breadcrumb("After eframe::run_native returned.");

    Ok(())
}
