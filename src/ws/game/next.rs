use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::future::Future;
use std::task::{self, Poll, Waker};

#[derive(Clone, Debug)]
pub struct NextFut {
    next: Arc<Mutex<bool>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl NextFut {
    pub fn new() -> Self {
        Self {
            next: Arc::new(Mutex::new(false)),
            waker: Arc::new(Mutex::new(None)),
        }
    }

    pub fn wait(&self) -> NextFut {
        NextFut {
            next: self.next.clone(),
            waker: self.waker.clone(),
        }
    }

    pub fn wake(&self) {
        *self.next.lock().unwrap() = true;
        let waker = self.waker.lock().unwrap().clone();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl Future for NextFut {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if *self.next.lock().unwrap() {
            *self.next.lock().unwrap() = false;
            *self.waker.lock().unwrap() = None;
            Poll::Ready(())
        } else {
            *self.waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
