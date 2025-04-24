use crate::{project::project::Project, settings::project_settings_config::ProjectSettingsConfig};
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use squalr_engine_api::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        project::{
            closed::project_closed_event::ProjectClosedEvent, created::project_created_event::ProjectCreatedEvent,
            deleted::project_deleted_event::ProjectDeletedEvent,
        },
    },
    structures::projects::{project_info::ProjectInfo, project_manifest::ProjectManifest},
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
    event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
    watcher: Option<RecommendedWatcher>,
}

impl ProjectManager {
    pub fn new(event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>) -> Self {
        let mut instance = ProjectManager {
            opened_project: Arc::new(RwLock::new(None)),
            event_emitter,
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
            log::info!("Opened project: {}", project_info.get_name());
            *project = Some(project_info);
        }
    }

    /// Clears the project to which we are currently attached.
    pub fn clear_opened_project(&self) {
        if let Ok(mut project) = self.opened_project.write() {
            *project = None;

            log::info!("Project closed.");

            (self.event_emitter)(ProjectClosedEvent {}.to_engine_event());
        }
    }

    /// Gets a reference to the shared lock containing the currently opened project.
    /// Take caution not to directly set the project if the desire is to capture project events.
    /// To capture these, call `set_opened_project` and `clear_opened_project` instead.
    pub fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>> {
        self.opened_project.clone()
    }

    pub fn watch_projects_directory(&mut self) -> notify::Result<()> {
        // Cancel any existing directory watcher threads.
        self.watcher = None;

        let (tx, rx): (Sender<Result<Event, notify::Error>>, Receiver<Result<Event, notify::Error>>) = mpsc::channel();
        let projects_root: PathBuf = ProjectSettingsConfig::get_projects_root();
        let mut watcher = notify::recommended_watcher(tx)?;
        let event_emitter = self.event_emitter.clone();

        // Watch only the top-level directory (not recursive) for project changes.
        watcher.watch(&projects_root, RecursiveMode::NonRecursive)?;

        println!("Watching project directory: {}", projects_root.display());

        // Spawn a thread to handle events.
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    Ok(Event { kind, paths, attrs: _attrs }) => match kind {
                        EventKind::Create(create_kind) => match create_kind {
                            CreateKind::File => {}
                            _ => {
                                for path in paths {
                                    (event_emitter)(
                                        ProjectCreatedEvent {
                                            project_info: Self::create_project_info(&path),
                                        }
                                        .to_engine_event(),
                                    );
                                }
                            }
                        },
                        EventKind::Modify(modify_kind) => match modify_kind {
                            ModifyKind::Name(rename_mode) => match rename_mode {
                                RenameMode::From => {
                                    // There should only be one path, but handle this gracefully anyhow.
                                    for path in paths {
                                        (event_emitter)(
                                            ProjectDeletedEvent {
                                                project_info: Self::create_project_info(&path),
                                            }
                                            .to_engine_event(),
                                        );
                                    }
                                }
                                RenameMode::To => {
                                    // There should only be one path, but handle this gracefully anyhow.
                                    for path in paths {
                                        (event_emitter)(
                                            ProjectCreatedEvent {
                                                project_info: Self::create_project_info(&path),
                                            }
                                            .to_engine_event(),
                                        );
                                    }
                                }
                                RenameMode::Both => {
                                    if paths.len() == 2 {
                                        (event_emitter)(
                                            ProjectDeletedEvent {
                                                project_info: Self::create_project_info(&paths[0]),
                                            }
                                            .to_engine_event(),
                                        );
                                        (event_emitter)(
                                            ProjectCreatedEvent {
                                                project_info: Self::create_project_info(&paths[1]),
                                            }
                                            .to_engine_event(),
                                        );
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
                                    (event_emitter)(
                                        ProjectDeletedEvent {
                                            project_info: Self::create_project_info(&path),
                                        }
                                        .to_engine_event(),
                                    );
                                }
                            }
                        },
                        _ => {}
                    },
                    Err(err) => eprintln!("Watch error: {:?}", err),
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
