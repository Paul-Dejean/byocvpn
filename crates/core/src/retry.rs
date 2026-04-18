use std::future::Future;

use tokio::time::{Duration, sleep};

pub async fn retry<F, Fut, T, E>(
    mut task: F,
    max_attempts: u32,
    interval: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut result = task().await;
    for _ in 1..max_attempts {
        if result.is_ok() {
            return result;
        }
        sleep(interval).await;
        result = task().await;
    }
    result
}
