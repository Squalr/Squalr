use crate::{project::project::Project, settings::project_settings_config::ProjectSettingsConfig};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

pub struct ProjectManager {
    opened_project: Arc<RwLock<Option<Project>>>,
    watcher: Option<RecommendedWatcher>,
}

impl ProjectManager {
    pub fn new() -> Self {
        let mut instance = ProjectManager {
            opened_project: Arc::new(RwLock::new(None)),
            watcher: None,
        };

        if let Err(err) = instance.watch_projects_directory() {
            log::error!("Failed to watch projects directory! Projects may not be listed properly: {}", err);
        }

        instance
    }

    /// Sets the project to which we are currently attached.
    pub fn set_opened_project(
        &self,
        project_info: Project,
    ) {
        if let Ok(mut project) = self.opened_project.write() {
            log::info!("Opened project: {}, pid: {}", project_info.get_name(), project_info.get_name());
            *project = Some(project_info);
        }
    }

    /// Clears the project to which we are currently attached.
    pub fn clear_opened_project(&self) {
        if let Ok(mut project) = self.opened_project.write() {
            *project = None;

            log::info!("Project closed.");
        }
    }

    /// Gets a reference to the shared lock containing the currently opened project.
    pub fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>> {
        self.opened_project.clone()
    }

    pub fn watch_projects_directory(&mut self) -> notify::Result<()> {
        let projects_root: PathBuf = ProjectSettingsConfig::get_projects_root();

        // Cancel any existing watcher.
        self.watcher = None;

        // Create a new channel for receiving events.
        let (tx, rx): (Sender<Result<Event, notify::Error>>, Receiver<Result<Event, notify::Error>>) = mpsc::channel();

        // Create the watcher.
        let mut watcher = notify::recommended_watcher(tx)?;

        // Watch only the top-level directory (not recursive).
        watcher.watch(&projects_root, RecursiveMode::NonRecursive)?;

        println!("Watching top-level project directory: {}", projects_root.display());

        // Spawn a thread to handle events.
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    Ok(Event { kind, paths, .. }) => {
                        match kind {
                            EventKind::Create(create_kind) => {
                                //
                            }
                            EventKind::Modify(modify_kind) => {
                                //
                            }
                            EventKind::Remove(remove_kind) => {
                                //
                            }
                            _ => {}
                        }
                    }
                    Err(err) => eprintln!("Watch error: {:?}", err),
                }
            }
        });

        // Store the new watcher.
        self.watcher = Some(watcher);

        Ok(())
    }
}
