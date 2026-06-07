use crate::memory_writer::memory_writer_trait::MemoryWriterTrait;
use libc::{WNOHANG, c_void, iovec, pid_t, process_vm_writev};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::{
    fs::OpenOptions,
    mem::size_of,
    os::unix::fs::FileExt,
    ptr::null_mut,
    thread,
    time::{Duration, Instant},
};

const PTRACE_ATTACH_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const PTRACE_ATTACH_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct LinuxMemoryWriter;

impl LinuxMemoryWriter {
    pub fn new() -> Self {
        LinuxMemoryWriter
    }

    fn write_process_memory(
        process_id: u32,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        if source_bytes.is_empty() {
            return true;
        }

        let local_iovec = iovec {
            iov_base: source_bytes.as_ptr() as *mut c_void,
            iov_len: source_bytes.len(),
        };

        let remote_iovec = iovec {
            iov_base: destination_address as *mut c_void,
            iov_len: source_bytes.len(),
        };

        let bytes_written = unsafe { process_vm_writev(process_id as pid_t, &local_iovec, 1, &remote_iovec, 1, 0) };

        bytes_written == source_bytes.len() as isize
    }

    fn write_process_memory_with_proc_mem(
        process_id: u32,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        if source_bytes.is_empty() {
            return true;
        }

        let proc_mem_path = format!("/proc/{}/mem", process_id);
        let Ok(proc_mem_file) = OpenOptions::new().write(true).open(proc_mem_path) else {
            return false;
        };

        proc_mem_file
            .write_at(source_bytes, destination_address)
            .is_ok_and(|bytes_written| bytes_written == source_bytes.len())
    }

    fn write_process_memory_with_ptrace(
        process_id: u32,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        if source_bytes.is_empty() {
            return true;
        }

        let process_id = process_id as pid_t;

        if Self::ptrace_write_bytes(process_id, destination_address, source_bytes) {
            return true;
        }

        if !Self::ptrace_attach(process_id) {
            return false;
        }

        let write_succeeded = Self::ptrace_write_bytes(process_id, destination_address, source_bytes);
        let detach_succeeded = Self::ptrace_detach(process_id);

        write_succeeded && detach_succeeded
    }

    fn ptrace_write_bytes(
        process_id: pid_t,
        destination_address: u64,
        source_bytes: &[u8],
    ) -> bool {
        let word_size = size_of::<libc::c_long>();
        let mut bytes_written = 0usize;

        while bytes_written < source_bytes.len() {
            let write_address = destination_address.saturating_add(bytes_written as u64);
            let word_offset = write_address as usize % word_size;
            let bytes_remaining = source_bytes.len() - bytes_written;
            let bytes_this_word = (word_size - word_offset).min(bytes_remaining);
            let source_end = bytes_written + bytes_this_word;

            let mut word_bytes = if word_offset == 0 && bytes_this_word == word_size {
                [0_u8; size_of::<libc::c_long>()]
            } else {
                let aligned_word_address = write_address.saturating_sub(word_offset as u64);
                let Some(existing_word_bytes) = Self::ptrace_peek_data(process_id, aligned_word_address) else {
                    return false;
                };

                existing_word_bytes
            };

            word_bytes[word_offset..word_offset + bytes_this_word].copy_from_slice(&source_bytes[bytes_written..source_end]);

            let aligned_word_address = write_address.saturating_sub(word_offset as u64);
            if !Self::ptrace_poke_data(process_id, aligned_word_address, word_bytes) {
                return false;
            }

            bytes_written = source_end;
        }

        true
    }

    fn ptrace_attach(process_id: pid_t) -> bool {
        if unsafe { libc::ptrace(libc::PTRACE_ATTACH, process_id, null_mut::<c_void>(), null_mut::<c_void>()) } != 0 {
            return false;
        }

        if Self::wait_for_process_stop(process_id, PTRACE_ATTACH_WAIT_TIMEOUT) {
            true
        } else {
            let _ = Self::ptrace_detach(process_id);
            false
        }
    }

    fn ptrace_detach(process_id: pid_t) -> bool {
        (unsafe { libc::ptrace(libc::PTRACE_DETACH, process_id, null_mut::<c_void>(), null_mut::<c_void>()) }) == 0
    }

    fn ptrace_peek_data(
        process_id: pid_t,
        address: u64,
    ) -> Option<[u8; size_of::<libc::c_long>()]> {
        Self::clear_errno();
        let ptrace_result = unsafe { libc::ptrace(libc::PTRACE_PEEKDATA, process_id, address as *mut c_void, null_mut::<c_void>()) };

        if ptrace_result == -1 && Self::current_errno() != 0 {
            None
        } else {
            Some(ptrace_result.to_ne_bytes())
        }
    }

    fn ptrace_poke_data(
        process_id: pid_t,
        address: u64,
        word_bytes: [u8; size_of::<libc::c_long>()],
    ) -> bool {
        let word = libc::c_long::from_ne_bytes(word_bytes);

        (unsafe { libc::ptrace(libc::PTRACE_POKEDATA, process_id, address as *mut c_void, word as usize as *mut c_void) }) == 0
    }

    fn wait_for_process_stop(
        process_id: pid_t,
        timeout: Duration,
    ) -> bool {
        let wait_started_at = Instant::now();

        while wait_started_at.elapsed() < timeout {
            let mut wait_status = 0;
            let wait_result = unsafe { libc::waitpid(process_id, &mut wait_status, WNOHANG) };

            if wait_result == process_id && libc::WIFSTOPPED(wait_status) {
                return true;
            }

            if wait_result < 0 {
                return false;
            }

            thread::sleep(PTRACE_ATTACH_POLL_INTERVAL);
        }

        false
    }

    fn clear_errno() {
        unsafe {
            *libc::__errno_location() = 0;
        }
    }

    fn current_errno() -> i32 {
        unsafe { *libc::__errno_location() }
    }
}

impl MemoryWriterTrait for LinuxMemoryWriter {
    fn write_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &[u8],
    ) -> bool {
        Self::write_process_memory(process_info.get_process_id_raw(), address, values)
            || Self::write_process_memory_with_proc_mem(process_info.get_process_id_raw(), address, values)
            || Self::write_process_memory_with_ptrace(process_info.get_process_id_raw(), address, values)
    }
}
