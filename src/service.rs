use crate::Result;
use axum::Router;
use tokio::{sync::broadcast, task::JoinSet};

pub async fn run_with_ctl_c(services: impl Iterator<Item = Service>) -> Result<()> {
    let (close, close_recv) = broadcast::channel(1);
    let mut service_set = JoinSet::new();
    for service in services {
        service_set.spawn(service.run_single(close_recv.resubscribe()));
    }
    service_set.spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        Ok(())
    });
    service_set.join_next().await;
    close.send(()).map_err(|e| format!("{e}"))?;

    Ok(())
}

pub struct Service {
    name: &'static str,
    addr: String,
    router: Router,
}

impl Service {
    pub fn new(name: &'static str, addr: String, router: Router) -> Self {
        Self { name, addr, router }
    }

    async fn run_single(self, mut close_signal: broadcast::Receiver<()>) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        tracing::info!("Bound {} on {}", self.name, listener.local_addr().unwrap());
        axum::serve(listener, self.router)
            .with_graceful_shutdown(async move {
                let _ = close_signal.recv().await;
            })
            .await?;
        Ok(())
    }
}
