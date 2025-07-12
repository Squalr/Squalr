use windows_sys::Win32::{
    Graphics::Gdi::{ReleaseDC, HDC},
    UI::WindowsAndMessaging::{DestroyIcon, HICON},
};

pub struct IconHandle(pub HICON);

impl Drop for IconHandle {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.0);
        }
    }
}

// Similarly for DC
pub struct DcHandle(pub HDC);

impl Drop for DcHandle {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(std::ptr::null_mut(), self.0);
        }
    }
}
