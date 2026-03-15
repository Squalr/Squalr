use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Graphics::Gdi::{DeleteDC, DeleteObject, HBITMAP, HDC, HGDIOBJ, ReleaseDC, SelectObject},
    UI::WindowsAndMessaging::{DestroyIcon, HICON},
};

const SHELL_EXTRACT_ICON_FAILURE_SENTINEL: usize = 1;

pub struct IconHandle(pub HICON);

impl IconHandle {
    pub fn new(icon_handle: HICON) -> Option<Self> {
        if icon_handle.is_null() || icon_handle as usize <= SHELL_EXTRACT_ICON_FAILURE_SENTINEL {
            None
        } else {
            Some(Self(icon_handle))
        }
    }
}

impl Drop for IconHandle {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.0);
        }
    }
}

pub struct ProcessHandle(pub HANDLE);

impl ProcessHandle {
    pub fn new(process_handle: HANDLE) -> Option<Self> {
        if process_handle.is_null() { None } else { Some(Self(process_handle)) }
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub struct DcHandle(pub HDC);

impl Drop for DcHandle {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(std::ptr::null_mut(), self.0);
        }
    }
}

pub struct MemoryDcHandle(pub HDC);

impl Drop for MemoryDcHandle {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.0);
        }
    }
}

pub struct BitmapHandle(pub HBITMAP);

impl Drop for BitmapHandle {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.0 as HGDIOBJ);
        }
    }
}

pub struct SelectedGdiObjectGuard {
    device_context: HDC,
    previous_object: HGDIOBJ,
}

impl SelectedGdiObjectGuard {
    pub fn new(
        device_context: HDC,
        previous_object: HGDIOBJ,
    ) -> Self {
        Self {
            device_context,
            previous_object,
        }
    }
}

impl Drop for SelectedGdiObjectGuard {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.device_context, self.previous_object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IconHandle;

    #[test]
    fn rejects_shell_extract_icon_failure_sentinel() {
        assert!(IconHandle::new(1usize as *mut core::ffi::c_void).is_none());
    }
}
