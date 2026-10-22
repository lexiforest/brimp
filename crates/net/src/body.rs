use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body::{Body as HttpBody, Frame, SizeHint};
use tokio::sync::{mpsc, oneshot};

use crate::error::Error;

/// Decoded response bytes, implementing the same body trait used by Hyper.
/// Dropping the body cancels its transfer. A slow consumer pauses only its own
/// transfer; completion errors are delivered after any already queued bytes.
pub struct Body {
    receiver: mpsc::Receiver<Bytes>,
    completion: oneshot::Receiver<Result<(), Error>>,
    done: bool,
    wake: std::sync::Arc<crate::multi::Wake>,
}

impl Body {
    pub(crate) fn new(
        receiver: mpsc::Receiver<Bytes>,
        completion: oneshot::Receiver<Result<(), Error>>,
        wake: std::sync::Arc<crate::multi::Wake>,
    ) -> Self {
        Self {
            receiver,
            completion,
            done: false,
            wake,
        }
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Error>>> {
        if self.done {
            return Poll::Ready(None);
        }
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(bytes)) => {
                self.wake.notify();
                return Poll::Ready(Some(Ok(Frame::data(bytes))));
            }
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => {}
        }
        use std::future::Future;
        let result = match Pin::new(&mut self.completion).poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(result)) => result,
            Poll::Ready(Err(_)) => Err(Error::Network("curl worker closed the transfer".into())),
        };
        self.done = true;
        Poll::Ready(result.err().map(Err))
    }

    fn is_end_stream(&self) -> bool {
        self.done
    }
    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

impl Drop for Body {
    fn drop(&mut self) {
        self.receiver.close();
        self.wake.notify();
    }
}
