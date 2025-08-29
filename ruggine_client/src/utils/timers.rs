use std::time::Duration;

/// Async sleep for wasm using gloo_timers
pub async fn sleep_ms(ms: u64) {
    // gloo_timers::future::sleep returns a Future that resolves after the duration
    gloo_timers::future::sleep(Duration::from_millis(ms)).await;
}
