use crate::core::PreviewData;

#[derive(Debug, Clone)]
pub enum NavigationMsg {
    PrevFileClicked,
    NextFileClicked,
    HistoryBack,
    HistoryForward,
    SiblingFilesLoaded(Vec<String>),
    DirectorySyncCompleted {
        dir: std::path::PathBuf,
        files: Vec<String>,
        generation_id: usize,
    },
    ToggleViewMode,
    FileClickedInGrid(usize),
    GridThumbnailLoaded {
        path: String,
        handle: Option<iced::widget::image::Handle>,
    },
    PreloadCompleted {
        path: String,
        content: std::sync::Arc<PreviewData>,
        decoded_cache: Option<crate::core::CachedContent>,
    },
    ToggleSettingsClicked,
    FileClicked(usize),
    FolderScrolled(iced::widget::scrollable::Viewport),
    OpenPath(String),
}

#[derive(Debug, Clone)]
pub enum FolderMsg {
    SearchQueryChanged(String),
    SearchClosed,
}

#[derive(Debug, Clone)]
pub enum ImageMsg {
    Zoom {
        factor: f32,
        cursor: iced::Point,
    },
    PanDelta(f32, f32),
    DoubleClick,
    RotateRight,
    RotateLeft,
    FlipHorizontal,
    FlipVertical,
    FitToWindow,
    ToggleInfo,
    CloseInfo,
    Decoded {
        load_id: u64,
        handle: iced::widget::image::Handle,
        cached_preview: Option<(iced::widget::image::Handle, u32, u32)>,
        width: u32,
        height: u32,
    },
    AnimationTick,
}

#[derive(Debug, Clone)]
pub enum TextMsg {
    SelectionChanged(Option<crate::ui::components::code_viewer::SelectionRange>),
    SelectionDragStarted(crate::ui::components::code_viewer::TextPosition),
    SelectionDragEnded,
    AutoScroll(Option<f32>, iced::Point),
    AutoScrollTick,
    TokensReady(crate::features::text::IndexResult),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchClosed,
    WrapToggled,
    Scrolled(iced::widget::scrollable::Viewport),
    WheelScrolled(iced::mouse::ScrollDelta),
    ToggleOutline,
    SymbolClicked(usize),
    GotoLineToggle,
    GotoLineQueryChanged(String),
    GotoLineSubmitted,
    GotoLineClosed,
}

#[derive(Debug, Clone)]
pub enum AudioMsg {
    PlayPauseClicked,
    SeekClicked(f32),
    SeekRelativeClicked(f32),
    VolumeChanged(f32),
    ToggleMuteClicked,
    Tick,
    EndOfStream,
}

#[derive(Debug, Clone)]
pub enum MediaMsg {
    PlayPauseClicked,
    SeekClicked(f32),
    SeekRelativeClicked(f32),
    VideoNewFrame,
    VideoEndOfStream,

    MouseEnter,
    MouseLeave,
    VideoThumbnailLoaded {
        data: Vec<u8>,
    },
    ToggleSubtitles,
    CycleSubtitleTrack,
    SubtitlesLoaded {
        tracks: Vec<crate::features::video::SubtitleTrack>,
    },
}

#[derive(Debug, Clone)]
pub enum PdfMsg {
    PagesLoaded(Vec<Option<(Vec<u8>, u32, u32)>>),
    PageReady(
        usize,
        Vec<u8>,
        u32,
        u32,
        Option<crate::features::pdf::PdfPageText>,
    ),
    ThumbReady(usize, Vec<u8>, u32, u32),
    Scrolled(iced::widget::scrollable::Viewport),
    SidebarScrolled(iced::widget::scrollable::Viewport),
    SidebarToggled,
    SetSidebarMode(crate::core::PdfSidebarMode),
    ThumbnailClicked(usize),
    TocItemClicked(usize),
    SidebarResized(f32),
    WheelScrolled(iced::mouse::ScrollDelta),
    SelectionDragStart(crate::features::pdf::PdfPosition),
    SelectionDragUpdate(crate::features::pdf::PdfPosition),
    SelectionDragEnd,
    SelectionClear,
    AutoScrollTick,
}

#[derive(Debug, Clone)]
pub enum TypstMsg {
    PagesLoaded,
    PageReady(
        usize,
        Vec<u8>,
        u32,
        u32,
        Option<crate::features::pdf::PdfPageText>,
    ),
    Scrolled(iced::widget::scrollable::Viewport),
    CompileError,
    ToggleSource,
    SelectionChanged(Option<crate::ui::components::code_viewer::SelectionRange>),
    SourceScrolled(iced::widget::scrollable::Viewport),
    SourceWheelScrolled(iced::mouse::ScrollDelta),
}

#[derive(Debug, Clone)]
pub enum SpreadsheetMsg {
    SheetTabClicked(usize),
    ColumnClicked(usize),
    CellPressed { row: usize, col: usize },
    CellEntered { row: usize, col: usize },
    CellReleased,
    RowHeaderPressed(usize),
    RowHeaderEntered(usize),
    AutoScrollTick,
    CopySelection,
    SearchQueryChanged(String),
    SearchClosed,
    Scrolled(iced::widget::scrollable::Viewport),
    WheelScrolled(iced::mouse::ScrollDelta),
}

#[derive(Debug, Clone)]
pub enum GridMsg {
    SearchQueryChanged(String),
    SearchClosed,
}

#[derive(Debug, Clone)]
pub enum MarkdownMsg {
    TocToggled,
    TocHeadingClicked(usize),
    TocToggleCollapse(usize),
    Scrolled {
        y: f32,
        viewport_height: f32,
        content_height: f32,
    },
    SearchToggle,
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchClosed,
    MermaidBlockRendered {
        generation_id: usize,
        index: usize,
        png_bytes: Option<Vec<u8>>,
    },
    ImageLoaded {
        generation_id: usize,
        path: String,
        payload: Option<crate::features::image::DecodedImagePayload>,
    },
    SidebarResized(f32),
    SelectionChanged(Option<String>),
    SelectionDragStart {
        block: usize,
        offset: usize,
    },
    SelectionDragUpdate {
        block: usize,
        offset: usize,
    },
    SelectionDragEnd,
    SelectionClear,
    AutoScrollTick,
    SmoothWheelScrolled(iced::mouse::ScrollDelta),
    BlockMeasured {
        block_index: usize,
        height: f32,
    },
    TouchpadGestureEnded(std::time::Instant),
    AnimationTick,
}

#[derive(Debug, Clone)]
pub enum EpubMsg {
    SidebarToggled,
    ChapterClicked(usize),
    ChapterToggleCollapse(usize),
    SidebarResized(f32),
}

#[derive(Debug, Clone)]
pub enum JsonMsg {
    ToggleMode,
    ToggleNode(usize),
    Scrolled(iced::widget::scrollable::Viewport),
    WheelScrolled(iced::mouse::ScrollDelta),
    RawScrolled(iced::widget::scrollable::Viewport),
    RawWheelScrolled(iced::mouse::ScrollDelta),
    RawSelectionChanged(Option<crate::ui::components::code_viewer::SelectionRange>),
    SearchToggle,
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchClosed,
    ExpandAll,
    CollapseAll,
    CopyPath(usize),
    CopyValue(usize),
    CopyKey(usize),
    CopySubtree(usize),
    NodeClicked(usize),
    BreadcrumbClicked(usize),
    ToggleFormat,
}

#[derive(Debug, Clone)]
pub enum ActionMsg {
    OpenClicked,
    CopyPathClicked,
    BackClicked,
    CloseRequested,
    CopyText(String),
}

#[derive(Debug, Clone)]
pub enum SystemMsg {
    ThemeToggled,
    DaemonOpenWindow {
        path: String,
    },
    DaemonOpenWithPlaylist {
        path: String,
        content: PreviewData,
        playlist: Vec<String>,
    },
    DaemonUpdateWindow {
        path: String,
        content: PreviewData,
    },
    DaemonUpdateWithPlaylist {
        path: String,
        content: PreviewData,
        playlist: Vec<String>,
    },
    DaemonOpenLoading {
        path: String,
        playlist: Vec<String>,
    },
    DaemonUpdateLoading {
        path: String,
        playlist: Vec<String>,
    },
    FileLoaded {
        path: String,
        content: PreviewData,
        generation_id: usize,
    },
    WindowEvent(iced::window::Id, iced::window::Event),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    ToastDismissed(u64),
    OpenLink(String),
    FilePreviewError(String),
    ActiveFileModified(String),
    ActiveFileDeleted(String),
    DirectoryChanged(std::path::PathBuf),
    WindowResized(f32, f32),
    ReadPositionsTick,
    SpinnerTick,
}

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    TabChanged(crate::core::SettingTab),
    ThemeChanged(String),
    FontSizeChanged(f32),
    FontFamilySelected(String),
    FontFamilyMonoSelected(String),
    EpubFontFamilySelected(String),
    MaxTextWidthChanged(Option<f32>),
    DefaultWidthChanged(u32),
    DefaultHeightChanged(u32),
    MinWidthChanged(u32),
    MinHeightChanged(u32),
    UseCurrentWindowSize,
    SetWindowPreset { width: u32, height: u32 },
    WordWrapChanged(bool),
    JsonTreeViewChanged(bool),
    RememberWindowSizeChanged(bool),
    EpubReadingModeChanged(crate::core::config::EpubReadingMode),
    RubyDisplayModeChanged(crate::core::config::RubyDisplayMode),
    PreferMermaidCliChanged(bool),
    MaxMemoryMbChanged(usize),
    MaxDiskCacheMbChanged(usize),
    SmoothScrollChanged(bool),
    ScrollFrictionChanged(f32),
    ScrollSpringStiffnessChanged(f32),
    SelectionMenuToggled(bool),
    CustomAction(CustomActionEditorMsg),
    TrashEnabledChanged(bool),
    TrashConfirmOnDeleteChanged(bool),
    TrashUndoTimeoutChanged(u64),
    KeybindingTrashFileChanged(String),
    KeybindingUndoTrashChanged(String),
    ResetToDefaults,
}

#[derive(Debug, Clone)]
pub enum TrashMsg {
    RequestTrashCurrent,
    ConfirmTrashCurrent,
    CancelTrashDialog,
    TrashCompleted {
        file_to_trash: String,
        result: Result<crate::core::trash::TrashedItem, String>,
    },
    UndoTrash,
    RestoreCompleted {
        restored_item: crate::core::trash::TrashedItem,
        result: Result<(), String>,
    },
    ToastActionClicked(crate::core::types::ToastAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomActionEditorMsg {
    OpenAdd,
    OpenEdit(usize),
    NameChanged(String),
    CommandChanged(String),
    PopupToggled(bool),
    ToastToggled(bool),
    StreamToggled(bool),
    Save,
    Cancel,
    Delete(usize),
    ToggleEnabled { index: usize, enabled: bool },
}

#[derive(Debug, Clone)]
pub enum SelectionMenuMsg {
    Show {
        pos: iced::Point,
        text: String,
    },
    Dismiss,
    CopyClicked(String),
    CustomActionClicked {
        action: crate::core::selection_action::CustomAction,
        text: String,
    },
    ActionResult {
        action_name: String,
        result: Result<String, String>,
        popup: bool,
        toast: bool,
    },
    StreamChunk {
        action_name: String,
        chunk: String,
    },
    StreamFinished {
        action_name: String,
        toast: bool,
    },
    StreamFailed {
        action_name: String,
        error: String,
        toast: bool,
    },
    ResultSelectionChanged(Option<String>),
    ResultDragStart {
        block: usize,
        offset: usize,
    },
    ResultDragUpdate {
        block: usize,
        offset: usize,
    },
    ResultDragEnd,
    ResultSelectionClear,
    ResultAutoScrollTick,
}

#[derive(Debug, Clone)]
pub enum FileTreeMsg {
    Toggle,
    ToggleFolder(std::path::PathBuf),
    SelectFile(std::path::PathBuf),
    CollapseAll,
}

#[derive(Debug, Clone)]
pub enum Model3dMsg {
    Rotate { delta_yaw: f32, delta_pitch: f32 },
    Zoom(f32),
    DragStarted(iced::Point),
    DragMoved(iced::Point),
    DragEnded,
    ResetCamera,
    ToggleShading,
    ToggleAutoRotate,
    ToggleGrid,
    Tick,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    // Action & System
    Action(ActionMsg),
    System(SystemMsg),
    // Sub-domain messages
    Navigation(NavigationMsg),
    Image(ImageMsg),
    Text(TextMsg),
    Audio(AudioMsg),
    Media(MediaMsg),
    Pdf(PdfMsg),
    Typst(TypstMsg),
    Spreadsheet(SpreadsheetMsg),
    Grid(GridMsg),
    Markdown(MarkdownMsg),
    Epub(EpubMsg),
    Json(JsonMsg),
    Model3d(Model3dMsg),
    Settings(SettingsMsg),
    FileTree(FileTreeMsg),
    SelectionMenu(SelectionMenuMsg),
    Trash(TrashMsg),
    Folder(FolderMsg),

    // Layout / Global Events
    SmoothScrollTick(std::time::Instant),
    ScrollDelta { x: f32, y: f32 },
    CtrlHeldChanged(bool),
    ShiftHeldChanged(bool),
    ModifiersUpdated(iced::keyboard::Modifiers),
    SortByFieldClicked(crate::core::SortField),

    // Sidebar resize / Mouse drag (shared by Markdown & Epub)
    SidebarDragStarted,
    SidebarDragEnded,
    MouseMoved(f32, f32),
    MousePressed(f32, f32),
    MouseReleased,
}

impl From<ActionMsg> for Message {
    fn from(m: ActionMsg) -> Self {
        Message::Action(m)
    }
}
impl From<SystemMsg> for Message {
    fn from(m: SystemMsg) -> Self {
        Message::System(m)
    }
}
impl From<NavigationMsg> for Message {
    fn from(m: NavigationMsg) -> Self {
        Message::Navigation(m)
    }
}
impl From<ImageMsg> for Message {
    fn from(m: ImageMsg) -> Self {
        Message::Image(m)
    }
}
impl From<TextMsg> for Message {
    fn from(m: TextMsg) -> Self {
        Message::Text(m)
    }
}
impl From<AudioMsg> for Message {
    fn from(m: AudioMsg) -> Self {
        Message::Audio(m)
    }
}
impl From<MediaMsg> for Message {
    fn from(m: MediaMsg) -> Self {
        Message::Media(m)
    }
}
impl From<PdfMsg> for Message {
    fn from(m: PdfMsg) -> Self {
        Message::Pdf(m)
    }
}
impl From<TypstMsg> for Message {
    fn from(m: TypstMsg) -> Self {
        Message::Typst(m)
    }
}
impl From<SpreadsheetMsg> for Message {
    fn from(m: SpreadsheetMsg) -> Self {
        Message::Spreadsheet(m)
    }
}
impl From<GridMsg> for Message {
    fn from(m: GridMsg) -> Self {
        Message::Grid(m)
    }
}
impl From<MarkdownMsg> for Message {
    fn from(m: MarkdownMsg) -> Self {
        Message::Markdown(m)
    }
}
impl From<EpubMsg> for Message {
    fn from(m: EpubMsg) -> Self {
        Message::Epub(m)
    }
}
impl From<JsonMsg> for Message {
    fn from(m: JsonMsg) -> Self {
        Message::Json(m)
    }
}
impl From<Model3dMsg> for Message {
    fn from(m: Model3dMsg) -> Self {
        Message::Model3d(m)
    }
}
impl From<SettingsMsg> for Message {
    fn from(m: SettingsMsg) -> Self {
        Message::Settings(m)
    }
}
impl From<FileTreeMsg> for Message {
    fn from(m: FileTreeMsg) -> Self {
        Message::FileTree(m)
    }
}
impl From<SelectionMenuMsg> for Message {
    fn from(m: SelectionMenuMsg) -> Self {
        Message::SelectionMenu(m)
    }
}
impl From<TrashMsg> for Message {
    fn from(m: TrashMsg) -> Self {
        Message::Trash(m)
    }
}
impl From<FolderMsg> for Message {
    fn from(m: FolderMsg) -> Self {
        Message::Folder(m)
    }
}
