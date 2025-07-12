use std::pin::Pin;
use tokio::spawn;

#[macro_export]
#[allow(unused_macros)]
macro_rules! run_services {
    ($($service:expr),+ $(,)?) => {
        jam_ready::utils::service_runner::ServiceRunner::run(Vec::from([$($service),+]));
    };
}

pub type JamReadyService = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct ServiceRunner;

impl ServiceRunner {
    pub async fn run(futures: Vec<JamReadyService>) {
        spawn(async {
            let mut handles = Vec::new();
            for fut in futures {
                handles.push(spawn(fut));
            }

            for handle in handles {
                handle.await.expect("Task panicked");
            }
        });
    }
}