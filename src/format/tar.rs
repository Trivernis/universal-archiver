use crate::format::gzip::GZipFormat;
use crate::format::xz::XZFormat;
use crate::format::{FileFormat, FileObject};
use anyhow::{bail, Context};
use libflate::gzip;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::{fs, io};
use tar::{Archive, EntryType};

const TAR_HEADER: &[u8] = &[0x75, 0x73, 0x74, 0x61, 0x72];

pub enum TarFormat {
    Xz,
    Gz,
    Uncompressed,
}

impl FileFormat for TarFormat {
    fn parse(file: &FileObject) -> anyhow::Result<Self> {
        if file.header.starts_with(TAR_HEADER) {
            tracing::info!("Detected uncompressed tar file");

            Ok(Self::Uncompressed)
        } else if file.ext.contains(".tar.") {
            if let Ok(_xz) = XZFormat::parse(file) {
                tracing::info!("Detected tar file compressed with xz");

                Ok(Self::Xz)
            } else if let Ok(_gz) = GZipFormat::parse(file) {
                tracing::info!("Detected tarfile compressed with gz");

                Ok(Self::Gz)
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
        let mut reader = BufReader::new(File::open(file).context("Opening input")?);
        match self {
            TarFormat::Xz => {
                tracing::debug!("Creating memory mapped file");
                tracing::debug!("Decompressing into memorys");
                let mut buf = Vec::new();
                lzma_rs::xz_decompress(&mut reader, &mut buf).context("Decompressing file")?;
                extract_tar(&mut &buf[..], output)
            }
            TarFormat::Gz => {
                let mut decoder = gzip::Decoder::new(&mut reader).context("Creating decoder")?;
                extract_tar(&mut decoder, output)
            }
            TarFormat::Uncompressed => extract_tar(&mut reader, output).context("Extract tar"),
        }
    }
}
/// Extracts a tar file to the given output directory
fn extract_tar<R: Read>(reader: &mut R, output: &Path) -> anyhow::Result<()> {
    if output.is_file() {
        bail!("The output must be a directory.");
    }
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
                let mut output_file = BufWriter::new(
                    File::create(&output_path)
                        .with_context(|| format!("Create output file {output_path:?}"))?,
                );
                io::copy(&mut file, &mut output_file).context("writing tar entry to output")?;
                output_file.flush()?;
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
