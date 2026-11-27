pub mod frame_rate;
pub mod resolution;

/// Apply load-time byte-checked frame rate and resolution patches to the mapped main image
/// before any title code executes (`host_main`).
pub unsafe fn apply_asked(base: usize, size: usize) {
    if let Some(ms) = frame_rate::asked_interval_ms() {
        let asked = std::env::var("XCL_TITLE_FPS")
            .ok()
            .or_else(|| std::env::var("VICTORY_TITLE_FPS").ok());
        match frame_rate::title_rate(ms, asked.as_deref()) {
            Some(rate) => {
                let done = frame_rate::apply(base, size, rate, &frame_rate::SITES);
                let patched = done.iter().filter(|d| **d == frame_rate::Patched::Done).count();
                crate::xlog_info!(
                    "frame rate: clock {ms} ms; title rate {rate} ({patched}/{} sites patched; title asks {} us at 60 Hz)",
                    frame_rate::SITES.len(),
                    frame_rate::title_interval_us(rate, false)
                );
            }
            None => crate::xlog_info!("frame rate: clock {ms} ms; title's own rate left untouched"),
        }
    }
    resolution::apply_asked(base, size);
}
