use kglance::core::types::{KglanceState, ViewMode};

#[test]
fn test_kglance_state_navigation_defaults() {
    let state = KglanceState::default();
    assert!(state.playlist.is_empty());
    assert_eq!(state.current_index, 0);
    assert_eq!(state.view_mode, ViewMode::Detail);
    assert_eq!(state.cache.cap().get(), 7);
}
