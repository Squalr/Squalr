use crate::process_query::process_queryer::ProcessQueryer;
use image::DynamicImage;
use sysinfo::Pid;

pub trait ProcessExtensionMethods {
    fn has_window(
        &self,
        query: &dyn ProcessQueryer,
    ) -> bool;
    fn is_32bit(&self) -> bool;
    fn is_64bit(&self) -> bool;
    fn get_icon(
        &self,
        query: &dyn ProcessQueryer,
    ) -> Option<DynamicImage>;
}

impl ProcessExtensionMethods for Pid {
    fn has_window(
        &self,
        query: &dyn ProcessQueryer,
    ) -> bool {
        query.is_process_windowed(self)
    }

    fn is_32bit(&self) -> bool {
        !cfg!(target_pointer_width = "64")
    }

    fn is_64bit(&self) -> bool {
        cfg!(target_pointer_width = "64")
    }

    fn get_icon(
        &self,
        query: &dyn ProcessQueryer,
    ) -> Option<DynamicImage> {
        query.get_icon(self)
    }
}
