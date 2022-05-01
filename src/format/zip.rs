use crate::format::{FileFormat, FileObject};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::path::Path;
use std::{fs, io};
use zip::ZipArchive;

const ZIP_HEADER: &[u8] = &[0x50, 0x4b];

pub struct ZipFormat;

impl FileFormat for ZipFormat {
    fn parse(file: &FileObject) -> Result<Self> {
        if file.header.starts_with(ZIP_HEADER) {
            if !file.ext.ends_with("zip") {
                tracing::warn!("The file has a zip signature but no zip extension.");
            }
            Ok(Self)
        } else {
            bail!("Not a zip file");
        }
    }

    fn extract(&self, file: &Path, output: &Path) -> Result<()> {
        if output.is_file() {
            bail!("The given output is a file");
        }
        let file = File::open(file)?;
        let mut archive = ZipArchive::new(file).context("Opening zip file")?;
        tracing::info!("Zip file with {} entries", archive.len());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("Getting file by index")?;
            let path = match file.enclosed_name() {
                None => {
                    tracing::error!(
                        "Cannot extract {:?} because it has an invalid name",
                        file.name()
                    );
                    continue;
                }
                Some(path) => path,
            };
            let output_path = output.join(path);

            if (*file.name()).ends_with('/') {
                tracing::debug!("Creating directory {output_path:?}");
                fs::create_dir_all(output_path).context("Creating directory")?;
            } else {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        tracing::debug!("Creating parent directory {parent:?}");
                        fs::create_dir_all(parent).context("Creating parent dir")?;
                    }
                }
                let mut file_output = File::create(&output_path).context("Creating output file")?;
                tracing::debug!("Extracting to {output_path:?}");
                io::copy(&mut file, &mut file_output).context("Writing output file")?;
            }
        }

        Ok(())
    }
}
