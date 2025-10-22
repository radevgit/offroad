use std::mem;
use togo::prelude::*;

fn main() {
    println!("Memory Layout Analysis");
    println!("====================\n");

    // Check struct sizes
    println!("Struct sizes:");
    println!("  Point: {} bytes", mem::size_of::<Point>());
    println!("  Arc: {} bytes", mem::size_of::<Arc>());
    println!("  Vec<Arc>: {} bytes", mem::size_of::<Vec<Arc>>());
    
    // Alignment
    println!("\nAlignment:");
    println!("  Point: {} bytes", mem::align_of::<Point>());
    println!("  Arc: {} bytes", mem::align_of::<Arc>());
    
    // Create test arcs
    let arcs = vec![
        arc(point(0.0, 0.0), point(1.0, 1.0), point(0.5, 0.5), 0.7),
        arc(point(1.0, 1.0), point(2.0, 0.0), point(1.5, 0.5), 0.7),
        arc(point(2.0, 0.0), point(3.0, 1.0), point(2.5, 0.5), 0.7),
    ];

    println!("\nVector properties:");
    println!("  Capacity: {}", arcs.capacity());
    println!("  Len: {}", arcs.len());
    println!("  Total bytes: {}", arcs.capacity() * mem::size_of::<Arc>());

    // Test iteration patterns
    println!("\nIteration patterns (1M iterations):");
    
    let big_vec = vec![
        arc(point(0.0, 0.0), point(1.0, 1.0), point(0.5, 0.5), 0.7);
        1000
    ];
    
    use std::time::Instant;
    
    // Pattern 1: Direct iteration with reference
    let start = Instant::now();
    let mut sum = 0.0;
    for _ in 0..1000 {
        for a in &big_vec {
            sum += a.a.x + a.b.y + a.r;
        }
    }
    let t1 = start.elapsed();
    println!("  &big_vec iteration: {:.3}ms (sum={})", 
             t1.as_secs_f64() * 1000.0, sum);
    
    // Pattern 2: Index based
    let start = Instant::now();
    let mut sum = 0.0;
    for _ in 0..1000 {
        for i in 0..big_vec.len() {
            sum += big_vec[i].a.x + big_vec[i].b.y + big_vec[i].r;
        }
    }
    let t2 = start.elapsed();
    println!("  Index-based iteration: {:.3}ms (sum={})", 
             t2.as_secs_f64() * 1000.0, sum);
    
    // Pattern 3: Clone to owned
    let start = Instant::now();
    let mut sum = 0.0;
    for _ in 0..1000 {
        for a in big_vec.iter().cloned() {
            sum += a.a.x + a.b.y + a.r;
        }
    }
    let t3 = start.elapsed();
    println!("  Clone iteration: {:.3}ms (sum={})", 
             t3.as_secs_f64() * 1000.0, sum);

    println!("\nMemory alignment issues:");
    println!("  Arc may have padding. Check if .is_seg() or .is_arc() can be");
    println!("  inlined or if there are cache misses on field access.");
}
