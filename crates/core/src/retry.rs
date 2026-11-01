use std::future::Future;

use tokio::time::{Duration, sleep};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: u32,
}

impl RetryPolicy {
    pub const fn fixed(max_attempts: u32, interval: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay: interval,
            max_delay: interval,
            backoff_multiplier: 1,
        }
    }

    pub const fn exponential(
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            backoff_multiplier: 2,
        }
    }

    pub fn delay_before_attempt(&self, attempt: u32) -> Duration {
        let completed_retries = attempt.saturating_sub(2);
        let multiplier = self.backoff_multiplier.saturating_pow(completed_retries);
        self.initial_delay
            .saturating_mul(multiplier)
            .min(self.max_delay)
    }
}

pub async fn retry<F, Fut, T, E>(task: F, policy: RetryPolicy) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    retry_if(task, policy, |_| true).await
}

pub async fn retry_if<F, Fut, T, E, P>(
    mut task: F,
    policy: RetryPolicy,
    mut should_retry: P,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: FnMut(&E) -> bool,
{
    let mut attempt = 1;
    loop {
        match task().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt >= policy.max_attempts || !should_retry(&error) {
                    return Err(error);
                }
                attempt += 1;
                sleep(policy.delay_before_attempt(attempt)).await;
            }
        }
    }
}
