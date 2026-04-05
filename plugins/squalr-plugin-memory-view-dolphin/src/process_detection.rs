pub fn matches_dolphin_process_name(process_name: &str) -> bool {
    let normalized_process_name = process_name.to_ascii_lowercase();

    normalized_process_name.contains("dolphin") || normalized_process_name.contains("slippi")
}
