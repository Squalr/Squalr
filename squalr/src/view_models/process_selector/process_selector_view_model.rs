use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::view_models::view_model_base::ViewModel;
use crate::view_models::view_model_base::ViewModelBase;
use slint::ComponentHandle;
use slint::Image;
use slint::Model;
use slint::ModelRc;
use slint::SharedPixelBuffer;
use slint::VecModel;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use std::rc::Rc;
use sysinfo::Pid;

pub struct ProcessSelectorViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_model_base: ViewModelBase<MainWindowView>) -> Self {
        let view = ProcessSelectorViewModel {
            view_model_base: view_model_base,
        };

        view.create_view_bindings();

        return view;
    }

    fn create_process_view_data(process_info: ProcessInfo) -> ProcessViewData {
        let icon = process_info.icon.map_or_else(
            || {
                // Create 1x1 transparent image as fallback
                let mut icon_data = SharedPixelBuffer::new(1, 1);
                let icon_data_bytes = icon_data.make_mut_bytes();
                icon_data_bytes.copy_from_slice(&[0, 0, 0, 0]);
                Image::from_rgba8(icon_data)
            },
            |icon| {
                let mut icon_data = SharedPixelBuffer::new(icon.width, icon.height);
                let icon_data_bytes = icon_data.make_mut_bytes();
                icon_data_bytes.copy_from_slice(&icon.bytes_rgba);
                Image::from_rgba8(icon_data)
            },
        );

        ProcessViewData {
            process_id_str: process_info.pid.to_string().into(),
            process_id: process_info.pid.as_u32() as i32,
            name: process_info.name.to_string().into(),
            icon,
        }
    }

    fn update_process_list(
        process_list: &Rc<VecModel<ProcessViewData>>,
        new_processes: Vec<ProcessInfo>,
    ) {
        // Create a hash map of existing processes for quick lookup
        let mut existing_processes: std::collections::HashMap<i32, usize> = (0..process_list.row_count())
            .filter_map(|i| {
                if let Some(process) = process_list.row_data(i) {
                    Some((process.process_id, i))
                } else {
                    None
                }
            })
            .collect();

        // Track indices that need to be removed
        let mut to_remove: Vec<usize> = Vec::new();

        // First pass: Update existing and mark for removal
        for (index, process_info) in new_processes.iter().enumerate() {
            let pid = process_info.pid.as_u32() as i32;

            if let Some(&existing_index) = existing_processes.get(&pid) {
                // Create new view data
                let new_view_data = Self::create_process_view_data(process_info.clone());

                // Check if data actually changed before updating
                if let Some(current) = process_list.row_data(existing_index) {
                    if current != new_view_data {
                        process_list.set_row_data(existing_index, new_view_data);
                    }
                }
                existing_processes.remove(&pid);
            } else if index < process_list.row_count() {
                // Update existing slot with new process
                process_list.set_row_data(index, Self::create_process_view_data(process_info.clone()));
            } else {
                // Add new process at the end
                process_list.push(Self::create_process_view_data(process_info.clone()));
            }
        }

        // Remove processes that no longer exist (in reverse order to maintain indices)
        let indices: Vec<_> = existing_processes.values().copied().collect();
        for &index in indices.iter().rev() {
            to_remove.push(index);
        }

        for index in to_remove.iter().rev() {
            process_list.remove(*index);
        }

        // Trim excess items if new list is shorter
        while process_list.row_count() > new_processes.len() {
            process_list.remove(process_list.row_count() - 1);
        }
    }

    fn refresh_process_list(
        view_model_base: ViewModelBase<MainWindowView>,
        refresh_windowed_list: bool,
    ) {
        view_model_base.execute_on_ui_thread(move |main_window_view, _view_model_base| {
            let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

            let process_list = if refresh_windowed_list {
                match process_selector_view
                    .get_windowed_processes()
                    .as_any()
                    .downcast_ref::<VecModel<ProcessViewData>>()
                {
                    Some(model) => Rc::new(VecModel::from(model.iter().collect::<Vec<_>>())),
                    None => Rc::new(VecModel::default()),
                }
            } else {
                match process_selector_view
                    .get_processes()
                    .as_any()
                    .downcast_ref::<VecModel<ProcessViewData>>()
                {
                    Some(model) => Rc::new(VecModel::from(model.iter().collect::<Vec<_>>())),
                    None => Rc::new(VecModel::default()),
                }
            };

            let process_query_options = ProcessQueryOptions {
                required_pid: None,
                search_name: None,
                require_windowed: refresh_windowed_list,
                match_case: false,
                fetch_icons: true,
                skip_cache: false,
                limit: None,
            };

            let processes = ProcessQuery::get_processes(process_query_options);
            Self::update_process_list(&process_list, processes);

            // Update the UI with the modified list
            if refresh_windowed_list {
                process_selector_view.set_windowed_processes(ModelRc::from(process_list));
            } else {
                process_selector_view.set_processes(ModelRc::from(process_list));
            }
        });
    }
}

impl ViewModel for ProcessSelectorViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

                // Initialize empty lists
                process_selector_view.set_processes(ModelRc::from(Rc::new(VecModel::<ProcessViewData>::default())));
                process_selector_view.set_windowed_processes(ModelRc::from(Rc::new(VecModel::<ProcessViewData>::default())));

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_full_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), false);
                });

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_windowed_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), true);
                });

                process_selector_view.on_select_process(|process_entry| {
                    let process_to_open = ProcessInfo {
                        pid: Pid::from_u32(process_entry.process_id as u32),
                        name: String::new(),
                        is_windowed: false,
                        icon: None,
                    };
                    match ProcessQuery::open_process(&process_to_open) {
                        Ok(opened_process) => {
                            if let Ok(mut session_manager) = SessionManager::get_instance().write() {
                                session_manager.set_opened_process(opened_process);
                            } else {
                                Logger::get_instance().log(LogLevel::Warn, "Failed to open process.", None);
                            }
                        }
                        Err(err) => {
                            Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_to_open.pid, err), None);
                        }
                    }
                });
            });
    }
}
