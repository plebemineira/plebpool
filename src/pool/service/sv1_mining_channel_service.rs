use std::{
    future::Future,
    pin::Pin,
    task::Poll,
};
use tracing::{info, debug};

pub struct Sv1MiningChannelService {
    tcp_listener: tokio::net::TcpListener,
}

impl Sv1MiningChannelService {
    pub async fn new(listen_host: String, listen_port: u16) -> anyhow::Result<Self> {
        let tcp_listener = tokio::net::TcpListener::bind((listen_host.as_str(), listen_port)).await?;

        Ok(Self { tcp_listener })
    }
}

impl<R> tower::Service<R> for Sv1MiningChannelService
where
    R: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    type Response = ();
    type Error = ();
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // self.inner.poll_ready(cx)
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: R) -> Self::Future {
        debug!("this is Sv1MiningChannelService::call");
        let fut = async move { Ok(()) };
        Box::pin(fut)
    }
}

// some boilerplate for tower_test
// inspired by https://github.com/tower-rs/tower/blob/master/tower-test/tests/mock.rs

#[cfg(test)]
use tokio_test::{assert_pending, assert_ready};
use tower_test::{assert_request_eq, mock};
#[tokio::test(flavor = "current_thread")]
async fn single_request_ready() {
    let (mut service, mut handle) = mock::spawn();

    assert_pending!(handle.poll_request());

    assert_ready!(service.poll_ready()).unwrap();

    let response = service.call("hello");

    assert_request_eq!(handle, "hello").send_response("world");

    assert_eq!(response.await.unwrap(), "world");
}

#[tokio::test(flavor = "current_thread")]
#[should_panic]
async fn backpressure() {
    let (mut service, mut handle) = mock::spawn::<_, ()>();

    handle.allow(0);

    assert_pending!(service.poll_ready());

    service.call("hello").await.unwrap();
}
