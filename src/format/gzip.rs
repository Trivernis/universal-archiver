use crate::format::{FileFormat, FileObject};
use anyhow::{bail, Context};
use libflate::gzip::Decoder;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

pub const GZIP_HEADER: &[u8] = &[0x1f, 0x8b];

pub struct GZipFormat;

impl FileFormat for GZipFormat {
    fn parse(file: &FileObject) -> anyhow::Result<Self> {
        if file.header.starts_with(GZIP_HEADER) {
            if !file.ext.ends_with(".gz") && !file.ext.ends_with(".gzip") {
                tracing::error!("The file has a valid gzip signature but not a gzip extension");
            }
            Ok(Self)
        } else {
            bail!("Not a gzip file")
        }
    }

    fn extract(&self, file: &Path, output: &Path) -> anyhow::Result<()> {
        let mut reader = BufReader::new(File::open(file).context("Opening input")?);
        let mut decoder = Decoder::new(&mut reader).context("Creating decoder")?;
        let mut output_file =
            File::create(output).with_context(|| format!("Creating output file {output:?}"))?;
        tracing::debug!("Extracting to {output:?}");
        io::copy(&mut decoder, &mut output_file).context("Deompressing file to output")?;

        Ok(())
    }
}
