use std::path::Path;

pub trait SerializableProjectFile {
    fn save_to_path(
        &mut self,
        directory: &Path,
        allow_overwrite: bool,
        force_save: bool,
    ) -> anyhow::Result<()>;
    fn load_from_path(directory: &Path) -> anyhow::Result<Self>
    where
        Self: Sized;
}
