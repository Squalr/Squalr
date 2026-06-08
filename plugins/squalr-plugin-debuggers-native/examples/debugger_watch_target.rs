use std::{
    error::Error,
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

static WATCH_TARGET_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() -> Result<(), Box<dyn Error>> {
    thread::spawn(|| {
        loop {
            WATCH_TARGET_COUNTER.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(10));
        }
    });

    let counter_address = &WATCH_TARGET_COUNTER as *const AtomicU64 as u64;
    println!("READY pid={} address={counter_address:#x} size=8", std::process::id());
    std::io::stdout().flush()?;

    let mut heartbeat_number = 0u64;
    loop {
        let counter_value = WATCH_TARGET_COUNTER.load(Ordering::Relaxed);
        println!("HEARTBEAT sequence={heartbeat_number} value={counter_value}");
        std::io::stdout().flush()?;
        heartbeat_number = heartbeat_number.saturating_add(1);
        thread::sleep(Duration::from_millis(250));
    }
}
