use std::path::Path;

pub trait SerializableProjectFile {
    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()>;
    fn load_from_path(directory: &Path) -> anyhow::Result<Self>
    where
        Self: Sized;
}
