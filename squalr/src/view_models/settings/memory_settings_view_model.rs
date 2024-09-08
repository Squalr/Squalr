use crate::view_models::view_model::ViewModel;
use crate::MainWindowView;
use crate::MemorySettingsViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_memory::memory_settings::MemorySettings;
use std::sync::Arc;

pub struct MemorySettingsViewModel {
    view_handle: Arc<MainWindowView>,
}

impl MemorySettingsViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = MemorySettingsViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_bindings();

        return view;
    }
}

impl ViewModel for MemorySettingsViewModel {
    fn create_bindings(&self) {
        let memory_settings_view = self.view_handle.global::<MemorySettingsViewModelBindings>();

        // Required
        memory_settings_view.set_required_write(MemorySettings::get_instance().get_required_write());
        memory_settings_view.on_required_write_changed(|value| {
            MemorySettings::get_instance().set_required_write(value);
        });
        memory_settings_view.set_required_execute(MemorySettings::get_instance().get_required_execute());
        memory_settings_view.on_required_execute_changed(|value| {
            MemorySettings::get_instance().set_required_execute(value);
        });
        memory_settings_view.set_required_copy_on_write(MemorySettings::get_instance().get_required_copy_on_write());
        memory_settings_view.on_required_copy_on_write_changed(|value| {
            MemorySettings::get_instance().set_required_copy_on_write(value);
        });

        // Excluded
        memory_settings_view.set_excluded_write(MemorySettings::get_instance().get_excluded_write());
        memory_settings_view.on_excluded_write_changed(|value| {
            MemorySettings::get_instance().set_excluded_write(value);
        });
        memory_settings_view.set_excluded_execute(MemorySettings::get_instance().get_excluded_execute());
        memory_settings_view.on_excluded_execute_changed(|value| {
            MemorySettings::get_instance().set_excluded_execute(value);
        });
        memory_settings_view.set_excluded_copy_on_write(MemorySettings::get_instance().get_excluded_copy_on_write());
        memory_settings_view.on_excluded_copy_on_write_changed(|value| {
            MemorySettings::get_instance().set_excluded_copy_on_write(value);
        });

        // Memory types
        memory_settings_view.set_memory_type_none(MemorySettings::get_instance().get_memory_type_none());
        memory_settings_view.on_memory_type_none_changed(|value| {
            MemorySettings::get_instance().set_memory_type_none(value);
        });
        memory_settings_view.set_memory_type_image(MemorySettings::get_instance().get_memory_type_image());
        memory_settings_view.on_memory_type_image_changed(|value| {
            MemorySettings::get_instance().set_memory_type_image(value);
        });
        memory_settings_view.set_memory_type_private(MemorySettings::get_instance().get_memory_type_private());
        memory_settings_view.on_memory_type_private_changed(|value| {
            MemorySettings::get_instance().set_memory_type_private(value);
        });
        memory_settings_view.set_memory_type_mapped(MemorySettings::get_instance().get_memory_type_mapped());
        memory_settings_view.on_memory_type_mapped_changed(|value| {
            MemorySettings::get_instance().set_memory_type_mapped(value);
        });

        let implement_me_query_ranges = 5;
    }
}
