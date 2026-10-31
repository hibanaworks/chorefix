//! Polling clock for the in-process Miri fixture; OS wake delivery is native-tested.
use std::{
    future::poll_fn,
    io,
    task::{Poll, Waker},
    time::{Duration, Instant},
};

pub(super) struct Clock;

impl hibana_runtime::Driver for Clock {
    fn waker(&self) -> Waker {
        Waker::noop().clone()
    }

    fn wait(&mut self) {
        // Every tick polls all live localsides, including timed pending probes.
        // No scoped address or protocol state is retained in the wake handle.
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(super) async fn deadline<T, E: From<io::Error>>(duration: Duration) -> Result<T, E> {
    let start = Instant::now();
    poll_fn(|_| {
        if start.elapsed() >= duration {
            Poll::Ready(Err(io::Error::from(io::ErrorKind::TimedOut).into()))
        } else {
            Poll::Pending
        }
    })
    .await
}
