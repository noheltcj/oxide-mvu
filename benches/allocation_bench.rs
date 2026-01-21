//! Heap allocation benchmarks for oxide-mvu.
//!
//! Measures actual heap allocations to track optimization impact over time.
//!
//! Run with: cargo bench --bench allocation_bench

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracking allocator that wraps the system allocator
struct TrackingAllocator {
    allocated_bytes: AtomicUsize,
    allocation_count: AtomicUsize,
    deallocated_bytes: AtomicUsize,
    deallocation_count: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            allocated_bytes: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocated_bytes: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
        }
    }

    fn reset(&self) {
        self.allocated_bytes.store(0, Ordering::SeqCst);
        self.allocation_count.store(0, Ordering::SeqCst);
        self.deallocated_bytes.store(0, Ordering::SeqCst);
        self.deallocation_count.store(0, Ordering::SeqCst);
    }

    fn report(&self) -> AllocationReport {
        AllocationReport {
            allocated_bytes: self.allocated_bytes.load(Ordering::SeqCst),
            allocation_count: self.allocation_count.load(Ordering::SeqCst),
            deallocated_bytes: self.deallocated_bytes.load(Ordering::SeqCst),
            deallocation_count: self.deallocation_count.load(Ordering::SeqCst),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocated_bytes
            .fetch_add(layout.size(), Ordering::SeqCst);
        self.allocation_count.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocated_bytes
            .fetch_add(layout.size(), Ordering::SeqCst);
        self.deallocation_count.fetch_add(1, Ordering::SeqCst);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

struct AllocationReport {
    allocated_bytes: usize,
    allocation_count: usize,
    deallocated_bytes: usize,
    deallocation_count: usize,
}

impl AllocationReport {
    fn net_bytes(&self) -> isize {
        self.allocated_bytes as isize - self.deallocated_bytes as isize
    }

    fn net_allocations(&self) -> isize {
        self.allocation_count as isize - self.deallocation_count as isize
    }

    fn print(&self, label: &str) {
        println!("\n{}", label);
        println!("  Allocations:      {} calls, {} bytes",
                 self.allocation_count, self.allocated_bytes);
        println!("  Deallocations:    {} calls, {} bytes",
                 self.deallocation_count, self.deallocated_bytes);
        println!("  Net allocations:  {}", self.net_allocations());
        println!("  Net heap usage:   {} bytes", self.net_bytes());
        if self.allocation_count > 0 {
            println!("  Avg alloc size:   {} bytes",
                     self.allocated_bytes / self.allocation_count);
        }
    }
}

use oxide_mvu::Effect;

#[derive(Clone, Debug)]
enum BenchEvent {
    Increment,
    Decrement,
    NoOp,
}

fn measure_effect_none(n: usize) {
    ALLOCATOR.reset();

    for _ in 0..n {
        let _effect: Effect<BenchEvent> = Effect::none();
    }

    let report = ALLOCATOR.report();
    report.print(&format!("Effect::none() - {} calls", n));
    if report.allocation_count > 0 {
        println!("  Per call:         {} allocations, {} bytes",
                 report.allocation_count / n,
                 report.allocated_bytes / n);
    }
}

fn measure_effect_just(n: usize) {
    ALLOCATOR.reset();

    for _ in 0..n {
        let _effect = Effect::just(BenchEvent::NoOp);
    }

    let report = ALLOCATOR.report();
    report.print(&format!("Effect::just() - {} calls", n));
    if report.allocation_count > 0 {
        println!("  Per call:         {} allocations, {} bytes",
                 report.allocation_count / n,
                 report.allocated_bytes / n);
    }
}

fn measure_effect_batch(n: usize, batch_size: usize) {
    ALLOCATOR.reset();

    for _ in 0..n {
        let mut effects = Vec::new();
        for i in 0..batch_size {
            effects.push(match i % 3 {
                0 => Effect::just(BenchEvent::Increment),
                1 => Effect::just(BenchEvent::Decrement),
                _ => Effect::none(),
            });
        }
        let _effect = Effect::batch(effects);
    }

    let report = ALLOCATOR.report();
    report.print(&format!("Effect::batch({}) - {} calls", batch_size, n));
    if report.allocation_count > 0 {
        println!("  Per call:         {} allocations, {} bytes",
                 report.allocation_count / n,
                 report.allocated_bytes / n);
    }
}

fn measure_effect_from_async(n: usize) {
    ALLOCATOR.reset();

    for _ in 0..n {
        let _effect: Effect<BenchEvent> = Effect::from_async(|_emitter| async {
            // Async work placeholder
        });
    }

    let report = ALLOCATOR.report();
    report.print(&format!("Effect::from_async() - {} calls", n));
    if report.allocation_count > 0 {
        println!("  Per call:         {} allocations, {} bytes",
                 report.allocation_count / n,
                 report.allocated_bytes / n);
    }
}

fn measure_mixed_workload(n: usize) {
    ALLOCATOR.reset();

    for i in 0..n {
        match i % 20 {
            0..=9 => {
                // 50% none
                let _effect: Effect<BenchEvent> = Effect::none();
            }
            10..=15 => {
                // 30% just
                let _effect = Effect::just(BenchEvent::Increment);
            }
            16..=18 => {
                // 15% batch (size 2)
                let _effect = Effect::batch(vec![
                    Effect::just(BenchEvent::Increment),
                    Effect::none(),
                ]);
            }
            _ => {
                // 5% from_async
                let _effect: Effect<BenchEvent> = Effect::from_async(|_emitter| async {});
            }
        }
    }

    let report = ALLOCATOR.report();
    report.print(&format!("Mixed workload - {} effects", n));
    println!("  Distribution:     50% none, 30% just, 15% batch(2), 5% from_async");
    if report.allocation_count > 0 {
        println!("  Per effect:       {} allocations, {} bytes",
                 report.allocation_count / n,
                 report.allocated_bytes / n);
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        Oxide-MVU Heap Allocation Benchmark                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    measure_effect_none(10_000);
    measure_effect_just(10_000);
    measure_effect_batch(5_000, 3);
    measure_effect_batch(1_000, 10);
    measure_effect_from_async(5_000);
    measure_mixed_workload(10_000);

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    Benchmark Complete                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
