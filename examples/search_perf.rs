use std::time::Instant;

fn main() {
    println!("Detailed iteration performance analysis\n");
    
    // Simulate offset_split_arcs loop pattern
    let parts: Vec<(usize, f64)> = (0..30).map(|i| (i, i as f64)).collect();
    let index_map: Vec<usize> = (0..30).collect();
    
    println!("Pattern: {}  parts with linear search in index_map", parts.len());
    
    let start = Instant::now();
    let mut _total_searches = 0;
    
    for iteration in 0..100000 {
        // Simulate: find position by searching index_map
        let search_for = iteration % 30;
        for j_pos in 0..index_map.len() {
            if index_map[j_pos] == search_for {
                _total_searches += 1;
                break;
            }
        }
    }
    
    let t = start.elapsed();
    println!("Linear search (100k iterations): {:.3}ms", t.as_secs_f64() * 1000.0);
    println!("Average search time: {:.1}µs", t.as_secs_f64() * 1_000_000.0 / 100000.0);
    
    // Alternative: use HashMap
    use std::collections::HashMap;
    
    let start = Instant::now();
    let mut _total_searches = 0;
    
    for iteration in 0..100000 {
        let search_for = iteration % 30;
        let map: HashMap<usize, usize> = index_map.iter().enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        
        if let Some(_) = map.get(&search_for) {
            _total_searches += 1;
        }
    }
    
    let t2 = start.elapsed();
    println!("HashMap lookup (100k iterations): {:.3}ms", t2.as_secs_f64() * 1000.0);
    println!("Average lookup time: {:.1}µs", t2.as_secs_f64() * 1_000_000.0 / 100000.0);
    
    // Better: build HashMap once
    let start = Instant::now();
    let mut _total_searches = 0;
    
    let map: HashMap<usize, usize> = index_map.iter().enumerate()
        .map(|(i, &v)| (v, i))
        .collect();
    
    for iteration in 0..100000 {
        let search_for = iteration % 30;
        if let Some(_) = map.get(&search_for) {
            _total_searches += 1;
        }
    }
    
    let t3 = start.elapsed();
    println!("HashMap (built once): {:.3}ms", t3.as_secs_f64() * 1000.0);
    println!("Average lookup time: {:.1}µs", t3.as_secs_f64() * 1_000_000.0 / 100000.0);
    
    println!("\nConclusion:");
    println!("  Linear search is actually very fast for small vectors (30 items)");
    println!("  HashMap overhead may not be worth it");
    println!("  But: code does this search MANY times - could add up");
}
