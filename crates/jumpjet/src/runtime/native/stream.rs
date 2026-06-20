//! Host-side helpers for the component-model native `stream<u8>` type.
//!
//! wasmtime 45/47 ships no ergonomic read API — consuming a `stream<u8>` means
//! implementing the poll-based `StreamConsumer` trait and driving it with
//! `StreamReader::pipe` (this mirrors wasmtime's own WASI filesystem
//! `WriteStreamConsumer`). These helpers wrap that once so the interface
//! functions stay small. Producing is already easy: `Vec<u8>` implements
//! `StreamProducer`, so `StreamReader::new(store, bytes)` emits a stream.
//!
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::channel::oneshot;
use wasmtime::{AsContextMut, StoreContextMut};
use wasmtime::component::{Accessor, HasData, Source, StreamConsumer, StreamReader, StreamResult};

/// A `StreamConsumer` that drains a `stream<u8>` into a `Vec`, handing the
/// collected bytes back through a oneshot when the stream ends. The send happens
/// in `Drop` (when the writer closes the stream and the consumer is torn down),
/// matching the WASI reference consumer.
struct VecCollector {
    buffer: Vec<u8>,
    result: Option<oneshot::Sender<Vec<u8>>>,
}

impl<D> StreamConsumer<D> for VecCollector {
    type Item = u8;

    fn poll_consume(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        store: StoreContextMut<D>,
        src: Source<Self::Item>,
        _finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let mut src = src.as_direct(store);
        let chunk = src.remaining();
        let n = chunk.len();
        if n > 0 {
            self.buffer.extend_from_slice(chunk);
            src.mark_read(n);
        }
        Poll::Ready(Ok(StreamResult::Completed))
    }
}

impl Drop for VecCollector {
    fn drop(&mut self) {
        if let Some(tx) = self.result.take() {
            let _ = tx.send(mem::take(&mut self.buffer));
        }
    }
}

/// Read a `stream<u8>` fully into a `Vec`, host-side, from inside an async
/// (`Accessor`-based) host function.
pub async fn read_stream_to_vec<T, D>(
    accessor: &Accessor<T, D>,
    stream: StreamReader<u8>,
) -> wasmtime::Result<Vec<u8>>
where
    D: HasData,
{
    let (tx, rx) = oneshot::channel();
    accessor.with(|mut access| {
        stream.pipe(
            access.as_context_mut(),
            VecCollector {
                buffer: Vec::new(),
                result: Some(tx),
            },
        )
    })?;
    Ok(rx.await.unwrap_or_default())
}

/// Emit a `Vec<u8>` as a host-backed `stream<u8>`. `Vec<u8>` already implements
/// `StreamProducer`, so this just wraps `StreamReader::new`.
pub fn vec_to_stream<T, D>(
    accessor: &Accessor<T, D>,
    bytes: Vec<u8>,
) -> wasmtime::Result<StreamReader<u8>>
where
    D: HasData,
{
    accessor.with(|mut access| StreamReader::new(access.as_context_mut(), bytes))
}
