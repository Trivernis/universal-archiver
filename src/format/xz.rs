use crate::format::{FileFormat, FileObject};
use anyhow::{bail, Context};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

pub const XZ_HEADER: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

pub struct XZFormat;

impl FileFormat for XZFormat {
    fn parse(file: &FileObject) -> anyhow::Result<Self> {
        if file.header.starts_with(XZ_HEADER) {
            if !file.ext.ends_with("xz") && !file.ext.ends_with("lzma") && !file.ext.ends_with("7z")
            {
                tracing::warn!("The file has a xz signature but not a xz extension.");
            }
            Ok(Self)
        } else {
            bail!("Not an xz file");
        }
    }

    fn extract(&self, file: &Path, output: &Path) -> anyhow::Result<()> {
        if output.is_dir() {
            bail!("The given output is a directory");
        }
        let mut reader = BufReader::new(File::open(&file).context("Opening input file")?);
        let mut output = File::create(&output).context("Creating output file")?;
        tracing::debug!("Decompressing file {file:?} to output {output:?}");
        lzma_rs::xz_decompress(&mut reader, &mut output).context("Decompressing file")?;

        Ok(())
    }
}
