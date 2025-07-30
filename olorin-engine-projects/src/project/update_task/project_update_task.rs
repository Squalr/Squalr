use crate::project::project::Project;
use crate::settings::project_settings_config::ProjectSettingsConfig;
use olorin_engine_api::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use olorin_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use olorin_engine_api::structures::tasks::trackable_task::TrackableTask;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

const TASK_NAME: &'static str = "Project Update Task";

pub struct ProjectUpdateTask;

/// Implementation of a task that updates all project items.
impl ProjectUpdateTask {
    pub fn start_task(
        opened_project: Arc<RwLock<Option<Project>>>,
        opened_process: Arc<RwLock<Option<OpenedProcessInfo>>>,
        project_item_type_registry: Arc<RwLock<ProjectItemTypeRegistry>>,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();

        std::thread::spawn(move || {
            loop {
                if task_clone.get_cancellation_token().load(Ordering::Acquire) {
                    break;
                }
                Self::update_project_task(&opened_project, &opened_process, &project_item_type_registry);

                thread::sleep(Duration::from_millis(ProjectSettingsConfig::get_project_update_interval()));
            }

            task_clone.complete();
        });

        task
    }

    fn update_project_task(
        opened_project: &Arc<RwLock<Option<Project>>>,
        opened_process: &Arc<RwLock<Option<OpenedProcessInfo>>>,
        project_item_type_registry: &Arc<RwLock<ProjectItemTypeRegistry>>,
    ) {
        let mut opened_project_guard = match opened_project.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on opened project: {}", error);

                return;
            }
        };
        let opened_process_guard = match opened_process.read() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on process info for project updates: {}", error);

                return;
            }
        };
        let project_item_type_registry_guard = match project_item_type_registry.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return;
            }
        };

        // Call tick on the project root, which will in turn recursively tick all project items.
        if let Some(opened_project) = opened_project_guard.as_mut() {
            let project_root = opened_project.get_project_root_mut();

            if let Some(project_item_type) = project_item_type_registry_guard.get(project_root.get_item_type().get_project_item_type_id()) {
                project_item_type.tick(&opened_process_guard, &project_item_type_registry_guard, project_root);
            }
        }
    }
}
