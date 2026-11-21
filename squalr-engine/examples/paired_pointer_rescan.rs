//! Capture native pointer graphs across a relaunch and compare a module-relative root.
//! Run without arguments for usage. A limit applies separately to each requested root.

use anyhow::{Context, Result, bail, ensure};
use squalr_engine::{
    command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor, engine_mode::EngineMode,
    engine_privileged_state::create_engine_privileged_state,
};
use squalr_engine_api::{
    commands::{pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest, process::open::process_open_request::ProcessOpenRequest},
    structures::{
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        pointer_scans::{
            paired_pointer_walk::{PairedWalkResult, walk_paired_root},
            pointer_scan_address_space::PointerScanAddressSpace,
            pointer_scan_results::PointerScanResults,
            pointer_scan_target_request::PointerScanTargetRequest,
        },
    },
};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use tempfile::NamedTempFile;

fn main() -> Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command, pid, target, pointer_size, radius, depth, file] if command == "capture" => {
            let state = create_engine_privileged_state(EngineMode::Standalone)?;
            let process = ProcessOpenRequest {
                process_id: Some(pid.parse()?),
                search_name: None,
                match_case: false,
            }
            .execute(&state);
            ensure!(process.opened_process_info.is_some(), "could not open PID {pid}");
            let request = PointerScanStartRequest {
                target: PointerScanTargetRequest::address(AnonymousValueString::new(
                    format!("{:X}", hex_address(target)?),
                    AnonymousValueStringFormat::Hexadecimal,
                    ContainerType::None,
                )),
                pointer_size: pointer_size.parse().map_err(anyhow::Error::msg)?,
                offset_radius: radius.parse()?,
                max_depth: depth.parse()?,
                address_space: PointerScanAddressSpace::EmulatorMemory,
            };
            ensure!(request.execute(&state).success, "pointer scan failed");
            let store = state.get_pointer_scan_results();
            let guard = store
                .read()
                .map_err(|_| anyhow::anyhow!("pointer results lock poisoned"))?;
            let graph = guard.as_ref().context("scan has no graph")?;
            let path = Path::new(file);
            let directory = path
                .parent()
                .filter(|dir| !dir.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let mut capture = NamedTempFile::new_in(directory)?;
            {
                let mut writer = BufWriter::new(capture.as_file_mut());
                serde_json::to_writer(&mut writer, graph)?;
                writer.flush()?;
            }
            capture.persist_noclobber(path)?;
        }
        [command, previous_file, current_file, module, root, depth, limit] if command == "compare" => {
            let mut previous = read_graph(previous_file)?;
            let mut current = read_graph(current_file)?;
            let depth: usize = depth.parse()?;
            let limit = limit.parse()?;
            let offsets = if root == "all" {
                ensure!(
                    depth > 0 && depth as u64 <= previous.get_max_depth().min(current.get_max_depth()),
                    "depth must be within both scans"
                );
                let module_index = previous
                    .get_module_index_by_name(module)
                    .context("module missing from baseline")?;
                previous
                    .get_pointer_scan_level_candidates()
                    .get(depth - 1)
                    .into_iter()
                    .flat_map(|level| level.get_static_candidates())
                    .filter(|candidate| candidate.get_module_index() == module_index)
                    .map(|candidate| candidate.get_module_offset())
                    .collect()
            } else {
                vec![hex_address(root)?]
            };
            for module_offset in offsets {
                let result = walk_paired_root(&mut previous, &mut current, module, module_offset, depth, limit)?;
                match result {
                    PairedWalkResult::Matched(offsets) => println!("{module}+{module_offset:#x} matched {offsets:?}"),
                    PairedWalkResult::NoMatch => println!("{module}+{module_offset:#x} no match"),
                    PairedWalkResult::WorkLimit => println!("{module}+{module_offset:#x} unresolved"),
                }
            }
        }
        _ => bail!(
            "usage:\n  capture PID TARGET_HEX POINTER_SIZE RADIUS DEPTH NEW_GRAPH.json\n  compare OLD.json NEW.json MODULE RVA_HEX|all DEPTH WORK_PER_ROOT"
        ),
    }
    Ok(())
}

fn read_graph(file: &str) -> Result<PointerScanResults> {
    serde_json::from_reader(BufReader::new(File::open(file).with_context(|| format!("opening {file}"))?)).with_context(|| format!("reading {file}"))
}

fn hex_address(input: &str) -> Result<u64> {
    Ok(u64::from_str_radix(
        input
            .strip_prefix("0x")
            .or_else(|| input.strip_prefix("0X"))
            .unwrap_or(input),
        16,
    )?)
}
