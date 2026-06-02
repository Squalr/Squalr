use crate::structures::memory::bitness::Bitness;
use crate::structures::processes::process_icon::ProcessIcon;
use crate::structures::processes::target_architecture::TargetArchitecture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenedProcessInfo {
    process_id: u32,
    name: String,
    handle: u64,
    bitness: Bitness,
    #[serde(default)]
    target_architecture: TargetArchitecture,
    icon: Option<ProcessIcon>,
}

impl OpenedProcessInfo {
    pub fn new(
        process_id: u32,
        name: String,
        handle: u64,
        bitness: Bitness,
        icon: Option<ProcessIcon>,
    ) -> Self {
        Self {
            process_id,
            name,
            handle,
            bitness,
            target_architecture: TargetArchitecture::default_for_bitness(bitness),
            icon,
        }
    }

    pub fn with_target_architecture(
        mut self,
        target_architecture: TargetArchitecture,
    ) -> Self {
        self.target_architecture = target_architecture;
        self.bitness = self.target_architecture.get_pointer_width();

        self
    }

    pub fn get_process_id(&self) -> u32 {
        self.process_id
    }

    pub fn get_process_id_raw(&self) -> u32 {
        self.process_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_handle(&self) -> u64 {
        self.handle
    }

    pub fn get_bitness(&self) -> Bitness {
        self.bitness
    }

    pub fn get_target_architecture(&self) -> &TargetArchitecture {
        &self.target_architecture
    }

    pub fn get_icon(&self) -> &Option<ProcessIcon> {
        &self.icon
    }
}

#[cfg(test)]
mod tests {
    use super::OpenedProcessInfo;
    use crate::structures::memory::bitness::Bitness;
    use crate::structures::processes::target_architecture::TargetArchitecture;

    #[test]
    fn opened_process_info_defaults_target_architecture_from_bitness() {
        let opened_process_info = OpenedProcessInfo::new(1, String::from("target.exe"), 2, Bitness::Bit64, None);

        assert_eq!(
            opened_process_info
                .get_target_architecture()
                .get_instruction_set_id(),
            "x64"
        );
        assert_eq!(
            opened_process_info
                .get_target_architecture()
                .get_instruction_data_type_id(),
            "i_x64"
        );
    }

    #[test]
    fn opened_process_info_allows_explicit_target_architecture_override() {
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("remote-android"), 2, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::arm64());

        assert_eq!(opened_process_info.get_bitness(), Bitness::Bit64);
        assert_eq!(
            opened_process_info
                .get_target_architecture()
                .get_instruction_set_id(),
            "arm64"
        );
        assert_eq!(
            opened_process_info
                .get_target_architecture()
                .get_instruction_data_type_id(),
            "i_arm64"
        );
    }
}
