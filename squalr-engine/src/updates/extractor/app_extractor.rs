use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use zip::ZipArchive;
use zip::read::ZipFile;

pub struct AppExtractor {
    install_dir: PathBuf,
    progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>,
}

impl AppExtractor {
    pub fn new(
        install_dir: &Path,
        progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>,
    ) -> Self {
        Self {
            install_dir: install_dir.to_path_buf(),
            progress_callback,
        }
    }

    pub async fn extract_archive(
        &self,
        archive_path: &Path,
    ) -> Result<()> {
        let mut archive = Self::validate_zip_archive(archive_path)?;
        let total_files = archive.len() as u64;
        let mut files_processed = 0u64;

        log::info!("Clearing existing install directory...");
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir)?;
        }
        fs::create_dir_all(&self.install_dir)?;

        log::info!("Starting archive extraction...");
        let archive_len = archive.len();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            log::info!("Processing file {}/{}: {}", i + 1, archive_len, file.name());

            let file_path = file
                .enclosed_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file path in archive"))?;

            Self::validate_archive_path(&file_path)?;

            let path = self.install_dir.join(file_path);
            log::info!("Extracting to: {}", path.display());

            if file.name().ends_with('/') {
                self.create_directory(&path)?;
            } else {
                self.extract_file(&path, &mut file)?;
            }

            files_processed += 1;
            (self.progress_callback)(files_processed, total_files);
        }

        Ok(())
    }

    fn validate_zip_archive(archive_path: &Path) -> Result<ZipArchive<File>> {
        log::info!("Validating ZIP archive...");
        let archive = ZipArchive::new(File::open(archive_path)?).context("Failed to open ZIP archive - file may be corrupted")?;

        let file_count = archive.len();
        log::info!("ZIP archive contains {} files", file_count);

        if file_count == 0 {
            anyhow::bail!("ZIP archive is empty");
        }

        Ok(archive)
    }

    fn validate_archive_path(path: &Path) -> Result<()> {
        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            log::info!("WARNING: Detected path traversal attempt in: {}", path.display());
            anyhow::bail!("Archive contains invalid path traversal sequences");
        }

        Ok(())
    }

    fn create_directory(
        &self,
        path: &Path,
    ) -> Result<()> {
        log::info!("Creating directory: {}", path.display());
        fs::create_dir_all(path).context("Failed to create directory from archive")
    }

    fn extract_file(
        &self,
        path: &Path,
        file: &mut ZipFile<File>,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            log::info!("Creating parent directory: {}", parent.display());
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }

        let mut outfile = File::create(path).context("Failed to create output file")?;
        let copied = std::io::copy(file, &mut outfile).context("Failed to write file contents")?;
        log::info!("Wrote {} bytes to file", copied);
        Ok(())
    }
}
