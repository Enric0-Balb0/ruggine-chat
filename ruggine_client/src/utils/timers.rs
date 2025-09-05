use std::time::Duration;

/// Async sleep for wasm using gloo_timers
pub async fn sleep_ms(ms: u64) {
    // gloo_timers::future::sleep returns a Future that resolves after the duration
    gloo_timers::future::sleep(Duration::from_millis(ms)).await;
}

/// Return a monotonically-increasing timestamp in milliseconds suitable for both wasm and native.
pub fn now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // Prefer high resolution performance timer in browser
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now() as u64)
            .unwrap_or_else(|| js_sys::Date::now() as u64)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}
