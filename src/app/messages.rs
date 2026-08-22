use crate::core::PreviewData;

#[derive(Debug, Clone)]
pub enum NavigationMsg {
    PrevFileClicked,
    NextFileClicked,
    HistoryBack,
    HistoryForward,
    SiblingFilesLoaded(Vec<String>),
    ToggleViewMode,
    FileClickedInGrid(usize),
    GridThumbnailLoaded {
        index: usize,
        handle: Option<iced::widget::image::Handle>,
    },
    PreloadCompleted {
        path: String,
        content: std::sync::Arc<PreviewData>,
    },
    ToggleSettingsClicked,
    FileClicked(usize),
}

#[derive(Debug, Clone)]
pub enum ImageMsg {
    Zoom(f32),
    PanDelta(f32, f32),
    DoubleClick,
}

#[derive(Debug, Clone)]
pub enum TextMsg {
    Edit(iced::widget::text_editor::Action),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchClosed,
    WrapToggled,
    Scrolled(f32),
    ToggleOutline,
    SymbolClicked(usize),
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
    VideoThumbnailLoaded { data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub enum PdfMsg {
    PagesLoaded(Vec<Option<(Vec<u8>, u32, u32)>>),
    PageReady(usize, Vec<u8>, u32, u32),
    ThumbReady(usize, Vec<u8>, u32, u32),
    Scrolled(iced::widget::scrollable::Viewport),
    SidebarScrolled(iced::widget::scrollable::Viewport),
    SidebarToggled,
    SetSidebarMode(crate::core::PdfSidebarMode),
    ThumbnailClicked(usize),
    TocItemClicked(usize),
    SidebarResized(f32),
}

#[derive(Debug, Clone)]
pub enum TypstMsg {
    PagesLoaded,
    PageReady(usize, Vec<u8>, u32, u32),
    Scrolled(iced::widget::scrollable::Viewport),
    CompileError,
    ToggleSource,
}

#[derive(Debug, Clone)]
pub enum SpreadsheetMsg {
    SheetTabClicked(usize),
    ColumnClicked(usize),
    SearchQueryChanged(String),
    SearchClosed,
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
    },
    SearchToggle,
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchClosed,
    MermaidBlockRendered {
        index: usize,
        png_bytes: Option<Vec<u8>>,
    },
    ImageLoaded {
        index: usize,
        png_bytes: Option<Vec<u8>>,
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
    Scrolled(f32),
    RawEdit(iced::widget::text_editor::Action),
    SearchToggle,
    SearchQueryChanged(String),
    SearchClosed,
    ExpandAll,
    CollapseAll,
    CopyPath(usize),
    NodeClicked(usize),
    BreadcrumbClicked(usize),
    ToggleFormat,
    EditStart(usize),
    EditValue(String),
    EditSave,
    EditCancel,
    SchemaToggle,
}

#[derive(Debug, Clone)]
pub enum ActionMsg {
    OpenClicked,
    CopyPathClicked,
    BackClicked,
    CloseRequested,
    CopyContentClicked,
    CopyCode(String),
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
    FileLoaded {
        path: String,
        content: PreviewData,
    },
    WindowEvent(iced::window::Id, iced::window::Event),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    ToastDismissed(u64),
    OpenLink(String),
    FilePreviewError(String),
    FileChanged(String),
    WindowResized(f32, f32),
    ReadPositionsTick,
}

#[derive(Debug, Clone)]
pub enum SettingsMsg {
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
    WordWrapChanged(bool),
    JsonTreeViewChanged(bool),
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
    Media(MediaMsg),
    Pdf(PdfMsg),
    Typst(TypstMsg),
    Spreadsheet(SpreadsheetMsg),
    Grid(GridMsg),
    Markdown(MarkdownMsg),
    Epub(EpubMsg),
    Json(JsonMsg),
    Settings(SettingsMsg),

    // Layout / Global Events
    ScrollDelta { x: f32, y: f32 },
    CtrlHeldChanged(bool),
    ShiftHeldChanged(bool),
    ModifiersUpdated(iced::keyboard::Modifiers),
    SortByFieldClicked(crate::core::SortField),

    // Sidebar resize / Mouse drag (shared by Markdown & Epub)
    SidebarDragStarted,
    SidebarDragEnded,
    MouseMoved(f32, f32),
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
impl From<SettingsMsg> for Message {
    fn from(m: SettingsMsg) -> Self {
        Message::Settings(m)
    }
}
