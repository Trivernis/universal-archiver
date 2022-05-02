use crate::utils::channel_sink::ChannelSink;
use lzma_rs::xz_decompress;
use std::cmp::min;
use std::io;
use std::io::{BufRead, Read, Write};
use std::sync::mpsc::Receiver;

pub struct XzDecoder {
    buffer: Vec<u8>,
    rx: Receiver<Vec<u8>>,
}

impl XzDecoder {
    pub fn new<R: BufRead + Send + 'static>(mut reader: R) -> Self {
        let (mut sink, rx) = ChannelSink::new(1024);
        std::thread::spawn(move || {
            tracing::debug!("Async decompression thread running");
            if let Err(e) = xz_decompress(&mut reader, &mut sink) {
                tracing::error!("Async decompressing finished with error {e}");
            } else {
                tracing::debug!("async decompressing succeeded");
            }
        });
        Self {
            rx,
            buffer: Vec::new(),
        }
    }
}

impl Read for XzDecoder {
    #[tracing::instrument(skip_all, level = "trace")]
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        self.buffer.reverse();
        if self.buffer.is_empty() {
            tracing::trace!("Receiving chunk from channel");
            if let Ok(chunk) = self.rx.recv() {
                self.buffer = chunk;
            } else {
                tracing::debug!("Receiving timed out");
            }
        }

        let max_write = min(self.buffer.len(), buf.len());
        tracing::trace!("Wrote {max_write} bytes");
        buf.write_all(&self.buffer[0..max_write])?;
        self.buffer.reverse();
        self.buffer.truncate(self.buffer.len() - max_write);

        Ok(max_write)
    }
}
