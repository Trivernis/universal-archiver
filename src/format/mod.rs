mod gzip;
mod tar;
mod xz;
mod zip;

use crate::format::gzip::GZipFormat;
use crate::format::tar::TarFormat;
use crate::format::xz::XZFormat;
use crate::format::zip::ZipFormat;
use anyhow::{bail, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub enum Format {
    Zip(ZipFormat),
    Xz(XZFormat),
    Gz(GZipFormat),
    Tar(TarFormat),
}

pub struct FileObject {
    pub ext: String,
    pub header: [u8; 32],
}

pub trait FileFormat: Sized {
    fn parse(file: &FileObject) -> Result<Self>;
    fn extract(&self, file: &Path, output: &Path) -> Result<()>;
}

impl FileFormat for Format {
    fn parse(file: &FileObject) -> Result<Self> {
        if let Ok(zip) = ZipFormat::parse(file) {
            tracing::info!("Detected zip format");
            Ok(Self::Zip(zip))
        } else if let Ok(tar) = TarFormat::parse(file) {
            tracing::info!("Detected tar format");
            Ok(Self::Tar(tar))
        } else if let Ok(xz) = XZFormat::parse(file) {
            tracing::info!("Detected xz format");
            Ok(Self::Xz(xz))
        } else if let Ok(gz) = GZipFormat::parse(file) {
            tracing::info!("Detected gzip format");
            Ok(Self::Gz(gz))
        } else {
            bail!("Unknown file format");
        }
    }

    fn extract(&self, file: &Path, output: &Path) -> Result<()> {
        match self {
            Format::Zip(zip) => zip.extract(file, output),
            Format::Xz(xz) => xz.extract(file, output),
            Format::Gz(gz) => gz.extract(file, output),
            Format::Tar(tar) => tar.extract(file, output),
        }
    }
}

/// Parses the format of the file
pub fn parse_format(file: &Path) -> Result<Format> {
    let obj = FileObject {
        ext: get_file_extensions(file).unwrap_or_default(),
        header: get_file_header(file)?,
    };

    Format::parse(&obj)
}

/// Returns the extensions for a given file.
/// This works different to the extension format of the standard library
/// as it recognizes everything after the first dot as an extension. As we're
/// just using the extensions for format detection that behaviour isn't a problem.
fn get_file_extensions(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    let extensions: String = name
        .split('.')
        .skip(1)
        .fold(String::new(), |acc, val| format!("{acc}.{val}"));

    Some(extensions)
}

/// Returns the first 32 bytes of the file that can be used to detect
/// the signature from the magic number
fn get_file_header(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut header_buf = [0u8; 32];
    file.read_exact(&mut header_buf)?;

    Ok(header_buf)
}
