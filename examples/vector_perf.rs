// Performance analysis: index_map lookup patterns
// 
// Current code (inefficient):
//   for each candidate orig_idx:
//     for each j_pos in 0..index_map.len():
//       if index_map[j_pos] == orig_idx: break
//   
// This is O(m * n) where m = candidates, n = current parts count
//
// Optimization ideas:
//
// 1. REVERSE MAPPING DURING ITERATION
//    Create HashMap<orig_idx, j_pos> for O(1) lookup
//    But HashMap overhead may not be worth it for small n (~17-30 parts)
//
// 2. VECTOR DEQUE for faster removal
//    Current: parts.remove(j_current) is O(n) because it shifts all elements
//    Solution: Use VecDeque, remove() from middle is O(n) anyway
//             But pop_back() is O(1) - could restructure algorithm
//
// 3. DIRECT INDEX TRACKING
//    Instead of searching index_map, maintain it as mirror:
//    parts[i] corresponds to index_map[i]
//    When removing j_current, both vectors are synchronized
//
// 4. AVOID CLONING LARGE ARCS
//    Line 84: let part1 = parts[j_pos].clone();
//    Arc is 64 bytes - cloning is not cheap
//    Consider taking reference instead, pass by ref to split functions
//
// KEY INSIGHT: The real bottleneck is likely:
// - Searching index_map O(n) for each candidate
// - Large Arc clones (64 bytes each)
// - Remove operations from middle of vector O(n)

fn main() {
    println!("Vector remove performance test");
    
    use std::time::Instant;
    
    // Benchmark: removing from middle of vector
    let v: Vec<usize> = (0..1000).collect();
    
    let start = Instant::now();
    for _ in 0..10000 {
        let mut v = v.clone();
        if v.len() > 500 {
            v.remove(500);  // Remove from middle
        }
    }
    let t = start.elapsed();
    println!("Remove from middle (10k x 1000-item vec): {:.3}ms", t.as_secs_f64() * 1000.0);
    
    // Benchmark: searching vector for value
    let v: Vec<usize> = (0..1000).collect();
    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _ = v.iter().position(|&x| x == 500);
    }
    let t = start.elapsed();
    println!("Linear search 1M times: {:.3}ms", t.as_secs_f64() * 1000.0);
}
