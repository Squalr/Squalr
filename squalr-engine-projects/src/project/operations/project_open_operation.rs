use crate::project::{project::Project, project_manager::ProjectManager};

impl ProjectManager {
    /// Sets the project to which we are currently attached.
    pub fn operation_open_project(
        &self,
        project: Project,
    ) {
        let opened_project = self.get_opened_project();

        match opened_project.write() {
            Ok(mut opened_project) => {
                log::info!("Opened project: {}", project.get_name());
                *opened_project = Some(project);
            }
            Err(error) => {
                log::error!("Error opening project: {}", error);
                return;
            }
        }

        self.notify_project_items_changed();
    }
}
