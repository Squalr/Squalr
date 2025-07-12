#[derive(Clone, Debug)]
pub struct AndroidProcessInfo {
    pub process_id: u32,
    pub parent_process_id: u32,
    pub package_name: String,
}
