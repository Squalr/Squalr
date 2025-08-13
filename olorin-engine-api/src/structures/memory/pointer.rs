#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Pointer {
    address: u64,
    offsets: Vec<u8>,
    module_name: String,
}

impl Pointer {
    pub fn new(
        address: u64,
        offsets: Vec<u8>,
        module_name: String,
    ) -> Self {
        Self { address, offsets, module_name }
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn set_address(
        &mut self,
        address: u64,
    ) {
        self.address = address;
    }

    pub fn get_offsets(&self) -> &[u8] {
        &self.offsets
    }

    pub fn set_offsets(
        &mut self,
        offsets: Vec<u8>,
    ) {
        self.offsets = offsets;
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn set_module_name(
        &mut self,
        module_name: String,
    ) {
        self.module_name = module_name;
    }
}
