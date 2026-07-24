#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ImageLoadState {
    #[default]
    Loading,
    Ready,
    Error(String),
}
