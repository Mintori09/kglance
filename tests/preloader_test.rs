use kglance::core::preloader::calculate_preload_window;

#[test]
fn test_preload_window_indices() {
    let window = calculate_preload_window(0, 10, 3);
    // current = 0 -> lookahead 3 (next 1, 2, 3; prev 9, 8, 7)
    assert!(window.contains(&1));
    assert!(window.contains(&9));
    assert_eq!(window.len(), 6);

    let window_small = calculate_preload_window(1, 3, 2);
    // playlist len 3 -> current 1 -> prev 0, next 2
    assert!(window_small.contains(&0));
    assert!(window_small.contains(&2));

    // Boundary conditions:
    assert!(calculate_preload_window(0, 0, 10).is_empty());
    assert!(calculate_preload_window(0, 1, 10).is_empty());
    assert!(calculate_preload_window(0, 10, 0).is_empty());

    // Out of bounds current index wrapped gracefully
    let window_wrapped = calculate_preload_window(10, 10, 2); // 10 % 10 = 0
    assert_eq!(window_wrapped, calculate_preload_window(0, 10, 2));
}

#[test]
fn test_calculate_dynamic_lookahead_ratios() {
    use kglance::core::preloader::calculate_dynamic_lookahead;

    let max = 1000;

    // Cache < 50% capacity: lookahead 10
    assert_eq!(calculate_dynamic_lookahead(0, max), 10);
    assert_eq!(calculate_dynamic_lookahead(499, max), 10);

    // Cache between 50% and 80%: lookahead 6
    assert_eq!(calculate_dynamic_lookahead(500, max), 6);
    assert_eq!(calculate_dynamic_lookahead(799, max), 6);

    // Cache >= 80% capacity or full: lookahead 4
    assert_eq!(calculate_dynamic_lookahead(800, max), 4);
    assert_eq!(calculate_dynamic_lookahead(1000, max), 4);
    assert_eq!(calculate_dynamic_lookahead(1200, max), 4);

    // Edge case: max_bytes is 0
    assert_eq!(calculate_dynamic_lookahead(0, 0), 4);
}
