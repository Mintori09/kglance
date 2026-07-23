use kglance::core::preloader::calculate_preload_window;

#[test]
fn test_preload_window_indices() {
    let window = calculate_preload_window(0, 10);
    // current = 0 -> prev = 9, next = 1, 2, 3
    assert_eq!(window, vec![9, 1, 2, 3]);

    let window_small = calculate_preload_window(1, 3);
    // playlist len 3 -> current 1 -> prev 0, next 2
    assert!(window_small.contains(&0));
    assert!(window_small.contains(&2));
}
