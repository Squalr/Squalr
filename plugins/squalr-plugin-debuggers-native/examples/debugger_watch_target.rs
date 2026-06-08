use std::{
    error::Error,
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

static WATCH_TARGET_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() -> Result<(), Box<dyn Error>> {
    let is_single_threaded = std::env::args().any(|argument| argument == "--single-thread");
    let counter_address = &WATCH_TARGET_COUNTER as *const AtomicU64 as u64;

    println!(
        "READY pid={} address={counter_address:#x} size=8 mode={}",
        std::process::id(),
        if is_single_threaded { "single-thread" } else { "worker-thread" }
    );
    std::io::stdout().flush()?;

    if is_single_threaded {
        run_single_threaded_target()
    } else {
        run_worker_thread_target()
    }
}

fn run_worker_thread_target() -> Result<(), Box<dyn Error>> {
    thread::spawn(|| {
        loop {
            WATCH_TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(10));
        }
    });

    let mut heartbeat_number = 0u64;
    loop {
        let counter_value = WATCH_TARGET_COUNTER.load(Ordering::Relaxed);
        println!("HEARTBEAT sequence={heartbeat_number} value={counter_value}");
        std::io::stdout().flush()?;
        heartbeat_number = heartbeat_number.saturating_add(1);
        thread::sleep(Duration::from_millis(250));
    }
}

fn run_single_threaded_target() -> Result<(), Box<dyn Error>> {
    let mut heartbeat_number = 0u64;
    let mut writes_since_heartbeat = 0u64;

    loop {
        let counter_value = WATCH_TARGET_COUNTER
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        writes_since_heartbeat = writes_since_heartbeat.saturating_add(1);

        if writes_since_heartbeat >= 25 {
            println!("HEARTBEAT sequence={heartbeat_number} value={counter_value}");
            std::io::stdout().flush()?;
            heartbeat_number = heartbeat_number.saturating_add(1);
            writes_since_heartbeat = 0;
        }

        thread::sleep(Duration::from_millis(10));
    }
}
