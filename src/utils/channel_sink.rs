use std::io::{ErrorKind, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{io, mem};

pub struct ChannelSink {
    buffer: Vec<u8>,
    block_size: usize,
    tx: SyncSender<Vec<u8>>,
}

impl ChannelSink {
    /// Creates a new sink with a channel to send the data to
    pub fn new(block_size: usize) -> (Self, Receiver<Vec<u8>>) {
        let (tx, rx) = sync_channel(1);
        (
            Self {
                buffer: Vec::new(),
                block_size,
                tx,
            },
            rx,
        )
    }
}

impl Write for ChannelSink {
    #[tracing::instrument(skip_all, level = "trace")]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.append(&mut buf.to_vec());
        if self.buffer.len() >= self.block_size {
            tracing::trace!("Block size reached. Sending buffer...");
            self.tx
                .send(mem::take(&mut self.buffer))
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
        }

        Ok(buf.len())
    }

    #[tracing::instrument(skip_all, level = "trace")]
    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            self.tx
                .send(mem::take(&mut self.buffer))
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
        }

        Ok(())
    }
}

impl Drop for ChannelSink {
    #[tracing::instrument(skip_all, level = "trace")]
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            tracing::debug!("Error while trying to flush buffer during drop {e}")
        }
    }
}
