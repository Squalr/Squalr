use super::*;
use crate::structures::pointer_scans::{
    pointer_scan_address_space::PointerScanAddressSpace, pointer_scan_candidate::PointerScanCandidate, pointer_scan_heap_slot::PointerScanHeapSlot,
    pointer_scan_node_type::PointerScanNodeType, pointer_scan_pointer_size::PointerScanPointerSize,
    pointer_scan_target_descriptor::PointerScanTargetDescriptor,
};

fn graph(
    module_index: usize,
    root_value: u64,
    leaf_slots: &[(u64, u64)],
    target: u64,
) -> PointerScanResults {
    let mut modules = vec!["other".to_string(); module_index + 1];
    modules[module_index] = "game".to_owned();
    let slots = leaf_slots
        .iter()
        .map(|&(pointer_address, pointer_value)| PointerScanHeapSlot {
            pointer_address,
            pointer_value,
        })
        .collect();
    let root = PointerScanCandidate::new(1, 2, PointerScanNodeType::Static, 0x1000, root_value, module_index, 0x28);
    PointerScanResults::new(
        1,
        PointerScanTargetDescriptor::address(target),
        vec![target],
        PointerScanAddressSpace::EmulatorMemory,
        PointerScanPointerSize::Pointer64,
        2,
        32,
        modules,
        vec![],
        vec![
            PointerScanLevelCandidates::with_sorted_heap_slots(1, vec![], 2, slots),
            PointerScanLevelCandidates::new(2, vec![root], vec![]),
        ],
        1,
        leaf_slots.len() as u64,
    )
}

#[test]
fn follows_relocated_root_and_backtracks_to_shared_positive_and_negative_offsets() -> Result<()> {
    let mut first = graph(0, 0x2000, &[(0x1ff8, 0x3008), (0x2020, 0x3004)], 0x3000);
    let mut second = graph(1, 0x6000, &[(0x5ff8, 0x900c), (0x6020, 0x9004)], 0x9000);
    assert_eq!(walk_paired_root(&mut first, &mut second, "game", 0x28, 2, 3)?, PairedWalkResult::WorkLimit);
    assert_eq!(
        walk_paired_root(&mut first, &mut second, "game", 0x28, 2, 64)?,
        PairedWalkResult::Matched(vec![32, -4])
    );
    Ok(())
}

#[test]
fn rejects_swizzled_routes_with_individually_matching_offsets_but_no_common_sequence() -> Result<()> {
    let mut first = graph(0, 0x2000, &[(0x2000, 0x3000), (0x2008, 0x3008)], 0x3000);
    let mut second = graph(0, 0x5000, &[(0x5000, 0x6008), (0x5008, 0x6000)], 0x6000);
    assert_eq!(walk_paired_root(&mut first, &mut second, "game", 0x28, 2, 100)?, PairedWalkResult::NoMatch);
    assert_eq!(walk_paired_root(&mut first, &mut second, "game", 0x40, 2, 100)?, PairedWalkResult::NoMatch);
    Ok(())
}

#[test]
fn skips_overflowed_addresses_and_considers_every_matching_child() -> Result<()> {
    let mut first = graph(0, 32, &[(0, 11), (40, 9)], 9);
    let mut second = graph(0, 0, &[(8, 20), (8, 21)], 21);
    assert_eq!(
        walk_paired_root(&mut first, &mut second, "game", 0x28, 2, 64)?,
        PairedWalkResult::Matched(vec![8, 0])
    );
    Ok(())
}

#[test]
fn returns_root_to_target_offsets_through_deep_relocated_graphs() -> Result<()> {
    const DEPTH: usize = 64;
    fn chain(base: u64) -> PointerScanResults {
        let mut levels = Vec::new();
        for depth in 1..DEPTH {
            let address = base + depth as u64 * 256;
            let next_address = address - 256;
            let value = if depth % 2 == 1 { next_address + 16 } else { next_address - 8 };
            levels.push(PointerScanLevelCandidates::with_sorted_heap_slots(
                depth as u64,
                vec![],
                depth as u64,
                vec![PointerScanHeapSlot {
                    pointer_address: address,
                    pointer_value: value,
                }],
            ));
        }
        let root = PointerScanCandidate::new(
            99,
            DEPTH as u64,
            PointerScanNodeType::Static,
            0x1000,
            base + (DEPTH as u64 - 1) * 256 - 8,
            0,
            40,
        );
        levels.push(PointerScanLevelCandidates::new(DEPTH as u64, vec![root], vec![]));
        PointerScanResults::new(
            1,
            PointerScanTargetDescriptor::address(base),
            vec![base],
            PointerScanAddressSpace::EmulatorMemory,
            PointerScanPointerSize::Pointer64,
            DEPTH as u64,
            32,
            vec!["game".to_string()],
            vec![],
            levels,
            1,
            DEPTH as u64 - 1,
        )
    }
    let expected = (1..=DEPTH)
        .rev()
        .map(|depth| if depth % 2 == 0 { 8 } else { -16 })
        .collect();
    assert_eq!(
        walk_paired_root(&mut chain(0x4000), &mut chain(0x9000), "game", 40, DEPTH, 1000)?,
        PairedWalkResult::Matched(expected)
    );
    Ok(())
}

#[test]
fn agrees_with_exhaustive_two_hop_routes() -> Result<()> {
    let offsets = [-32_i64, -8, 0, 8, 32];
    for pattern in 0_u32..32 {
        let children = |base: u64, target: u64, mask: u32| -> Vec<(u64, u64)> {
            offsets
                .iter()
                .enumerate()
                .filter(|(bit, _)| mask & (1 << bit) != 0)
                .map(|(bit, offset)| {
                    (
                        base.checked_add_signed(*offset)
                            .expect("fixture slot is in range"),
                        target + ((mask + bit as u32) % 5) as u64,
                    )
                })
                .collect()
        };
        let left = children(200, 400, pattern);
        for other_pattern in 0_u32..32 {
            let right = children(600, 700, other_pattern);
            let mut first = graph(0, 200, &left, 400);
            let mut second = graph(1, 600, &right, 700);
            let common: Vec<_> = left
                .iter()
                .flat_map(|&(addr, value)| {
                    right.iter().filter_map(move |&(other_addr, other_value)| {
                        let sequence = vec![i128::from(addr) - 200, 400 - i128::from(value)];
                        (sequence == [i128::from(other_addr) - 600, 700 - i128::from(other_value)]).then_some(sequence)
                    })
                })
                .collect();
            match walk_paired_root(&mut first, &mut second, "game", 0x28, 2, 100)? {
                PairedWalkResult::Matched(sequence) => assert!(common.contains(&sequence)),
                PairedWalkResult::NoMatch => assert!(common.is_empty()),
                PairedWalkResult::WorkLimit => bail!("fixture exceeded work limit"),
            }
        }
    }
    Ok(())
}

fn walk_paired_root(
    previous: &mut PointerScanResults,
    current: &mut PointerScanResults,
    module: &str,
    rva: u64,
    depth: usize,
    limit: usize,
) -> Result<PairedWalkResult> {
    Ok(PairedPointerWalk::new(previous, current, module, depth)?.walk_root(rva, limit))
}

#[test]
fn indexes_roots_in_capture_order_independently_of_module_index() -> Result<()> {
    let mut first = graph(0, 0x2000, &[(0x2000, 0x3000)], 0x3000);
    let mut second = graph(1, 0x5000, &[(0x5000, 0x6000)], 0x6000);
    let current_root = second.get_pointer_scan_level_candidates()[1].get_static_candidates()[0].clone();
    let other = PointerScanCandidate::new(2, 2, PointerScanNodeType::Static, 0x9000, 0, 0, 0);
    second.get_pointer_scan_level_candidates_mut()[1] = PointerScanLevelCandidates::with_sorted_heap_slots(2, vec![current_root, other], 3, vec![]);
    let mut walk = PairedPointerWalk::new(&mut first, &mut second, "game", 2)?;
    assert_eq!(walk.baseline_root_offsets().collect::<Vec<_>>(), [0x28]);
    assert_eq!(walk.walk_root(0x28, 64), PairedWalkResult::Matched(vec![0, 0]));
    Ok(())
}

#[test]
fn a_missing_module_is_an_input_error() {
    let mut first = graph(0, 1, &[], 2);
    let mut second = graph(1, 1, &[], 2);
    assert!(PairedPointerWalk::new(&mut first, &mut second, "other", 2).is_err());
}
