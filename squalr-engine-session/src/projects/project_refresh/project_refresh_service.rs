use crate::projects::project_refresh::project_refresh_config::ProjectRefreshConfig;
use crate::settings::scan_settings_store::ScanSettingsStore;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use squalr_engine_api::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::{
        catalog_changed::project_catalog_changed_event::ProjectCatalogChangedEvent, closed::project_closed_event::ProjectClosedEvent,
        created::project_created_event::ProjectCreatedEvent, deleted::project_deleted_event::ProjectDeletedEvent,
    },
    project_items::changed::project_items_changed_event::ProjectItemsChangedEvent,
};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, mpsc::RecvTimeoutError},
    thread,
    time::Duration,
};

type ProjectRefreshEventEmitter = Arc<dyn Fn(EngineEvent) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectRefreshWatcherScope {
    ProjectCatalog,
    OpenedProject,
}

/// Owns project refresh invalidation and optional file-system watchers for a session.
pub struct ProjectRefreshService {
    config: ProjectRefreshConfig,
    event_emitter: Arc<RwLock<Option<ProjectRefreshEventEmitter>>>,
    projects_root_path: Option<PathBuf>,
    opened_project_directory_path: Option<PathBuf>,
    projects_root_watcher: Option<RecommendedWatcher>,
    opened_project_watcher: Option<RecommendedWatcher>,
}

impl ProjectRefreshService {
    pub fn new(config: ProjectRefreshConfig) -> Self {
        Self {
            config,
            event_emitter: Arc::new(RwLock::new(None)),
            projects_root_path: None,
            opened_project_directory_path: None,
            projects_root_watcher: None,
            opened_project_watcher: None,
        }
    }

    /// Installs the event emitter used to notify GUI/CLI/TUI listeners about project invalidations.
    pub fn set_event_emitter(
        &self,
        event_emitter: ProjectRefreshEventEmitter,
    ) {
        match self.event_emitter.write() {
            Ok(mut stored_event_emitter) => {
                *stored_event_emitter = Some(event_emitter);
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh event emitter lock: {}", error);
            }
        }
    }

    /// Emits a lightweight invalidation for project item changes.
    pub fn notify_project_items_changed(&self) {
        if !self.config.emit_internal_project_events {
            return;
        }

        self.emit(ProjectItemsChangedEvent { project_root: None });
    }

    /// Emits a lightweight invalidation for project creation.
    pub fn notify_project_created(
        &self,
        project_info: ProjectInfo,
    ) {
        if !self.config.emit_internal_project_events {
            return;
        }

        self.emit(ProjectCreatedEvent {
            project_info: project_info.clone(),
        });
        self.emit(ProjectCatalogChangedEvent {
            changed_project_directory_path: project_info.get_project_directory(),
        });
    }

    /// Emits a lightweight invalidation for project deletion.
    pub fn notify_project_deleted(
        &self,
        project_info: ProjectInfo,
    ) {
        if !self.config.emit_internal_project_events {
            return;
        }

        self.emit(ProjectDeletedEvent {
            project_info: project_info.clone(),
        });
        self.emit(ProjectCatalogChangedEvent {
            changed_project_directory_path: project_info.get_project_directory(),
        });
    }

    /// Emits a lightweight invalidation for project close.
    pub fn notify_project_closed(&self) {
        if !self.config.emit_internal_project_events {
            return;
        }

        self.emit(ProjectClosedEvent {});
    }

    /// Starts watching the project catalog root when file-system watching is enabled.
    pub fn watch_projects_root(
        &mut self,
        projects_root: PathBuf,
    ) -> notify::Result<()> {
        self.projects_root_path = Some(projects_root);
        self.refresh_file_system_watchers()
    }

    /// Starts watching the opened project tree when file-system watching is enabled.
    pub fn watch_opened_project(
        &mut self,
        opened_project_directory_path: Option<PathBuf>,
    ) -> notify::Result<()> {
        self.opened_project_directory_path = opened_project_directory_path;
        self.refresh_file_system_watchers()
    }

    /// Applies the current file-system watcher setting immediately.
    pub fn set_file_system_watch_enabled(
        &mut self,
        watch_file_system: bool,
    ) -> notify::Result<()> {
        if self.config.watch_file_system == watch_file_system {
            return Ok(());
        }

        self.config.watch_file_system = watch_file_system;
        self.refresh_file_system_watchers()
    }

    fn refresh_file_system_watchers(&mut self) -> notify::Result<()> {
        self.projects_root_watcher = None;
        self.opened_project_watcher = None;

        if !self.should_watch_file_system() {
            return Ok(());
        }

        if let Some(projects_root_path) = self.projects_root_path.clone() {
            self.projects_root_watcher = Some(Self::create_watcher(
                projects_root_path,
                RecursiveMode::NonRecursive,
                "project catalog",
                ProjectRefreshWatcherScope::ProjectCatalog,
                self.event_emitter.clone(),
            )?);
        }

        if let Some(opened_project_directory_path) = self.opened_project_directory_path.clone() {
            self.opened_project_watcher = Some(Self::create_watcher(
                opened_project_directory_path,
                RecursiveMode::Recursive,
                "opened project",
                ProjectRefreshWatcherScope::OpenedProject,
                self.event_emitter.clone(),
            )?);
        }

        Ok(())
    }

    fn emit<Event>(
        &self,
        event: Event,
    ) where
        Event: EngineEventRequest,
    {
        match self.event_emitter.read() {
            Ok(stored_event_emitter) => {
                if let Some(event_emitter) = stored_event_emitter.as_ref() {
                    let event_emitter = event_emitter.clone();
                    let engine_event = event.to_engine_event();

                    thread::spawn(move || {
                        event_emitter(engine_event);
                    });
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh event emitter lock: {}", error);
            }
        }
    }

    fn create_watcher(
        watched_path: PathBuf,
        recursive_mode: RecursiveMode,
        label: &'static str,
        watcher_scope: ProjectRefreshWatcherScope,
        event_emitter: Arc<RwLock<Option<ProjectRefreshEventEmitter>>>,
    ) -> notify::Result<RecommendedWatcher> {
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(event_sender)?;

        if watcher_scope == ProjectRefreshWatcherScope::ProjectCatalog {
            if let Err(error) = fs::create_dir_all(&watched_path) {
                log::error!("Failed to create project catalog directory before watching: {}", error);
            }
        }

        watcher.watch(Path::new(&watched_path), recursive_mode)?;
        log::info!("Watching {} directory: {}", label, watched_path.display());

        thread::spawn(move || {
            while let Ok(event_result) = event_receiver.recv() {
                match event_result {
                    Ok(_event) => {
                        Self::drain_debounced_events(&event_receiver);
                        Self::emit_watcher_event(watcher_scope, &event_emitter);
                    }
                    Err(error) => log::error!("Project watcher error: {:?}", error),
                }
            }
        });

        Ok(watcher)
    }

    fn emit_watcher_event(
        watcher_scope: ProjectRefreshWatcherScope,
        event_emitter: &Arc<RwLock<Option<ProjectRefreshEventEmitter>>>,
    ) {
        if !ScanSettingsStore::get_project_file_system_watch_enabled() {
            return;
        }

        match event_emitter.read() {
            Ok(stored_event_emitter) => {
                if let Some(event_emitter) = stored_event_emitter.as_ref() {
                    match watcher_scope {
                        ProjectRefreshWatcherScope::ProjectCatalog => {
                            event_emitter(
                                ProjectCatalogChangedEvent {
                                    changed_project_directory_path: None,
                                }
                                .to_engine_event(),
                            );
                        }
                        ProjectRefreshWatcherScope::OpenedProject => {
                            event_emitter(ProjectItemsChangedEvent { project_root: None }.to_engine_event());
                        }
                    }
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh event emitter lock: {}", error);
            }
        }
    }

    fn drain_debounced_events(event_receiver: &std::sync::mpsc::Receiver<notify::Result<notify::Event>>) {
        loop {
            match event_receiver.recv_timeout(Duration::from_millis(150)) {
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => return,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn should_watch_file_system(&self) -> bool {
        self.config.watch_file_system && ScanSettingsStore::get_project_file_system_watch_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectRefreshService;
    use crate::projects::project_refresh::project_refresh_config::ProjectRefreshConfig;
    use crossbeam_channel::unbounded;
    use squalr_engine_api::events::{engine_event::EngineEvent, project_items::project_items_event::ProjectItemsEvent};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn notify_project_items_changed_emits_lightweight_event_when_enabled() {
        let project_refresh_service = ProjectRefreshService::new(ProjectRefreshConfig {
            emit_internal_project_events: true,
            watch_file_system: false,
        });
        let (event_sender, event_receiver) = unbounded();

        project_refresh_service.set_event_emitter(Arc::new(move |event| {
            event_sender
                .send(event)
                .expect("Expected test event channel to accept project refresh event.");
        }));

        project_refresh_service.notify_project_items_changed();

        let event = event_receiver
            .recv_timeout(Duration::from_millis(250))
            .expect("Expected project refresh service to emit project items changed event.");

        assert!(matches!(
            event,
            EngineEvent::ProjectItems(ProjectItemsEvent::ProjectItemsChanged { project_items_changed_event })
                if project_items_changed_event.project_root.is_none()
        ));
    }

    #[test]
    fn notify_project_items_changed_does_not_emit_when_internal_events_disabled() {
        let project_refresh_service = ProjectRefreshService::new(ProjectRefreshConfig {
            emit_internal_project_events: false,
            watch_file_system: false,
        });
        let (event_sender, event_receiver) = unbounded();

        project_refresh_service.set_event_emitter(Arc::new(move |event| {
            event_sender
                .send(event)
                .expect("Expected test event channel to accept project refresh event.");
        }));

        project_refresh_service.notify_project_items_changed();

        assert!(event_receiver.try_recv().is_err());
    }
}
