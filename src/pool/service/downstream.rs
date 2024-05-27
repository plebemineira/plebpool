use tracing::info;

pub struct DownstreamService {
    listener: tokio::net::TcpListener,
}

impl DownstreamService {
    pub async fn new(listen_host: String, listen_port: u16) -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind((listen_host.as_str(), listen_port)).await?;

        info!(
            "SV2: listening for Downstream connections at: {}:{}",
            listen_host, listen_port
        );

        Ok(Self { listener })
    }

    pub async fn serve(self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        let handle = tokio::task::spawn(async move {
            while let Ok((stream, addr)) = self.listener.accept().await {
                info!("SV2: listening for Downstream connections at: {}", addr);
                let (_receiver, _sender): (
                    async_channel::Receiver<
                        codec_sv2::StandardEitherFrame<
                            roles_logic_sv2::parsers::PoolMessages<'static>,
                        >,
                    >,
                    async_channel::Sender<
                        codec_sv2::StandardEitherFrame<
                            roles_logic_sv2::parsers::PoolMessages<'static>,
                        >,
                    >,
                ) = network_helpers_sv2::plain_connection_tokio::PlainConnection::new(stream).await;

                // todo: do something with receiver and sender
            }

            Ok(())
        });

        Ok(handle)
    }
}
