use crate::MainWindowView;
use crate::MemorySettingsViewModelBindings;
use crate::mvvm::view_binding::ViewBinding;
use slint::ComponentHandle;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine_memory::memory_settings::MemorySettings;

pub struct MemorySettingsViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl MemorySettingsViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = MemorySettingsViewModel {
            view_binding: view_binding.clone(),
        };

        create_view_bindings!(
            view_binding,
            {
                MemorySettingsViewModelBindings => {
                    on_required_write_changed(value: bool) -> |value| { MemorySettings::get_instance().set_required_write(value); },
                    on_required_execute_changed(value: bool) -> |value| { MemorySettings::get_instance().set_required_execute(value); },
                    on_required_copy_on_write_changed(value: bool) -> |value| { MemorySettings::get_instance().set_required_copy_on_write(value); },
                    on_excluded_write_changed(value: bool) -> |value| { MemorySettings::get_instance().set_excluded_write(value); },
                    on_excluded_execute_changed(value: bool) -> |value| { MemorySettings::get_instance().set_excluded_execute(value); },
                    on_excluded_copy_on_write_changed(value: bool) -> |value| { MemorySettings::get_instance().set_excluded_copy_on_write(value); },
                    on_memory_type_none_changed(value: bool) -> |value| { MemorySettings::get_instance().set_memory_type_none(value); },
                    on_memory_type_image_changed(value: bool) -> |value| { MemorySettings::get_instance().set_memory_type_image(value); },
                    on_memory_type_private_changed(value: bool) -> |value| { MemorySettings::get_instance().set_memory_type_private(value); },
                    on_memory_type_mapped_changed(value: bool) -> |value| { MemorySettings::get_instance().set_memory_type_mapped(value); },
                }
            }
        );

        view.sync_ui_with_memory_settings();

        view
    }

    fn sync_ui_with_memory_settings(&self) {
        self.view_binding.execute_on_ui_thread(|main_window_view, _| {
            let memory_settings_view = main_window_view.global::<MemorySettingsViewModelBindings>();

            // Required
            memory_settings_view.set_required_write(MemorySettings::get_instance().get_required_write());
            memory_settings_view.set_required_execute(MemorySettings::get_instance().get_required_execute());
            memory_settings_view.set_required_copy_on_write(MemorySettings::get_instance().get_required_copy_on_write());

            // Excluded
            memory_settings_view.set_excluded_write(MemorySettings::get_instance().get_excluded_write());
            memory_settings_view.set_excluded_execute(MemorySettings::get_instance().get_excluded_execute());
            memory_settings_view.set_excluded_copy_on_write(MemorySettings::get_instance().get_excluded_copy_on_write());

            // Memory types
            memory_settings_view.set_memory_type_none(MemorySettings::get_instance().get_memory_type_none());
            memory_settings_view.set_memory_type_image(MemorySettings::get_instance().get_memory_type_image());
            memory_settings_view.set_memory_type_private(MemorySettings::get_instance().get_memory_type_private());
            memory_settings_view.set_memory_type_mapped(MemorySettings::get_instance().get_memory_type_mapped());

            let _implement_me_query_ranges = 5;
        });
    }
}
