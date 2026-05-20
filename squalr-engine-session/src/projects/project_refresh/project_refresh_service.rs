use crate::projects::project_refresh::project_refresh_config::ProjectRefreshConfig;
use crate::settings::scan_settings_store::ScanSettingsStore;
use notify::{
    RecommendedWatcher, RecursiveMode, Watcher,
    event::{EventKind, ModifyKind, RenameMode},
};
use squalr_engine_api::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::{
        catalog_changed::project_catalog_changed_event::ProjectCatalogChangedEvent, closed::project_closed_event::ProjectClosedEvent,
        created::project_created_event::ProjectCreatedEvent, deleted::project_deleted_event::ProjectDeletedEvent,
    },
    project_items::changed::project_items_changed_event::ProjectItemsChangedEvent,
};
use squalr_engine_api::structures::projects::{
    project::Project,
    project_info::ProjectInfo,
    project_items::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item::ProjectItem, project_item_ref::ProjectItemRef},
};
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
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
    opened_project: Option<Arc<RwLock<Option<Project>>>>,
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
            opened_project: None,
            projects_root_path: None,
            opened_project_directory_path: None,
            projects_root_watcher: None,
            opened_project_watcher: None,
        }
    }

    /// Installs the opened-project lock used to reconcile external file-system edits.
    pub fn set_opened_project(
        &mut self,
        opened_project: Arc<RwLock<Option<Project>>>,
    ) {
        self.opened_project = Some(opened_project);
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

        self.emit(ProjectItemsChangedEvent {
            changed_project_paths: Vec::new(),
        });
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
                self.opened_project.clone(),
            )?);
        }

        if let Some(opened_project_directory_path) = self.opened_project_directory_path.clone() {
            self.opened_project_watcher = Some(Self::create_watcher(
                opened_project_directory_path,
                RecursiveMode::Recursive,
                "opened project",
                ProjectRefreshWatcherScope::OpenedProject,
                self.event_emitter.clone(),
                self.opened_project.clone(),
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
        opened_project: Option<Arc<RwLock<Option<Project>>>>,
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
                    Ok(event) => {
                        let mut events = vec![event];
                        Self::drain_debounced_events(&event_receiver, &mut events);
                        Self::emit_watcher_event(watcher_scope, &events, &event_emitter, opened_project.as_ref());
                    }
                    Err(error) => log::error!("Project watcher error: {:?}", error),
                }
            }
        });

        Ok(watcher)
    }

    fn emit_watcher_event(
        watcher_scope: ProjectRefreshWatcherScope,
        events: &[notify::Event],
        event_emitter: &Arc<RwLock<Option<ProjectRefreshEventEmitter>>>,
        opened_project: Option<&Arc<RwLock<Option<Project>>>>,
    ) {
        if !ScanSettingsStore::get_project_file_system_watch_enabled() {
            return;
        }

        let engine_event = match watcher_scope {
            ProjectRefreshWatcherScope::ProjectCatalog => ProjectCatalogChangedEvent {
                changed_project_directory_path: Self::first_event_path(events),
            }
            .to_engine_event(),
            ProjectRefreshWatcherScope::OpenedProject => {
                if !Self::apply_opened_project_file_system_events(opened_project, events) {
                    return;
                }

                ProjectItemsChangedEvent {
                    changed_project_paths: Self::project_refresh_paths(events),
                }
                .to_engine_event()
            }
        };

        match event_emitter.read() {
            Ok(stored_event_emitter) => {
                if let Some(event_emitter) = stored_event_emitter.as_ref() {
                    event_emitter(engine_event);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh event emitter lock: {}", error);
            }
        }
    }

    fn drain_debounced_events(
        event_receiver: &std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
        events: &mut Vec<notify::Event>,
    ) {
        loop {
            match event_receiver.recv_timeout(Duration::from_millis(150)) {
                Ok(Ok(event)) => events.push(event),
                Ok(Err(error)) => log::error!("Project watcher error while draining debounced events: {:?}", error),
                Err(RecvTimeoutError::Timeout) => return,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn should_watch_file_system(&self) -> bool {
        self.config.watch_file_system && ScanSettingsStore::get_project_file_system_watch_enabled()
    }

    fn apply_opened_project_file_system_events(
        opened_project: Option<&Arc<RwLock<Option<Project>>>>,
        events: &[notify::Event],
    ) -> bool {
        let Some(opened_project) = opened_project else {
            return false;
        };

        match opened_project.write() {
            Ok(mut opened_project_guard) => {
                let Some(opened_project) = opened_project_guard.as_mut() else {
                    return false;
                };
                let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
                    return false;
                };

                if opened_project.get_has_unsaved_changes() {
                    log::warn!(
                        "Skipping external project file-system reconciliation for '{}' because the opened project has unsaved changes.",
                        project_directory_path.display()
                    );
                    return false;
                }

                events.iter().fold(false, |did_change_project, event| {
                    Self::apply_opened_project_file_system_event(opened_project, &project_directory_path, event) || did_change_project
                })
            }
            Err(error) => {
                log::error!("Failed to acquire opened project lock for external project reconciliation: {}", error);
                false
            }
        }
    }

    fn apply_opened_project_file_system_event(
        project: &mut Project,
        project_directory_path: &Path,
        event: &notify::Event,
    ) -> bool {
        match event.kind {
            EventKind::Create(_) => Self::upsert_project_paths(project, project_directory_path, &event.paths),
            EventKind::Remove(_) => Self::remove_project_paths(project, project_directory_path, &event.paths),
            EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Any) | EventKind::Modify(ModifyKind::Other) => {
                Self::sync_project_paths(project, project_directory_path, &event.paths)
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                if event.paths.len() >= 2 {
                    let did_remove = Self::remove_project_paths(project, project_directory_path, &event.paths[0..1]);
                    let did_upsert = Self::upsert_project_paths(project, project_directory_path, &event.paths[1..2]);

                    did_remove || did_upsert
                } else {
                    Self::sync_project_paths(project, project_directory_path, &event.paths)
                }
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::From)) => Self::remove_project_paths(project, project_directory_path, &event.paths),
            EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Any | RenameMode::Other)) => {
                Self::sync_project_paths(project, project_directory_path, &event.paths)
            }
            EventKind::Any => Self::sync_project_paths(project, project_directory_path, &event.paths),
            EventKind::Modify(ModifyKind::Metadata(_)) | EventKind::Access(_) | EventKind::Other => false,
        }
    }

    fn sync_project_paths(
        project: &mut Project,
        project_directory_path: &Path,
        paths: &[PathBuf],
    ) -> bool {
        paths.iter().fold(false, |did_change_project, path| {
            let did_change_path = if path.exists() {
                Self::upsert_project_path(project, project_directory_path, path)
            } else {
                Self::remove_project_path(project, project_directory_path, path)
            };

            did_change_path || did_change_project
        })
    }

    fn upsert_project_paths(
        project: &mut Project,
        project_directory_path: &Path,
        paths: &[PathBuf],
    ) -> bool {
        paths.iter().fold(false, |did_change_project, path| {
            Self::upsert_project_path(project, project_directory_path, path) || did_change_project
        })
    }

    fn upsert_project_path(
        project: &mut Project,
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        if Self::is_project_info_path(project_directory_path, path) {
            match ProjectInfo::load_from_path(path) {
                Ok(project_info) => {
                    project.set_project_info(project_info);
                    true
                }
                Err(error) => {
                    log::warn!("Failed to load changed project info '{}': {}.", path.display(), error);
                    false
                }
            }
        } else if Self::is_project_item_file_path(project_directory_path, path) {
            Self::upsert_project_item_file(project, path)
        } else if Self::is_project_item_directory_path(project_directory_path, path) {
            Self::upsert_project_item_directory_tree(project, project_directory_path, path)
        } else {
            false
        }
    }

    fn upsert_project_item_file(
        project: &mut Project,
        path: &Path,
    ) -> bool {
        let project_item_ref = ProjectItemRef::new(path.to_path_buf());
        let was_activated = project
            .get_project_items()
            .get(&project_item_ref)
            .map(|project_item| project_item.get_is_activated())
            .unwrap_or(false);

        match ProjectItem::load_from_path(path) {
            Ok(mut project_item) => {
                if was_activated && !project_item.get_is_activated() {
                    project_item.toggle_activated();
                }
                project
                    .get_project_items_mut()
                    .insert(project_item_ref, project_item);
                true
            }
            Err(error) => {
                log::warn!("Failed to load changed project item '{}': {}.", path.display(), error);
                false
            }
        }
    }

    fn upsert_project_item_directory_tree(
        project: &mut Project,
        project_directory_path: &Path,
        directory_path: &Path,
    ) -> bool {
        if !directory_path.exists() || !directory_path.is_dir() {
            return false;
        }

        let mut did_change_project = Self::upsert_project_item_directory(project, project_directory_path, directory_path);
        let Ok(directory_entries) = fs::read_dir(directory_path) else {
            return did_change_project;
        };

        for directory_entry in directory_entries.flatten() {
            let directory_entry_path = directory_entry.path();
            let did_change_path = if directory_entry_path.is_dir() {
                Self::upsert_project_item_directory_tree(project, project_directory_path, &directory_entry_path)
            } else {
                Self::upsert_project_path(project, project_directory_path, &directory_entry_path)
            };

            did_change_project = did_change_path || did_change_project;
        }

        did_change_project
    }

    fn upsert_project_item_directory(
        project: &mut Project,
        project_directory_path: &Path,
        directory_path: &Path,
    ) -> bool {
        if !Self::is_project_item_directory_path(project_directory_path, directory_path) {
            return false;
        }

        let project_item_ref = ProjectItemRef::new(directory_path.to_path_buf());
        let mut project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);
        project_item.set_has_unsaved_changes(false);

        project
            .get_project_items_mut()
            .insert(project_item_ref, project_item);
        true
    }

    fn remove_project_paths(
        project: &mut Project,
        project_directory_path: &Path,
        paths: &[PathBuf],
    ) -> bool {
        paths.iter().fold(false, |did_change_project, path| {
            Self::remove_project_path(project, project_directory_path, path) || did_change_project
        })
    }

    fn remove_project_path(
        project: &mut Project,
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        if Self::is_project_info_path(project_directory_path, path) {
            log::warn!("Project info was removed from disk for opened project '{}'.", project_directory_path.display());
            return false;
        }

        if !Self::is_project_item_path(project_directory_path, path) {
            return false;
        }

        let root_project_item_ref = project.get_project_root_ref().clone();
        let project_item_refs_to_remove = project
            .get_project_items()
            .keys()
            .filter(|project_item_ref| {
                project_item_ref != &&root_project_item_ref
                    && (project_item_ref.get_project_item_path() == path || project_item_ref.get_project_item_path().starts_with(path))
            })
            .cloned()
            .collect::<Vec<ProjectItemRef>>();

        let did_remove_project_items = !project_item_refs_to_remove.is_empty();
        for project_item_ref in project_item_refs_to_remove {
            project.get_project_items_mut().remove(&project_item_ref);
        }

        did_remove_project_items
    }

    fn first_event_path(events: &[notify::Event]) -> Option<PathBuf> {
        events
            .iter()
            .flat_map(|event| event.paths.iter())
            .next()
            .cloned()
    }

    fn project_refresh_paths(events: &[notify::Event]) -> Vec<PathBuf> {
        let mut changed_project_paths = Vec::new();

        for event_path in events.iter().flat_map(|event| event.paths.iter()) {
            if !changed_project_paths.contains(event_path) {
                changed_project_paths.push(event_path.clone());
            }
        }

        changed_project_paths
    }

    fn is_project_info_path(
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        path == project_directory_path.join(Project::PROJECT_FILE)
    }

    fn is_project_item_directory_path(
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        Self::is_project_item_path(project_directory_path, path) && path.is_dir()
    }

    fn is_project_item_file_path(
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        Self::is_project_item_path(project_directory_path, path)
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case(Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.')))
    }

    fn is_project_item_path(
        project_directory_path: &Path,
        path: &Path,
    ) -> bool {
        path.starts_with(project_directory_path.join(Project::PROJECT_DIR))
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectRefreshService, ProjectRefreshWatcherScope};
    use crate::projects::project_refresh::project_refresh_config::ProjectRefreshConfig;
    use crate::settings::scan_settings_store::ScanSettingsStore;
    use crossbeam_channel::unbounded;
    use notify::{
        Event,
        event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode},
    };
    use squalr_engine_api::events::{engine_event::EngineEvent, project::project_event::ProjectEvent, project_items::project_items_event::ProjectItemsEvent};
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, RwLock};
    use std::time::Duration;
    use tempfile::TempDir;

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
                if project_items_changed_event.changed_project_paths.is_empty()
        ));
    }

    #[test]
    fn watcher_catalog_event_emits_catalog_changed_path() {
        let _watch_setting_guard = ProjectFileSystemWatchSettingGuard::enabled();
        let changed_project_directory_path = PathBuf::from("C:/Projects/Squalr/ExternalProject");
        let (event_sender, event_receiver) = unbounded();
        let event_emitter = Arc::new(RwLock::new(Some(Arc::new(move |event| {
            event_sender
                .send(event)
                .expect("Expected test event channel to accept project catalog event.");
        }) as Arc<dyn Fn(EngineEvent) + Send + Sync>)));
        let event = Event::new(EventKind::Create(CreateKind::Folder)).add_path(changed_project_directory_path.clone());

        ProjectRefreshService::emit_watcher_event(ProjectRefreshWatcherScope::ProjectCatalog, &[event], &event_emitter, None);

        let emitted_event = event_receiver
            .recv_timeout(Duration::from_millis(250))
            .expect("Expected catalog watcher event to emit.");

        assert!(matches!(
            emitted_event,
            EngineEvent::Project(ProjectEvent::ProjectCatalogChanged { project_catalog_changed_event })
                if project_catalog_changed_event.changed_project_directory_path == Some(changed_project_directory_path)
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

    #[test]
    fn apply_opened_project_file_system_events_updates_item_file_and_preserves_activation() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let item_path = write_project_to_disk(&temp_directory, "Original");
        let mut opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let item_ref = ProjectItemRef::new(item_path.clone());
        opened_project
            .get_project_item_mut(&item_ref)
            .expect("Expected loaded project item to exist.")
            .toggle_activated();
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));

        write_project_item_to_disk(&item_path, "Reloaded");

        let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(item_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after reconciliation.");
        let current_project_item = current_project
            .get_project_items()
            .get(&item_ref)
            .expect("Expected reconciled project item to exist.");

        assert_eq!(current_project_item.get_field_name(), "Reloaded");
        assert!(current_project_item.get_is_activated());
        assert!(!current_project.get_has_unsaved_changes());
    }

    #[test]
    fn apply_opened_project_file_system_events_removes_deleted_item_file() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let item_path = write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));

        fs::remove_file(&item_path).expect("Expected project item file to be removed.");

        let event = Event::new(EventKind::Remove(RemoveKind::File)).add_path(item_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after item removal.");

        assert!(
            !current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(item_path))
        );
    }

    #[test]
    fn apply_opened_project_file_system_events_adds_created_directory_tree() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));
        let created_directory_path = temp_directory.path().join(Project::PROJECT_DIR).join("Created");
        fs::create_dir_all(&created_directory_path).expect("Expected created project directory.");
        let created_item_path = created_directory_path.join("mana.json");
        write_project_item_to_disk(&created_item_path, "Mana");

        let event = Event::new(EventKind::Create(CreateKind::Folder)).add_path(created_directory_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after directory create.");

        assert!(
            current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(created_directory_path))
        );
        assert!(
            current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(created_item_path))
        );
    }

    #[test]
    fn apply_opened_project_file_system_events_updates_project_info_file() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));
        let project_info_path = temp_directory.path().join(Project::PROJECT_FILE);

        fs::write(
            &project_info_path,
            r#"{"icon":null,"manifest":{"sort_order":["project_items/updated.json"]},"symbols":{}}"#,
        )
        .expect("Expected updated project info file to be written.");

        let event = Event::new(EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content))).add_path(project_info_path);
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after project info reconciliation.");

        assert_eq!(
            current_project
                .get_project_manifest()
                .get_project_item_sort_order()
                .as_slice(),
            &[PathBuf::from("project_items/updated.json")]
        );
        assert!(!current_project.get_has_unsaved_changes());
    }

    #[test]
    fn apply_opened_project_file_system_events_moves_renamed_item_file() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let item_path = write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));
        let renamed_item_path = item_path.with_file_name("renamed.json");
        fs::rename(&item_path, &renamed_item_path).expect("Expected project item to be renamed.");

        let event = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(item_path.clone())
            .add_path(renamed_item_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after item rename.");

        assert!(
            !current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(item_path))
        );
        assert!(
            current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(renamed_item_path))
        );
    }

    #[test]
    fn apply_opened_project_file_system_events_removes_deleted_directory_subtree() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));
        let removed_directory_path = temp_directory.path().join(Project::PROJECT_DIR).join("Deleted");
        fs::create_dir_all(&removed_directory_path).expect("Expected deleted project directory to be created.");
        let removed_item_path = removed_directory_path.join("removed.json");
        write_project_item_to_disk(&removed_item_path, "Removed");
        let event = Event::new(EventKind::Create(CreateKind::Folder)).add_path(removed_directory_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));
        fs::remove_dir_all(&removed_directory_path).expect("Expected project directory subtree to be removed.");

        let event = Event::new(EventKind::Remove(RemoveKind::Folder)).add_path(removed_directory_path.clone());
        assert!(ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after directory removal.");

        assert!(
            !current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(removed_directory_path))
        );
        assert!(
            !current_project
                .get_project_items()
                .contains_key(&ProjectItemRef::new(removed_item_path))
        );
    }

    #[test]
    fn apply_opened_project_file_system_events_skips_dirty_project() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let item_path = write_project_to_disk(&temp_directory, "Original");
        let mut opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        opened_project.set_has_unsaved_changes(true);
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));

        write_project_item_to_disk(&item_path, "Disk Edit");

        let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(item_path.clone());
        assert!(!ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));

        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project lock to be readable.");
        let current_project = opened_project_guard
            .as_ref()
            .expect("Expected project to remain opened after skipped reconciliation.");
        let current_project_item = current_project
            .get_project_items()
            .get(&ProjectItemRef::new(item_path))
            .expect("Expected current project item to exist.");

        assert_eq!(current_project_item.get_field_name(), "Original");
        assert!(current_project.get_has_unsaved_changes());
    }

    #[test]
    fn apply_opened_project_file_system_events_ignores_non_project_paths() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        write_project_to_disk(&temp_directory, "Original");
        let opened_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load.");
        let opened_project = Arc::new(RwLock::new(Some(opened_project)));
        let ignored_path = temp_directory.path().join("notes.txt");
        fs::write(&ignored_path, "not a project item").expect("Expected ignored file to be written.");

        let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(ignored_path);

        assert!(!ProjectRefreshService::apply_opened_project_file_system_events(Some(&opened_project), &[event]));
    }

    #[test]
    fn project_refresh_paths_deduplicate_debounced_watcher_paths() {
        let item_path = PathBuf::from("C:/Projects/Squalr/Test/project_items/health.json");
        let first_event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(item_path.clone());
        let second_event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(item_path.clone());

        let changed_project_paths = ProjectRefreshService::project_refresh_paths(&[first_event, second_event]);

        assert_eq!(changed_project_paths, vec![item_path]);
    }

    fn write_project_to_disk(
        temp_directory: &TempDir,
        item_name: &str,
    ) -> PathBuf {
        let project_directory_path = temp_directory.path();
        fs::create_dir_all(project_directory_path.join(Project::PROJECT_DIR)).expect("Expected project items directory to be created.");
        fs::write(
            project_directory_path.join(Project::PROJECT_FILE),
            r#"{"icon":null,"manifest":{"sort_order":[]},"symbols":{}}"#,
        )
        .expect("Expected project file to be written.");
        let item_path = project_directory_path
            .join(Project::PROJECT_DIR)
            .join("watched.json");
        write_project_item_to_disk(&item_path, item_name);

        item_path
    }

    fn write_project_item_to_disk(
        item_path: &Path,
        item_name: &str,
    ) {
        let project_item = ProjectItemTypeAddress::new_project_item(item_name, 0x1234, "module", "", DataTypeU8::get_value_from_primitive(7));
        let item_file = File::create(item_path).expect("Expected project item file to be created.");

        serde_json::to_writer_pretty(item_file, &project_item).expect("Expected project item to be serialized.");
    }

    struct ProjectFileSystemWatchSettingGuard {
        original_value: bool,
    }

    impl ProjectFileSystemWatchSettingGuard {
        fn enabled() -> Self {
            let original_value = ScanSettingsStore::get_project_file_system_watch_enabled();
            ScanSettingsStore::set_project_file_system_watch_enabled(true);

            Self { original_value }
        }
    }

    impl Drop for ProjectFileSystemWatchSettingGuard {
        fn drop(&mut self) {
            ScanSettingsStore::set_project_file_system_watch_enabled(self.original_value);
        }
    }
}
