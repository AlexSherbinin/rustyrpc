use alloc::sync::Arc;
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};
use flume::{Receiver, SendError, Sender};
use std::io;

use quinn::ConnectionError;
use tokio::sync::{Semaphore, TryAcquireError};

use crate::{
    multipart::MultipartSendable,
    transport::{self, quic::stream::Stream},
};

pub struct PooledStream {
    inner: MaybeUninit<Stream>,
    pool: Arc<StreamPool>,
}

impl Drop for PooledStream {
    #[allow(clippy::undocumented_unsafe_blocks, clippy::let_underscore_must_use)]
    fn drop(&mut self) {
        let stream = unsafe { self.inner.assume_init_read() };
        let _: Result<(), SendError<Stream>> = self.pool.stream_sender.send(stream);
    }
}

impl Deref for PooledStream {
    type Target = Stream;

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.assume_init_ref() }
    }
}

impl DerefMut for PooledStream {
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.assume_init_mut() }
    }
}

impl transport::Stream for PooledStream {
    async fn send(&mut self, message: Vec<u8>) -> io::Result<()> {
        self.deref_mut().send(message).await
    }
    async fn send_not_prefixed(&mut self, message: Vec<u8>) -> io::Result<()> {
        self.deref_mut().send_not_prefixed(message).await
    }
    async fn send_multipart(&mut self, multipart: &MultipartSendable) -> io::Result<()> {
        self.deref_mut().send_multipart(multipart).await
    }
    async fn receive(&mut self) -> io::Result<Vec<u8>> {
        self.deref_mut().receive().await
    }
    async fn receive_not_prefixed(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        self.deref_mut().receive_not_prefixed(buffer).await
    }
    async fn flush(&mut self) -> io::Result<()> {
        self.deref_mut().flush().await
    }
}

pub(super) struct StreamPool {
    connection: quinn::Connection,
    stream_sender: Sender<Stream>,
    stream_receiver: Receiver<Stream>,
    size: Semaphore,
}

impl StreamPool {
    pub(super) fn new(connection: quinn::Connection, max_size: usize) -> Self {
        let (stream_sender, stream_receiver) = flume::bounded(max_size);
        Self {
            connection,
            stream_sender,
            stream_receiver,
            size: Semaphore::new(max_size),
        }
    }

    async fn try_create_stream(&self) -> Result<Result<Stream, ConnectionError>, TryAcquireError> {
        self.size.try_acquire()?.forget();
        Ok(self.connection.open_bi().await.map(Into::into))
    }

    #[allow(clippy::undocumented_unsafe_blocks)]
    pub(super) async fn get(self: &Arc<Self>) -> Result<PooledStream, ConnectionError> {
        if let Ok(stream) = self.stream_receiver.try_recv() {
            return Ok(self.new_pooled_stream(stream));
        }

        if let Ok(stream_creation_result) = self.try_create_stream().await {
            return stream_creation_result.map(|stream| self.new_pooled_stream(stream));
        }

        let stream = self.stream_receiver.recv_async().await;
        Ok(self.new_pooled_stream(unsafe { stream.unwrap_unchecked() }))
    }

    fn new_pooled_stream(self: &Arc<Self>, stream: Stream) -> PooledStream {
        PooledStream {
            inner: MaybeUninit::new(stream),
            pool: Arc::clone(self),
        }
    }
}
