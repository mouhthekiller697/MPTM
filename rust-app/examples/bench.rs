#![allow(dead_code)]
//! Micro-benchmark for the sampling hot path (no GUI required):
//!   cargo run --release --example bench
//! Prints how long one sampling pass takes, excluding the configured sleep.

#[path = "../src/util.rs"]
mod util;
#[path = "../src/gpu.rs"]
mod gpu;
#[path = "../src/sampler.rs"]
mod sampler;

use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let refresh = Arc::new(AtomicU64::new(250));
    let ctx = eframe::egui::Context::default();
    let shared = sampler::spawn(ctx, refresh);
    let mut last = 0u64;
    let mut samples: Vec<u64> = Vec::new();
    while samples.len() < 25 {
        {
            let st = shared.lock().unwrap();
            if st.seq != last {
                last = st.seq;
                samples.push(st.sample_us);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    samples.drain(0..3); // discard warm-up passes
    samples.sort_unstable();
    let avg = samples.iter().sum::<u64>() / samples.len() as u64;
    let median = samples[samples.len() / 2];
    let st = shared.lock().unwrap();
    println!("samples:  {}", samples.len());
    println!("min:      {} µs", samples[0]);
    println!("median:   {} µs", median);
    println!("avg:      {} µs", avg);
    println!("max:      {} µs", samples[samples.len() - 1]);
    println!("apps:     {}", st.snap.apps.len());
    println!("disks:    {}", st.snap.disks.len());
    println!("gpu:      {:?}", st.snap.gpu.as_ref().map(|g| &g.name));
    println!("battery:  {:?}", st.snap.battery.as_ref().map(|b| b.pct));
}
