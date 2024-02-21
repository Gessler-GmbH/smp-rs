use core::{
    future::Future,
    marker::PhantomPinned,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use super::{AsyncSMPTransport, Result};

pub(crate) fn send<'a, T>(transport: &'a mut T, frame: &'a [u8]) -> Send<'a, T>
where
    T: AsyncSMPTransport + Unpin + ?Sized,
{
    Send {
        transport,
        frame,
        _pin: PhantomPinned,
    }
}

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Send<'a, T: ?Sized> {
        transport: &'a mut T,
        frame: &'a [u8],
        #[pin]
        _pin: PhantomPinned,
    }
}

impl<T> Future for Send<'_, T>
where
    T: AsyncSMPTransport + Unpin + ?Sized,
{
    type Output = Result;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        Pin::new(&mut *me.transport).poll_send(cx, me.frame)
    }
}

pub(crate) fn receive<'a, T>(transport: &'a mut T) -> Receive<'a, T>
where
    T: AsyncSMPTransport + Unpin + ?Sized,
{
    Receive {
        transport,
        _pin: PhantomPinned,
    }
}

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Receive<'a, T: ?Sized> {
        transport: &'a mut T,
        #[pin]
        _pin: PhantomPinned,
    }
}

impl<T> Future for Receive<'_, T>
where
    T: AsyncSMPTransport + Unpin + ?Sized,
{
    type Output = Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        Pin::new(&mut *me.transport).poll_receive(cx)
    }
}
