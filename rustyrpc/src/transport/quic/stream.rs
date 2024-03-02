use core::num::TryFromIntError;
use quinn::{RecvStream, SendStream};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

use thiserror::Error;

use crate::transport;

/// Stream via QUIC protocol.
pub struct Stream {
    send_stream: BufWriter<SendStream>,
    receive_stream: BufReader<RecvStream>,
}

#[derive(Error, Debug)]
#[error("Trying to send invalid length prefix")]
struct SendingInvalidLengthPrefixError(#[from] TryFromIntError);

#[derive(Error, Debug)]
#[error("Invalid length prefix received")]
struct InvalidLengthPrefixReceivedError(#[from] TryFromIntError);

impl transport::Stream for Stream {
    async fn send(&mut self, message: Vec<u8>) -> io::Result<()> {
        let length_prefix: u32 = message.len().try_into().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                SendingInvalidLengthPrefixError(err),
            )
        })?;
        let length_prefix = length_prefix.to_be_bytes();

        self.send_stream.write_all(&length_prefix).await?;
        self.send_stream.write_all(&message).await?;
        Ok(())
    }

    async fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut length_prefix_buffer = [0u8; 4];
        self.receive_stream
            .read_exact(&mut length_prefix_buffer)
            .await?;
        let length_prefix: usize = u32::from_be_bytes(length_prefix_buffer)
            .try_into()
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    InvalidLengthPrefixReceivedError(err),
                )
            })?;

        let mut message_buffer = vec![0u8; length_prefix];
        self.receive_stream.read_exact(&mut message_buffer).await?;

        Ok(message_buffer)
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.send_stream.flush().await
    }
}

impl From<(SendStream, RecvStream)> for Stream {
    fn from((send_stream, receive_stream): (SendStream, RecvStream)) -> Self {
        Self {
            send_stream: BufWriter::new(send_stream),
            receive_stream: BufReader::new(receive_stream),
        }
    }
}
