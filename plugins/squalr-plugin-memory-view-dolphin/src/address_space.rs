const GC_WII_MODULE_NAME: &str = "gc_wii";
const GBA_WORK_RAM_MODULE_PREFIX: &str = "gba_wm_";
const GBA_INTERNAL_RAM_MODULE_PREFIX: &str = "gba_im_";

const GAMECUBE_MAIN_MEMORY_BASE_ADDRESS: u64 = 0x8000_0000;
const WII_EXTENDED_MEMORY_BASE_ADDRESS: u64 = 0x9000_0000;
const GAMECUBE_MAIN_MEMORY_SIZE: u64 = 0x0200_0000;
const WII_EXTENDED_MEMORY_SIZE: u64 = 0x0400_0000;
const GBA_WORK_RAM_BASE_ADDRESS: u64 = 0xA000_0000;
const GBA_INTERNAL_RAM_BASE_ADDRESS: u64 = 0xA004_0000;
const GBA_SLOT_STRIDE: u64 = 0x0010_0000;
const GBA_WORK_RAM_SIZE: u64 = 0x0004_0000;
const GBA_INTERNAL_RAM_SIZE: u64 = 0x0000_8000;
const MAX_GBA_SLOT_NUMBER: u8 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DolphinMemoryRegionKind {
    GameCubeMainMemory,
    WiiExtendedMemory,
    GameBoyAdvanceWorkRam(u8),
    GameBoyAdvanceInternalRam(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DolphinPointerScanDomain {
    GcWii,
    Gba(u8),
}

impl DolphinMemoryRegionKind {
    pub fn get_virtual_base_address(&self) -> u64 {
        match self {
            Self::GameCubeMainMemory => GAMECUBE_MAIN_MEMORY_BASE_ADDRESS,
            Self::WiiExtendedMemory => WII_EXTENDED_MEMORY_BASE_ADDRESS,
            Self::GameBoyAdvanceWorkRam(slot_number) => GBA_WORK_RAM_BASE_ADDRESS + gba_slot_ordinal(*slot_number).unwrap_or(0) * GBA_SLOT_STRIDE,
            Self::GameBoyAdvanceInternalRam(slot_number) => GBA_INTERNAL_RAM_BASE_ADDRESS + gba_slot_ordinal(*slot_number).unwrap_or(0) * GBA_SLOT_STRIDE,
        }
    }

    pub fn get_region_size(&self) -> u64 {
        match self {
            Self::GameCubeMainMemory => GAMECUBE_MAIN_MEMORY_SIZE,
            Self::WiiExtendedMemory => WII_EXTENDED_MEMORY_SIZE,
            Self::GameBoyAdvanceWorkRam(_) => GBA_WORK_RAM_SIZE,
            Self::GameBoyAdvanceInternalRam(_) => GBA_INTERNAL_RAM_SIZE,
        }
    }

    pub fn host_region_size(&self) -> u64 {
        self.get_region_size()
    }

    pub fn get_module_name(&self) -> String {
        match self {
            Self::GameCubeMainMemory | Self::WiiExtendedMemory => GC_WII_MODULE_NAME.to_string(),
            Self::GameBoyAdvanceWorkRam(slot_number) => format!("{}{}", GBA_WORK_RAM_MODULE_PREFIX, slot_number),
            Self::GameBoyAdvanceInternalRam(slot_number) => format!("{}{}", GBA_INTERNAL_RAM_MODULE_PREFIX, slot_number),
        }
    }

    pub fn get_pointer_scan_domain(&self) -> DolphinPointerScanDomain {
        match self {
            Self::GameCubeMainMemory | Self::WiiExtendedMemory => DolphinPointerScanDomain::GcWii,
            Self::GameBoyAdvanceWorkRam(slot_index) | Self::GameBoyAdvanceInternalRam(slot_index) => DolphinPointerScanDomain::Gba(*slot_index),
        }
    }

    pub fn contains_virtual_address(
        &self,
        virtual_address: u64,
    ) -> bool {
        let virtual_base_address = self.get_virtual_base_address();
        let virtual_end_address = virtual_base_address.saturating_add(self.get_region_size());

        virtual_address >= virtual_base_address && virtual_address < virtual_end_address
    }

    pub fn virtual_offset(
        &self,
        virtual_address: u64,
    ) -> Option<u64> {
        self.contains_virtual_address(virtual_address)
            .then(|| virtual_address.saturating_sub(self.get_virtual_base_address()))
    }

    pub fn module_offset(
        &self,
        virtual_address: u64,
    ) -> Option<u64> {
        let virtual_offset = self.virtual_offset(virtual_address)?;

        match self {
            Self::GameCubeMainMemory => Some(virtual_offset),
            Self::WiiExtendedMemory => Some(GAMECUBE_MAIN_MEMORY_SIZE.saturating_add(virtual_offset)),
            Self::GameBoyAdvanceWorkRam(_) | Self::GameBoyAdvanceInternalRam(_) => Some(virtual_offset),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DolphinMemoryRegionDescriptor {
    region_kind: DolphinMemoryRegionKind,
    host_base_address: u64,
}

impl DolphinMemoryRegionDescriptor {
    pub fn new(
        region_kind: DolphinMemoryRegionKind,
        host_base_address: u64,
    ) -> Self {
        Self {
            region_kind,
            host_base_address,
        }
    }

    pub fn get_region_kind(&self) -> DolphinMemoryRegionKind {
        self.region_kind
    }

    pub fn get_module_name(&self) -> String {
        self.region_kind.get_module_name()
    }

    pub fn get_virtual_base_address(&self) -> u64 {
        self.region_kind.get_virtual_base_address()
    }

    pub fn get_region_size(&self) -> u64 {
        self.region_kind.get_region_size()
    }

    pub fn get_host_base_address(&self) -> u64 {
        self.host_base_address
    }

    pub fn get_pointer_scan_domain(&self) -> DolphinPointerScanDomain {
        self.region_kind.get_pointer_scan_domain()
    }

    pub fn contains_virtual_address(
        &self,
        virtual_address: u64,
    ) -> bool {
        self.region_kind.contains_virtual_address(virtual_address)
    }

    pub fn virtual_to_host_address(
        &self,
        virtual_address: u64,
    ) -> Option<u64> {
        self.region_kind
            .virtual_offset(virtual_address)
            .map(|virtual_offset| self.host_base_address.saturating_add(virtual_offset))
    }

    pub fn host_to_virtual_address(
        &self,
        host_address: u64,
    ) -> Option<u64> {
        let host_end_address = self.host_base_address.saturating_add(self.get_region_size());

        (host_address >= self.host_base_address && host_address < host_end_address).then(|| {
            self.get_virtual_base_address()
                .saturating_add(host_address.saturating_sub(self.host_base_address))
        })
    }

    pub fn module_relative_address(
        &self,
        virtual_address: u64,
    ) -> Option<(String, u64)> {
        self.region_kind
            .module_offset(virtual_address)
            .map(|module_offset| (self.get_module_name(), module_offset))
    }
}

pub fn gc_wii_module_name() -> &'static str {
    GC_WII_MODULE_NAME
}

pub fn parse_gba_work_ram_slot(module_name: &str) -> Option<u8> {
    parse_gba_slot(module_name, GBA_WORK_RAM_MODULE_PREFIX)
}

pub fn parse_gba_internal_ram_slot(module_name: &str) -> Option<u8> {
    parse_gba_slot(module_name, GBA_INTERNAL_RAM_MODULE_PREFIX)
}

pub fn find_dolphin_region_by_virtual_address(
    region_descriptors: &[DolphinMemoryRegionDescriptor],
    virtual_address: u64,
) -> Option<DolphinMemoryRegionDescriptor> {
    region_descriptors
        .iter()
        .copied()
        .find(|region_descriptor| region_descriptor.contains_virtual_address(virtual_address))
}

pub fn find_dolphin_region_by_module_name(
    region_descriptors: &[DolphinMemoryRegionDescriptor],
    module_name: &str,
) -> Option<DolphinMemoryRegionDescriptor> {
    let normalized_module_name = module_name.to_ascii_lowercase();

    region_descriptors.iter().copied().find(|region_descriptor| {
        let candidate_module_name = region_descriptor.get_module_name();

        candidate_module_name.eq_ignore_ascii_case(&normalized_module_name)
            || (normalized_module_name == "gc" || normalized_module_name == "mem1")
                && matches!(region_descriptor.get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory)
            || (normalized_module_name == "wii" || normalized_module_name == "mem2")
                && matches!(region_descriptor.get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory)
    })
}

pub fn resolve_virtual_address_from_module(
    module_name: &str,
    module_offset: u64,
) -> Option<u64> {
    if module_name.eq_ignore_ascii_case(GC_WII_MODULE_NAME) {
        if module_offset < GAMECUBE_MAIN_MEMORY_SIZE {
            return Some(GAMECUBE_MAIN_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
        }

        let wii_offset = module_offset.saturating_sub(GAMECUBE_MAIN_MEMORY_SIZE);

        if wii_offset < WII_EXTENDED_MEMORY_SIZE {
            return Some(WII_EXTENDED_MEMORY_BASE_ADDRESS.saturating_add(wii_offset));
        }

        return None;
    }

    if module_name.eq_ignore_ascii_case("gc") || module_name.eq_ignore_ascii_case("mem1") {
        return (module_offset < GAMECUBE_MAIN_MEMORY_SIZE).then(|| GAMECUBE_MAIN_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
    }

    if module_name.eq_ignore_ascii_case("wii") || module_name.eq_ignore_ascii_case("mem2") {
        return (module_offset < WII_EXTENDED_MEMORY_SIZE).then(|| WII_EXTENDED_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
    }

    if let Some(slot_index) = parse_gba_work_ram_slot(module_name) {
        return (module_offset < GBA_WORK_RAM_SIZE).then(|| {
            GBA_WORK_RAM_BASE_ADDRESS
                .saturating_add(gba_slot_ordinal(slot_index).unwrap_or(0) * GBA_SLOT_STRIDE)
                .saturating_add(module_offset)
        });
    }

    if let Some(slot_index) = parse_gba_internal_ram_slot(module_name) {
        return (module_offset < GBA_INTERNAL_RAM_SIZE).then(|| {
            GBA_INTERNAL_RAM_BASE_ADDRESS
                .saturating_add(gba_slot_ordinal(slot_index).unwrap_or(0) * GBA_SLOT_STRIDE)
                .saturating_add(module_offset)
        });
    }

    None
}

fn parse_gba_slot(
    module_name: &str,
    module_prefix: &str,
) -> Option<u8> {
    let normalized_module_name = module_name.to_ascii_lowercase();
    let slot_number = normalized_module_name
        .strip_prefix(module_prefix)?
        .parse::<u8>()
        .ok()?;

    (1..=MAX_GBA_SLOT_NUMBER)
        .contains(&slot_number)
        .then_some(slot_number)
}

fn gba_slot_ordinal(slot_number: u8) -> Option<u64> {
    (1..=MAX_GBA_SLOT_NUMBER)
        .contains(&slot_number)
        .then_some(slot_number as u64 - 1)
}
