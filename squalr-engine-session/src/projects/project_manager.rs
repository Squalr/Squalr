use crate::projects::project_refresh::project_refresh_config::ProjectRefreshConfig;
use crate::projects::project_refresh::project_refresh_service::ProjectRefreshService;
use crate::settings::scan_settings_store::ScanSettingsStore;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::structures::projects::{project::Project, project_context::ProjectContext, project_info::ProjectInfo};
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub struct ProjectManager {
    opened_project: Arc<RwLock<Option<Project>>>,
    project_refresh_service: RwLock<ProjectRefreshService>,
}

impl ProjectManager {
    pub fn new() -> Self {
        let opened_project = Arc::new(RwLock::new(None));
        let mut project_refresh_service = ProjectRefreshService::new(ProjectRefreshConfig {
            watch_file_system: ScanSettingsStore::get_project_file_system_watch_enabled(),
            ..ProjectRefreshConfig::default()
        });
        project_refresh_service.set_opened_project(opened_project.clone());

        let project_manager = ProjectManager {
            opened_project,
            project_refresh_service: RwLock::new(project_refresh_service),
        };

        project_manager.watch_projects_root(ProjectSettingsConfig::get_projects_root());

        project_manager
    }

    /// Installs the session-local event emitter used for project invalidation events.
    pub fn set_event_emitter(
        &self,
        event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
    ) {
        match self.project_refresh_service.read() {
            Ok(project_refresh_service) => {
                project_refresh_service.set_event_emitter(event_emitter);
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while setting event emitter: {}", error);
            }
        }
    }

    /// Gets a reference to the shared lock containing the currently opened project.
    /// Take caution not to directly set the project if the desire is to capture project events.
    /// To capture these, call `set_opened_project` and `close_opened_project` instead.
    pub fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>> {
        self.opened_project.clone()
    }

    /// Dispatches an engine event indicating that the project items have changed.
    pub fn notify_project_items_changed(&self) {
        match self.project_refresh_service.read() {
            Ok(project_refresh_service) => {
                project_refresh_service.notify_project_items_changed();
            }
            Err(error) => {
                log::error!(
                    "Failed to acquire project refresh service lock while notifying project items changed: {}",
                    error
                );
            }
        }
    }

    /// Dispatches an engine event indicating that a project has been created.
    pub fn notify_project_created(
        &self,
        project_info: ProjectInfo,
    ) {
        match self.project_refresh_service.read() {
            Ok(project_refresh_service) => {
                project_refresh_service.notify_project_created(project_info);
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while notifying project created: {}", error);
            }
        }
    }

    /// Dispatches an engine event indicating that a project has been deleted.
    pub fn notify_project_deleted(
        &self,
        project_info: ProjectInfo,
    ) {
        match self.project_refresh_service.read() {
            Ok(project_refresh_service) => {
                project_refresh_service.notify_project_deleted(project_info);
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while notifying project deleted: {}", error);
            }
        }
    }

    /// Dispatches an engine event indicating that the opened project has been closed.
    pub fn notify_project_closed(&self) {
        match self.project_refresh_service.read() {
            Ok(project_refresh_service) => {
                project_refresh_service.notify_project_closed();
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while notifying project closed: {}", error);
            }
        }
    }

    /// Watches the project catalog root when file-system watching is enabled.
    pub fn watch_projects_root(
        &self,
        projects_root: PathBuf,
    ) {
        match self.project_refresh_service.write() {
            Ok(mut project_refresh_service) => {
                if let Err(error) = project_refresh_service.watch_projects_root(projects_root) {
                    log::error!("Failed to watch projects root: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while watching projects root: {}", error);
            }
        }
    }

    /// Watches the opened project directory when file-system watching is enabled.
    pub fn watch_opened_project(
        &self,
        opened_project_directory_path: Option<PathBuf>,
    ) {
        match self.project_refresh_service.write() {
            Ok(mut project_refresh_service) => {
                if let Err(error) = project_refresh_service.watch_opened_project(opened_project_directory_path) {
                    log::error!("Failed to watch opened project: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while watching opened project: {}", error);
            }
        }
    }

    /// Applies the file-system watcher setting immediately.
    pub fn set_project_file_system_watch_enabled(
        &self,
        project_file_system_watch_enabled: bool,
    ) {
        match self.project_refresh_service.write() {
            Ok(mut project_refresh_service) => {
                if let Err(error) = project_refresh_service.set_file_system_watch_enabled(project_file_system_watch_enabled) {
                    log::error!("Failed to apply project file-system watcher setting: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire project refresh service lock while applying watcher setting: {}", error);
            }
        }
    }
}

impl ProjectContext for ProjectManager {
    fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>> {
        ProjectManager::get_opened_project(self)
    }

    fn notify_project_items_changed(&self) {
        ProjectManager::notify_project_items_changed(self);
    }

    fn notify_project_created(
        &self,
        project_info: ProjectInfo,
    ) {
        ProjectManager::notify_project_created(self, project_info);
    }

    fn notify_project_deleted(
        &self,
        project_info: ProjectInfo,
    ) {
        ProjectManager::notify_project_deleted(self, project_info);
    }

    fn notify_project_closed(&self) {
        ProjectManager::notify_project_closed(self);
    }

    fn watch_opened_project(
        &self,
        opened_project_directory_path: Option<PathBuf>,
    ) {
        ProjectManager::watch_opened_project(self, opened_project_directory_path);
    }

    fn set_project_file_system_watch_enabled(
        &self,
        project_file_system_watch_enabled: bool,
    ) {
        ProjectManager::set_project_file_system_watch_enabled(self, project_file_system_watch_enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectManager;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn notify_project_items_changed_does_not_block_when_opened_project_write_lock_is_held() {
        let project_manager = ProjectManager::new();
        let opened_project_lock = project_manager.get_opened_project();
        let opened_project_write_guard = opened_project_lock
            .write()
            .expect("Expected to acquire opened project write lock for test.");
        let (completion_sender, completion_receiver) = mpsc::channel();

        thread::scope(|scope| {
            scope.spawn(|| {
                project_manager.notify_project_items_changed();
                completion_sender
                    .send(())
                    .expect("Expected to send completion after notify_project_items_changed.");
            });

            let completion_result = completion_receiver.recv_timeout(Duration::from_millis(250));

            assert!(
                completion_result.is_ok(),
                "notify_project_items_changed should not block while opened project write lock is held."
            );
        });

        drop(opened_project_write_guard);
    }
}
