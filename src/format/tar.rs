use crate::format::gzip::GZipFormat;
use crate::format::xz::XZFormat;
use crate::format::{get_file_header, FileFormat, FileObject};
use anyhow::{bail, Context};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};
use tar::{Archive, EntryType};
use tempfile::{tempdir, TempDir};

const TAR_HEADER: &[u8] = &[0x75, 0x73, 0x74, 0x61, 0x72];

pub enum TarFormat {
    Xz(XZFormat),
    Gz(GZipFormat),
    Uncompressed,
}

impl FileFormat for TarFormat {
    fn parse(file: &FileObject) -> anyhow::Result<Self> {
        if file.header.starts_with(TAR_HEADER) {
            tracing::info!("Detected uncompressed tar file");

            Ok(Self::Uncompressed)
        } else if file.ext.contains(".tar.") {
            if let Ok(xz) = XZFormat::parse(file) {
                tracing::info!("Detected tar file compressed with xz");

                Ok(Self::Xz(xz))
            } else if let Ok(gz) = GZipFormat::parse(file) {
                tracing::info!("Detected tarfile compressed with gz");

                Ok(Self::Gz(gz))
            } else {
                bail!("Not a tar file or a tar with unknown compression");
            }
        } else if file.ext.ends_with(".tar") {
            tracing::info!("Assuming tar based on the file extension");
            Ok(Self::Uncompressed)
        } else {
            bail!("Not a tar file");
        }
    }

    fn extract(&self, file: &Path, output: &Path) -> anyhow::Result<()> {
        match self {
            TarFormat::Xz(xz) => {
                let (tmp, _h) = create_tempfile()?;
                xz.extract(file, &tmp).context("Decompress with xz")?;
                check_extract_tar(&tmp, output)
            }
            TarFormat::Gz(gz) => {
                let (tmp, _h) = create_tempfile()?;
                gz.extract(file, &tmp).context("Decompress with gz")?;
                check_extract_tar(&tmp, output)
            }
            TarFormat::Uncompressed => extract_tar(file, output).context("Extract tar"),
        }
    }
}

/// Checks if the given tar has a valid tar signature and extracts it if that's the case
fn check_extract_tar(file: &Path, output: &Path) -> anyhow::Result<()> {
    if !has_tar_header(file)? {
        tracing::debug!("The extracted tar doesn't have a valid tar signature. This is normal for non POSIX compliant tars.");
    }
    extract_tar(file, output).context("Extract tar")
}

/// Extracts a tar file to the given output directory
fn extract_tar(file: &Path, output: &Path) -> anyhow::Result<()> {
    if output.is_file() {
        bail!("The output must be a directory.");
    }
    let reader = BufReader::new(File::open(file).context("Opening input file")?);
    let mut archive = Archive::new(reader);

    for file in archive.entries().context("Reading tar entries")? {
        let mut file = file.context("Retrieving tar file entry")?;
        let header = file.header();
        let file_path = header.path().context("Retrieving path of file")?;
        let output_path = output.join(file_path);

        match header.entry_type() {
            EntryType::Regular => {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        tracing::debug!("Creating parent path {parent:?}");
                        fs::create_dir_all(parent)?;
                    }
                }
                tracing::debug!("Decompressing entry to {output_path:?}");
                let mut output_file = File::create(&output_path)
                    .with_context(|| format!("Create output file {output_path:?}"))?;
                io::copy(&mut file, &mut output_file).context("writing tar entry to output")?;
            }
            EntryType::Directory => {
                tracing::debug!("Creating output directory {output_path:?}");
                fs::create_dir_all(&output_path)
                    .with_context(|| format!("Failed to create output directory {output_path:?}"))?
            }
            other => {
                tracing::debug!("Ignoring entry of type {other:?}");
            }
        }
    }

    Ok(())
}

fn create_tempfile() -> anyhow::Result<(PathBuf, TempDir)> {
    let tmp_dir = tempdir().context("Create tempdir")?;
    let tmp_file = tmp_dir.path().join(format!(
        ".extract-file-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    Ok((tmp_file, tmp_dir))
}

/// Reads the header of the given file to check if it's a tar file
fn has_tar_header(file: &Path) -> anyhow::Result<bool> {
    let header = get_file_header(file).context("Get file header")?;

    Ok(header.starts_with(TAR_HEADER))
}
