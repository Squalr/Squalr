use crate::os::engine_os_provider::EngineOsProviders;
use squalr_engine_api::structures::{
    memory::normalized_region::NormalizedRegion,
    patches::{PatchDescriptor, PatchKind},
    processes::opened_process_info::OpenedProcessInfo,
};
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PatchProcessKey {
    process_id: u32,
    process_handle: u64,
}

impl PatchProcessKey {
    fn from_opened_process(opened_process_info: &OpenedProcessInfo) -> Self {
        Self {
            process_id: opened_process_info.get_process_id_raw(),
            process_handle: opened_process_info.get_handle(),
        }
    }
}

#[derive(Default)]
struct PatchServiceState {
    process_key: Option<PatchProcessKey>,
    patches_by_id: HashMap<String, PatchDescriptor>,
}

pub struct PatchService {
    next_patch_id: AtomicU64,
    state: Mutex<PatchServiceState>,
}

impl PatchService {
    pub fn new() -> Self {
        Self {
            next_patch_id: AtomicU64::new(1),
            state: Mutex::new(PatchServiceState::default()),
        }
    }

    pub fn apply_patch(
        &self,
        opened_process_info: &OpenedProcessInfo,
        os_providers: &EngineOsProviders,
        address: u64,
        module_name: &str,
        patched_bytes: &[u8],
        kind: PatchKind,
        label: Option<String>,
    ) -> Result<PatchDescriptor, String> {
        if patched_bytes.is_empty() {
            return Err(String::from("Patch bytes cannot be empty."));
        }

        let absolute_address = Self::resolve_absolute_address(opened_process_info, os_providers, address, module_name)?;
        let patch_region = NormalizedRegion::new(absolute_address, patched_bytes.len() as u64);
        let mut state = self.lock_state()?;

        Self::ensure_process_scope(&mut state, opened_process_info);

        if let Some(conflicting_patch) = Self::find_active_overlap_locked(&state, &patch_region) {
            return Err(format!(
                "Patch range 0x{:X}-0x{:X} overlaps active patch '{}'.",
                patch_region.get_base_address(),
                patch_region.get_end_address(),
                conflicting_patch.get_patch_id()
            ));
        }

        let mut original_bytes = vec![0_u8; patched_bytes.len()];
        if !os_providers
            .memory_read
            .read_bytes(opened_process_info, absolute_address, &mut original_bytes)
        {
            return Err(format!("Failed to read original bytes at 0x{:X}.", absolute_address));
        }

        if !os_providers
            .memory_write
            .write_bytes(opened_process_info, absolute_address, patched_bytes)
        {
            return Err(format!("Failed to write patch bytes at 0x{:X}.", absolute_address));
        }

        let patch_id = self.allocate_patch_id();
        let patch_descriptor = PatchDescriptor::new(
            patch_id.clone(),
            module_name.to_string(),
            patch_region,
            original_bytes,
            patched_bytes.to_vec(),
            kind,
            label,
            true,
        );

        state.patches_by_id.insert(patch_id, patch_descriptor.clone());

        Ok(patch_descriptor)
    }

    pub fn restore_patch(
        &self,
        opened_process_info: &OpenedProcessInfo,
        os_providers: &EngineOsProviders,
        patch_id: &str,
    ) -> Result<PatchDescriptor, String> {
        let mut state = self.lock_state()?;

        Self::ensure_process_scope(&mut state, opened_process_info);
        Self::restore_patch_by_id_locked(&mut state, opened_process_info, os_providers, patch_id)
    }

    pub fn restore_patch_at_address(
        &self,
        opened_process_info: &OpenedProcessInfo,
        os_providers: &EngineOsProviders,
        address: u64,
        module_name: &str,
        expected_kind: Option<PatchKind>,
    ) -> Result<PatchDescriptor, String> {
        let absolute_address = Self::resolve_absolute_address(opened_process_info, os_providers, address, module_name)?;
        let mut state = self.lock_state()?;

        Self::ensure_process_scope(&mut state, opened_process_info);

        let Some(active_patch_match) = state
            .patches_by_id
            .values()
            .find(|patch_descriptor| {
                patch_descriptor.get_is_active() && Self::region_contains_address_half_open(patch_descriptor.get_region(), absolute_address)
            })
            .map(|patch_descriptor| (patch_descriptor.get_patch_id().to_string(), patch_descriptor.get_kind()))
        else {
            return Err(format!("No active patch contains address 0x{:X}.", absolute_address));
        };
        let (active_patch_id, active_patch_kind) = active_patch_match;

        if let Some(expected_kind) = expected_kind {
            if active_patch_kind != expected_kind {
                return Err(format!(
                    "Active patch '{}' at 0x{:X} has kind {:?}, not {:?}.",
                    active_patch_id, absolute_address, active_patch_kind, expected_kind
                ));
            }
        }

        Self::restore_patch_by_id_locked(&mut state, opened_process_info, os_providers, &active_patch_id)
    }

    pub fn list_patches(
        &self,
        opened_process_info: &OpenedProcessInfo,
    ) -> Result<Vec<PatchDescriptor>, String> {
        let mut state = self.lock_state()?;

        Self::ensure_process_scope(&mut state, opened_process_info);

        let mut patches = state.patches_by_id.values().cloned().collect::<Vec<_>>();
        patches.sort_by(|left_patch, right_patch| {
            left_patch
                .get_region()
                .get_base_address()
                .cmp(&right_patch.get_region().get_base_address())
                .then_with(|| left_patch.get_patch_id().cmp(right_patch.get_patch_id()))
        });

        Ok(patches)
    }

    pub fn find_active_overlap(
        &self,
        opened_process_info: &OpenedProcessInfo,
        region: &NormalizedRegion,
    ) -> Result<Option<PatchDescriptor>, String> {
        let mut state = self.lock_state()?;

        Self::ensure_process_scope(&mut state, opened_process_info);

        Ok(Self::find_active_overlap_locked(&state, region))
    }

    pub fn clear(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.process_key = None;
            state.patches_by_id.clear();
        }
    }

    pub fn clear_if_process_changed(
        &self,
        opened_process_info: Option<&OpenedProcessInfo>,
    ) {
        let next_process_key = opened_process_info.map(PatchProcessKey::from_opened_process);

        if let Ok(mut state) = self.state.lock() {
            if state.process_key != next_process_key {
                state.process_key = next_process_key;
                state.patches_by_id.clear();
            }
        }
    }

    fn restore_patch_by_id_locked(
        state: &mut PatchServiceState,
        opened_process_info: &OpenedProcessInfo,
        os_providers: &EngineOsProviders,
        patch_id: &str,
    ) -> Result<PatchDescriptor, String> {
        let Some(patch_descriptor) = state.patches_by_id.get_mut(patch_id) else {
            return Err(format!("Patch '{}' does not exist.", patch_id));
        };

        if !patch_descriptor.get_is_active() {
            return Err(format!("Patch '{}' is already inactive.", patch_id));
        }

        let patch_address = patch_descriptor.get_region().get_base_address();
        let mut current_bytes = vec![0_u8; patch_descriptor.get_patched_bytes().len()];
        if !os_providers
            .memory_read
            .read_bytes(opened_process_info, patch_address, &mut current_bytes)
        {
            return Err(format!("Failed to read patched bytes at 0x{:X}.", patch_address));
        }

        if current_bytes != patch_descriptor.get_patched_bytes() {
            return Err(format!(
                "Patch '{}' cannot be restored because target bytes no longer match the recorded patch bytes.",
                patch_id
            ));
        }

        if !os_providers
            .memory_write
            .write_bytes(opened_process_info, patch_address, patch_descriptor.get_original_bytes())
        {
            return Err(format!("Failed to restore original bytes at 0x{:X}.", patch_address));
        }

        patch_descriptor.set_is_active(false);

        Ok(patch_descriptor.clone())
    }

    fn find_active_overlap_locked(
        state: &PatchServiceState,
        region: &NormalizedRegion,
    ) -> Option<PatchDescriptor> {
        state
            .patches_by_id
            .values()
            .find(|patch_descriptor| patch_descriptor.get_is_active() && Self::regions_overlap_half_open(patch_descriptor.get_region(), region))
            .cloned()
    }

    fn regions_overlap_half_open(
        left_region: &NormalizedRegion,
        right_region: &NormalizedRegion,
    ) -> bool {
        let left_start_address = left_region.get_base_address();
        let left_end_address = left_region.get_end_address();
        let right_start_address = right_region.get_base_address();
        let right_end_address = right_region.get_end_address();

        left_start_address < right_end_address && right_start_address < left_end_address
    }

    fn region_contains_address_half_open(
        region: &NormalizedRegion,
        address: u64,
    ) -> bool {
        address >= region.get_base_address() && address < region.get_end_address()
    }

    fn resolve_absolute_address(
        opened_process_info: &OpenedProcessInfo,
        os_providers: &EngineOsProviders,
        address: u64,
        module_name: &str,
    ) -> Result<u64, String> {
        if module_name.trim().is_empty() {
            return Ok(address);
        }

        let modules = os_providers.memory_query.get_modules(opened_process_info);
        let Some(module) = modules
            .iter()
            .find(|module| module.get_module_name().eq_ignore_ascii_case(module_name))
        else {
            return Err(format!("Module '{}' is not loaded in the opened process.", module_name));
        };

        module
            .get_base_address()
            .checked_add(address)
            .ok_or_else(|| format!("Module-relative address {}+0x{:X} overflowed.", module_name, address))
    }

    fn ensure_process_scope(
        state: &mut PatchServiceState,
        opened_process_info: &OpenedProcessInfo,
    ) {
        let process_key = PatchProcessKey::from_opened_process(opened_process_info);

        if state.process_key != Some(process_key) {
            state.process_key = Some(process_key);
            state.patches_by_id.clear();
        }
    }

    fn allocate_patch_id(&self) -> String {
        let patch_number = self.next_patch_id.fetch_add(1, Ordering::SeqCst);

        format!("patch-{}", patch_number)
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, PatchServiceState>, String> {
        self.state
            .lock()
            .map_err(|error| format!("Failed to lock patch service state: {}.", error))
    }
}

impl Default for PatchService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PatchService;
    use crate::os::engine_os_provider::EngineOsProviders;
    use squalr_engine_api::structures::{
        data_values::data_value::DataValue,
        memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        patches::PatchKind,
        processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
        structs::valued_struct::ValuedStruct,
    };
    use squalr_engine_targets::{
        MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, PageRetrievalMode, ProcessQueryError, ProcessQueryOptions, ProcessQueryProvider,
    };
    use std::sync::{Arc, Mutex};

    struct TestProcessQueryProvider;

    impl ProcessQueryProvider for TestProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Ok(())
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            Vec::new()
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not used in patch service tests"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    struct TestMemoryQueryProvider;

    impl MemoryQueryProvider for TestMemoryQueryProvider {
        fn get_modules(
            &self,
            _process_info: &OpenedProcessInfo,
        ) -> Vec<NormalizedModule> {
            vec![NormalizedModule::new("game.exe", 0x1000, 0x200)]
        }

        fn address_to_module(
            &self,
            _address: u64,
            _modules: &Vec<NormalizedModule>,
        ) -> Option<(String, u64)> {
            None
        }

        fn resolve_module(
            &self,
            modules: &Vec<NormalizedModule>,
            identifier: &str,
        ) -> u64 {
            modules
                .iter()
                .find(|module| module.get_module_name().eq_ignore_ascii_case(identifier))
                .map(|module| module.get_base_address())
                .unwrap_or(0)
        }

        fn get_memory_page_bounds(
            &self,
            _process_info: &OpenedProcessInfo,
            _page_retrieval_mode: PageRetrievalMode,
        ) -> Vec<NormalizedRegion> {
            vec![NormalizedRegion::new(0x1000, 0x200)]
        }
    }

    #[derive(Clone)]
    struct TestMemory {
        bytes: Arc<Mutex<Vec<u8>>>,
        base_address: u64,
    }

    impl TestMemory {
        fn new(bytes: Vec<u8>) -> Self {
            Self {
                bytes: Arc::new(Mutex::new(bytes)),
                base_address: 0x1000,
            }
        }

        fn read_slice(
            &self,
            address: u64,
            values: &mut [u8],
        ) -> bool {
            let Ok(bytes) = self.bytes.lock() else {
                return false;
            };
            let Some(offset) = address
                .checked_sub(self.base_address)
                .and_then(|offset| usize::try_from(offset).ok())
            else {
                return false;
            };
            let end_offset = offset.saturating_add(values.len());

            if end_offset > bytes.len() {
                return false;
            }

            values.copy_from_slice(&bytes[offset..end_offset]);

            true
        }

        fn write_slice(
            &self,
            address: u64,
            values: &[u8],
        ) -> bool {
            let Ok(mut bytes) = self.bytes.lock() else {
                return false;
            };
            let Some(offset) = address
                .checked_sub(self.base_address)
                .and_then(|offset| usize::try_from(offset).ok())
            else {
                return false;
            };
            let end_offset = offset.saturating_add(values.len());

            if end_offset > bytes.len() {
                return false;
            }

            bytes[offset..end_offset].copy_from_slice(values);

            true
        }

        fn bytes(&self) -> Vec<u8> {
            self.bytes.lock().map(|bytes| bytes.clone()).unwrap_or_default()
        }
    }

    impl MemoryReadProvider for TestMemory {
        fn read(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            data_value: &mut DataValue,
        ) -> bool {
            let mut bytes = vec![0_u8; data_value.get_value_bytes().len()];
            if !self.read_slice(address, &mut bytes) {
                return false;
            }

            data_value.copy_from_bytes(&bytes);

            true
        }

        fn read_struct(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            valued_struct: &mut ValuedStruct,
        ) -> bool {
            let mut bytes = vec![0_u8; valued_struct.get_size_in_bytes() as usize];
            if !self.read_slice(address, &mut bytes) {
                return false;
            }

            valued_struct.copy_from_bytes(&bytes)
        }

        fn read_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            values: &mut [u8],
        ) -> bool {
            self.read_slice(address, values)
        }
    }

    impl MemoryWriteProvider for TestMemory {
        fn write_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            values: &[u8],
        ) -> bool {
            self.write_slice(address, values)
        }
    }

    fn create_test_context() -> (PatchService, OpenedProcessInfo, EngineOsProviders, TestMemory) {
        let test_memory = TestMemory::new(vec![0x90, 0x40, 0x41, 0x42, 0x43, 0x44]);
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider),
            Arc::new(test_memory.clone()),
            Arc::new(test_memory.clone()),
        );
        let opened_process_info = OpenedProcessInfo::new(123, String::from("game.exe"), 456, Bitness::Bit64, None);

        (PatchService::new(), opened_process_info, os_providers, test_memory)
    }

    #[test]
    fn apply_patch_records_original_bytes_and_writes_patch_bytes() {
        let (patch_service, opened_process_info, os_providers, test_memory) = create_test_context();

        let patch_descriptor = patch_service
            .apply_patch(
                &opened_process_info,
                &os_providers,
                0x1001,
                "",
                &[0x90, 0x90],
                PatchKind::Code,
                Some(String::from("nop")),
            )
            .expect("Expected patch application to succeed.");

        assert_eq!(patch_descriptor.get_original_bytes(), &[0x40, 0x41]);
        assert_eq!(patch_descriptor.get_patched_bytes(), &[0x90, 0x90]);
        assert_eq!(test_memory.bytes(), vec![0x90, 0x90, 0x90, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn apply_patch_rejects_overlapping_active_patch() {
        let (patch_service, opened_process_info, os_providers, _test_memory) = create_test_context();

        patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected first patch application to succeed.");
        let overlapping_patch_result = patch_service.apply_patch(&opened_process_info, &os_providers, 0x1002, "", &[0xCC], PatchKind::SoftwareBreakpoint, None);

        assert!(overlapping_patch_result.is_err());
    }

    #[test]
    fn apply_patch_rejects_no_operation_overlapping_software_breakpoint_patch() {
        let (patch_service, opened_process_info, os_providers, _test_memory) = create_test_context();

        patch_service
            .apply_patch(
                &opened_process_info,
                &os_providers,
                0x1001,
                "",
                &[0xCC],
                PatchKind::SoftwareBreakpoint,
                Some(String::from("breakpoint")),
            )
            .expect("Expected software breakpoint patch application to succeed.");
        let no_operation_patch_result = patch_service.apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90], PatchKind::NoOperation, None);

        assert!(no_operation_patch_result.is_err());
    }

    #[test]
    fn apply_patch_allows_adjacent_half_open_regions() {
        let (patch_service, opened_process_info, os_providers, _test_memory) = create_test_context();

        patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected first patch application to succeed.");
        let adjacent_patch_result = patch_service.apply_patch(&opened_process_info, &os_providers, 0x1003, "", &[0xCC], PatchKind::SoftwareBreakpoint, None);

        assert!(adjacent_patch_result.is_ok());
    }

    #[test]
    fn restore_patch_writes_original_bytes_and_marks_patch_inactive() {
        let (patch_service, opened_process_info, os_providers, test_memory) = create_test_context();
        let patch_descriptor = patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected patch application to succeed.");

        let restored_patch = patch_service
            .restore_patch(&opened_process_info, &os_providers, patch_descriptor.get_patch_id())
            .expect("Expected patch restore to succeed.");

        assert!(!restored_patch.get_is_active());
        assert_eq!(test_memory.bytes(), vec![0x90, 0x40, 0x41, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn restore_patch_rejects_external_mutation() {
        let (patch_service, opened_process_info, os_providers, test_memory) = create_test_context();
        let patch_descriptor = patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected patch application to succeed.");
        assert!(test_memory.write_slice(0x1001, &[0xCC, 0x90]));

        let restore_result = patch_service.restore_patch(&opened_process_info, &os_providers, patch_descriptor.get_patch_id());

        assert!(restore_result.is_err());
        assert_eq!(test_memory.bytes(), vec![0x90, 0xCC, 0x90, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn restore_patch_at_address_restores_containing_patch() {
        let (patch_service, opened_process_info, os_providers, test_memory) = create_test_context();
        patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1, "game.exe", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected module-relative patch application to succeed.");

        let restored_patch = patch_service
            .restore_patch_at_address(&opened_process_info, &os_providers, 0x2, "game.exe", None)
            .expect("Expected address-based patch restore to succeed.");

        assert!(!restored_patch.get_is_active());
        assert_eq!(test_memory.bytes(), vec![0x90, 0x40, 0x41, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn restore_patch_at_address_rejects_unexpected_patch_kind() {
        let (patch_service, opened_process_info, os_providers, test_memory) = create_test_context();
        patch_service
            .apply_patch(&opened_process_info, &os_providers, 0x1001, "", &[0x90, 0x90], PatchKind::Code, None)
            .expect("Expected patch application to succeed.");

        let restore_result = patch_service.restore_patch_at_address(&opened_process_info, &os_providers, 0x1001, "", Some(PatchKind::NoOperation));

        assert!(restore_result.is_err());
        assert_eq!(test_memory.bytes(), vec![0x90, 0x90, 0x90, 0x42, 0x43, 0x44]);
    }
}
