use sysinfo::Pid;

pub struct ProcessQueryOptions {
    pub required_process_id: Option<Pid>,
    pub search_name: Option<String>,
    pub require_windowed: bool,
    pub match_case: bool,
    pub fetch_icons: bool,
    pub limit: Option<u64>,
}
