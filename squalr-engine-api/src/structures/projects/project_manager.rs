use crate::structures::projects::{project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest};
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
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

        if let Err(error) = instance.watch_projects_directory() {
            log::error!("Failed to watch projects directory! Projects may not be listed properly: {}", error);
        }

        instance
    }

    /// Dispatches an engine event indicating that the project items have changed.
    pub fn notify_project_items_changed(&self) {
        if self.opened_project.try_read().is_ok() {
            /*
            let project_root = if let Some(project) = project.as_ref() {
                Some(project.get_project_root().clone())
            } else {
                None
            };*/

            // (self.event_emitter)(ProjectItemsChangedEvent { project_root }.to_engine_event());
        }
    }

    /// Gets a reference to the shared lock containing the currently opened project.
    /// Take caution not to directly set the project if the desire is to capture project events.
    /// To capture these, call `set_opened_project` and `close_opened_project` instead.
    pub fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>> {
        self.opened_project.clone()
    }

    fn watch_projects_directory(&mut self) -> notify::Result<()> {
        // Cancel any existing directory watcher threads.
        self.watcher = None;

        let (tx, rx): (Sender<Result<Event, notify::Error>>, Receiver<Result<Event, notify::Error>>) = mpsc::channel();
        let projects_root: PathBuf = PathBuf::new(); // ProjectSettingsConfig::get_projects_root();
        let mut watcher = notify::recommended_watcher(tx)?;

        // Watch only the top-level directory (not recursive) for project changes.
        watcher.watch(&projects_root, RecursiveMode::NonRecursive)?;

        log::info!("Watching project directory: {}", projects_root.display());

        // Spawn a thread to handle events.
        // JIRA: This is a bit jank, we miss icon updates, its not really as robust as it can be, etc.
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    Ok(Event { kind, paths, attrs: _attrs }) => match kind {
                        EventKind::Create(create_kind) => match create_kind {
                            CreateKind::File => {}
                            _ => {
                                for path in paths {
                                    Self::create_project_info(&path);
                                }
                            }
                        },
                        EventKind::Modify(modify_kind) => match modify_kind {
                            ModifyKind::Name(rename_mode) => match rename_mode {
                                RenameMode::From => {
                                    // There should only be one path, but handle this gracefully anyhow.
                                    for path in paths {
                                        Self::create_project_info(&path);
                                    }
                                }
                                RenameMode::To => {
                                    // There should only be one path, but handle this gracefully anyhow.
                                    for path in paths {
                                        Self::create_project_info(&path);
                                    }
                                }
                                RenameMode::Both => {
                                    if paths.len() == 2 {
                                        Self::create_project_info(&paths[0]);
                                        Self::create_project_info(&paths[1]);
                                    } else {
                                        log::warn!("Unsupported file rename operation detected in projects folder. Projects list may be out of sync!");
                                    }
                                }
                                _ => {
                                    log::warn!("Unsupported file system operation detected in projects folder. Projects list may be out of sync!");
                                }
                            },
                            _ => {}
                        },
                        EventKind::Remove(remove_kind) => match remove_kind {
                            RemoveKind::File => {}
                            _ => {
                                for path in paths {
                                    Self::create_project_info(&path);
                                }
                            }
                        },
                        _ => {}
                    },
                    Err(error) => log::error!("Watch error: {:?}", error),
                }
            }
        });

        // Store the new watcher.
        self.watcher = Some(watcher);

        Ok(())
    }

    fn create_project_info(path: &PathBuf) -> ProjectInfo {
        let project_info = ProjectInfo::new(path.clone(), None, ProjectManifest::default());

        project_info
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
