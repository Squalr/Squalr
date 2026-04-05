#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DolphinMemoryRegionKind {
    GameCubeMainMemory,
    WiiExtendedMemory,
}

impl DolphinMemoryRegionKind {
    pub fn module_name(&self) -> &'static str {
        match self {
            Self::GameCubeMainMemory => "GC",
            Self::WiiExtendedMemory => "Wii",
        }
    }

    pub fn guest_base_address(&self) -> u64 {
        match self {
            Self::GameCubeMainMemory => 0x8000_0000,
            Self::WiiExtendedMemory => 0x9000_0000,
        }
    }

    pub fn region_size(&self) -> u64 {
        match self {
            Self::GameCubeMainMemory => 0x0200_0000,
            Self::WiiExtendedMemory => 0x0400_0000,
        }
    }

    pub fn host_region_size(&self) -> u64 {
        self.region_size()
    }

    pub fn contains_guest_address(
        &self,
        guest_address: u64,
    ) -> bool {
        let guest_base_address = self.guest_base_address();
        let guest_end_address = guest_base_address.saturating_add(self.region_size());

        guest_address >= guest_base_address && guest_address < guest_end_address
    }

    pub fn guest_offset(
        &self,
        guest_address: u64,
    ) -> Option<u64> {
        self.contains_guest_address(guest_address)
            .then(|| guest_address.saturating_sub(self.guest_base_address()))
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

    pub fn get_module_name(&self) -> &'static str {
        self.region_kind.module_name()
    }

    pub fn get_guest_base_address(&self) -> u64 {
        self.region_kind.guest_base_address()
    }

    pub fn get_region_size(&self) -> u64 {
        self.region_kind.region_size()
    }

    pub fn get_host_base_address(&self) -> u64 {
        self.host_base_address
    }

    pub fn contains_guest_address(
        &self,
        guest_address: u64,
    ) -> bool {
        self.region_kind.contains_guest_address(guest_address)
    }

    pub fn guest_to_host_address(
        &self,
        guest_address: u64,
    ) -> Option<u64> {
        self.region_kind
            .guest_offset(guest_address)
            .map(|guest_offset| self.host_base_address.saturating_add(guest_offset))
    }

    pub fn host_to_guest_address(
        &self,
        host_address: u64,
    ) -> Option<u64> {
        let host_end_address = self.host_base_address.saturating_add(self.get_region_size());

        (host_address >= self.host_base_address && host_address < host_end_address).then(|| {
            self.get_guest_base_address()
                .saturating_add(host_address.saturating_sub(self.host_base_address))
        })
    }
}

pub fn find_dolphin_region_by_guest_address(
    region_descriptors: &[DolphinMemoryRegionDescriptor],
    guest_address: u64,
) -> Option<DolphinMemoryRegionDescriptor> {
    region_descriptors
        .iter()
        .copied()
        .find(|region_descriptor| region_descriptor.contains_guest_address(guest_address))
}

pub fn find_dolphin_region_by_module_name(
    region_descriptors: &[DolphinMemoryRegionDescriptor],
    module_name: &str,
) -> Option<DolphinMemoryRegionDescriptor> {
    region_descriptors.iter().copied().find(|region_descriptor| {
        region_descriptor
            .get_module_name()
            .eq_ignore_ascii_case(module_name)
    })
}
