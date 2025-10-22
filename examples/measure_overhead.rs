use std::time::Instant;

fn main() {
    println!("Measuring AABB creation and spatial index overhead");
    println!("===================================================\n");

    // Simulate small vector iteration like in offset_split_arcs
    let iterations = 10000;
    let vec_sizes = vec![5, 10, 15, 20, 30];

    for size in vec_sizes {
        // Baseline: just iterating and cloning
        let data = vec![(0usize, 0usize, 0.0, 1.0, 0.0, 1.0); size];
        let start = Instant::now();
        for _ in 0..iterations {
            for (idx, item) in data.iter().enumerate() {
                let _ = idx;
                let _ = item.clone();
            }
        }
        let baseline = start.elapsed();

        // With spatial index creation (empty BroadPhaseFlat simulation)
        let start = Instant::now();
        for _ in 0..iterations {
            let mut spatial = Vec::new();
            for (idx, item) in data.iter().enumerate() {
                spatial.push((idx, item.clone()));
            }
            let _ = spatial.len();
        }
        let with_spatial = start.elapsed();

        let overhead = (with_spatial.as_secs_f64() / baseline.as_secs_f64() - 1.0) * 100.0;
        println!("Size {}: baseline {:.2}ms, with spatial {:.2}ms, overhead {:.1}%",
                 size, 
                 baseline.as_secs_f64() * 1000.0,
                 with_spatial.as_secs_f64() * 1000.0,
                 overhead);
    }

    println!("\nKey findings:");
    println!("- Spatial index overhead is ONLY worthwhile if it reduces true pair tests significantly");
    println!("- For small vectors (< 20 items), iterating all is likely faster");
    println!("- Current implementation rebuilds spatial index on EVERY loop iteration!");
}
