use squalr_engine_api::commands::patches::patches_response::PatchesResponse;
use squalr_engine_api::structures::patches::PatchCommandStatus;

pub fn handle_patches_response(response: PatchesResponse) {
    match response {
        PatchesResponse::Apply { patch_apply_response } => {
            print_patch_status("Patch apply", &patch_apply_response.status);
            if let Some(patch) = patch_apply_response.patch {
                println!("Applied patch {} at 0x{:X}.", patch.get_patch_id(), patch.get_region().get_base_address());
            }
        }
        PatchesResponse::NoOperation { patch_no_operation_response } => {
            print_patch_status("No-operation patch", &patch_no_operation_response.status);
            if let Some(patch) = patch_no_operation_response.patch {
                println!(
                    "Applied no-operation patch {} at 0x{:X}.",
                    patch.get_patch_id(),
                    patch.get_region().get_base_address()
                );
            }
        }
        PatchesResponse::Restore { patch_restore_response } => {
            print_patch_status("Patch restore", &patch_restore_response.status);
            if let Some(patch) = patch_restore_response.patch {
                println!("Restored patch {}.", patch.get_patch_id());
            }
        }
        PatchesResponse::RestoreAddress {
            patch_restore_address_response,
        } => {
            print_patch_status("Patch restore", &patch_restore_address_response.status);
            if let Some(patch) = patch_restore_address_response.patch {
                println!("Restored patch {}.", patch.get_patch_id());
            }
        }
        PatchesResponse::List { patch_list_response } => {
            print_patch_status("Patch list", &patch_list_response.status);
            for patch in patch_list_response.patches {
                println!(
                    "{} 0x{:X}-0x{:X} active={} {:?} {}",
                    patch.get_patch_id(),
                    patch.get_region().get_base_address(),
                    patch.get_region().get_end_address(),
                    patch.get_is_active(),
                    patch.get_kind(),
                    patch.get_label().unwrap_or_default()
                );
            }
        }
    }
}

fn print_patch_status(
    label: &str,
    status: &PatchCommandStatus,
) {
    if status.get_success() {
        println!("{} succeeded.", label);
    } else {
        println!("{} failed: {}", label, status.get_message().unwrap_or("unknown error"));
    }
}
