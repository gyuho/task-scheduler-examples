use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

#[derive(Debug)]
pub struct Generator {
    prefix: u64,
    suffix: AtomicU64,
}

impl Generator {
    pub fn new(member_id: u64, since: Duration) -> Self {
        let unix_ms = since.as_nanos() / 1000000;

        let x = (u64::MAX >> 24) & (unix_ms as u64);
        let x = x << 8;

        Self {
            prefix: member_id << (8 * 6),
            suffix: AtomicU64::new(x),
        }
    }

    pub fn next(&self) -> u64 {
        // ref. https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html
        let suffix = self.suffix.fetch_add(1, Ordering::Acquire);
        let id = self.prefix | (suffix & (u64::MAX >> 16));
        id
    }
}

#[test]
fn test_generator() {
    let member_id = rand::random::<u64>();

    use std::time::Instant;
    let elapsed = Instant::now().elapsed();
    let gen = Generator::new(member_id, elapsed);

    println!("{}", gen.next());
    println!("{}", gen.next());
}

#[test]
fn test_generator_next() {
    let gen = Generator::new(0x12, Duration::from_millis(0x3456));
    let id = gen.next();
    assert_eq!(id, 0x12000000345600);
    for i in 0..1000 {
        let id2 = gen.next();
        assert_eq!(id2, id + i + 1);
    }
}

#[test]
fn test_generator_unique() {
    let gen0 = Generator::new(0, Duration::from_millis(100));
    let id0 = gen0.next();

    let gen1 = Generator::new(1, Duration::from_millis(100));
    let id1 = gen1.next();
    assert_ne!(id0, id1);

    let gen0_restarted = Generator::new(0, Duration::from_millis(101));
    let id0_restarted = gen0_restarted.next();
    assert_ne!(id0, id0_restarted);
}
