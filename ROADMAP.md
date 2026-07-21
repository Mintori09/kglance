# Kglance — Roadmap phát triển

> **Mục tiêu:** Trở thành bộ đôi **QuickLook + Preview.app** của macOS dành cho KDE Plasma 6 — xem trước mọi tệp tin tức thời bằng một phím bấm duy nhất.

---

## Tầm nhìn sản phẩm

macOS sở hữu hai công cụ xem trước file cực kỳ mạnh mẽ:

| Tính năng | macOS QuickLook                      | macOS Preview.app             |
| --------- | ------------------------------------ | ----------------------------- |
| Kích hoạt | Phím Space trong Finder              | Mở file trực tiếp             |
| Tốc độ    | Tức thì (< 100ms)                    | Nhanh                         |
| Chỉnh sửa | Không (chỉ xem)                      | Có (đánh dấu, crop, xoay, ký) |
| Định dạng | Hầu hết (text, ảnh, PDF, video, ...) | Ảnh + PDF                     |
| Plugin    | Có (QLGenerator)                     | Không                         |

**Kglance** nhắm đến việc lấp đầy khoảng trống này trên KDE bằng cách cung cấp:

1. **QuickLook mode** — Bấm Space trong Dolphin để xem trước tức thì (đã có — Daemon mode)
2. **Preview mode** — Mở file với khả năng tương tác nâng cao (mục tiêu tương lai)

---

## Trạng thái hiện tại (v0.4.0)

### Đã hoàn thành

- [x] Kiến trúc Client-Daemon qua DBus (`zbus`)
- [x] Giao diện Slint với cơ chế ẩn/hiện cửa sổ (không tắt process)
- [x] Chế độ Standalone tự động fallback
- [x] Tích hợp Dolphin qua KIO Service Menu (phím Space)
- [x] Autostart daemon
- [x] Parser: Text + syntax highlighting (`syntect`)
- [x] Parser: Hình ảnh (PNG, JPEG, WebP, GIF, BMP)
- [x] Parser: SVG → PNG (`resvg`)
- [x] Parser: PDF render + page navigation (`mupdf`)
- [x] Parser: Archive tree view (ZIP, Tar, 7z) + inner preview (extract → temp → parse)
- [x] Parser: Thư mục (danh sách nội dung, sắp xếp theo cột)
- [x] Parser: Markdown rendering (pulldown-cmark + syntect + Slint StyledText)
- [x] Parser: EXIF metadata (camera, ISO, f/, ngày chụp, GPS)
- [x] Parser: Audio metadata + duration (`symphonia`)
- [x] Parser: Video info + placeholder (`ffprobe`)
- [x] Parser: Office text extraction (DOCX direct + XLSX via `calamine` + LibreOffice fallback)
- [x] Parser: Font preview + metadata (`fontdue`)
- [x] Slint UI sections cho Video, Audio, Office, Font (button placeholder + waveform container)
- [x] Dark/Light mode tự động theo KDE theme (kdeglobals → palette)
- [x] Folder sorting: click header cột Name/Kind/Modified/Size
- [x] Build tối ưu (LTO, strip, opt-level z)
- [x] Zoom in/out, pan, xoay ảnh (cuộn chuột + kéo + thanh công cụ)
- [x] EXIF sidebar (camera, GPS, focal, flash)
- [x] PDF cuộn liên tục (render all pages → một ảnh dài)
- [x] PDF thumbnail sidebar + zoom tự do
- [x] Tìm kiếm text trong file (`Ctrl+F`), copy nội dung, word wrap toggle
- [x] Multi-file điều hướng (mũi tên trái/phải chuyển file trong thư mục)
- [x] History stack: back/forward qua các file đã xem

### Hạn chế

- Video: chỉ hiển thị info + placeholder, chưa có `libmpv` embed
- Audio: chưa có waveform rendering (chỉ metadata), chưa có output device
- Office: DOCX chỉ extract text đơn giản (không format), PPTX chưa có parser trực tiếp
- Chưa có hệ thống plugin
- Chưa có icon theo loại file trong folder view
- PDF text search + text copy: chưa triển khai (chỉ cho plain text / code)
- Chưa có gallery mode / slideshow cho thư mục ảnh

---

## Lộ trình phát triển

### Phase 1 — QuickLook hoàn chỉnh (v0.2.0)

> Mục tiêu: Đạt tính năng tương đương macOS QuickLook cho các định dạng phổ biến nhất.

#### 1.1 PDF Rendering thực sự

- [x] Render trang đầu tiên của PDF thành pixmap hiển thị trong UI
- [x] Điều hướng trang (nút Previous / Next)
- [x] Hiển thị số trang hiện tại / tổng số trang
- **Crate đã dùng:** `mupdf` 0.8 — render 150 DPI, navigation buttons trên toolbar

#### 1.2 Nâng cấp giao diện

- [x] File info bar: loại file, kích thước, ngày sửa đổi
- [x] Nút "Open with..." để mở file bằng ứng dụng mặc định (`xdg-open`) (Đã có nút "Open by default")
- [x] Nút "Share" / "Copy path" (Đã tích hợp nút "Copy Path")
- [x] Hiệu ứng chuyển đổi mượt khi thay đổi file (fade in)
- [x] Đồng bộ Dark/Light mode theo KDE theme (đọc `kdeglobals` → chọn palette)
- [x] Responsive: tự co giãn cửa sổ theo kích thước nội dung (Đã tối ưu hóa qua viewport-width)

#### 1.3 Cải thiện parser hiện có

- [x] Text: hỗ trợ cuộn (scroll) cho file dài, hiển thị số dòng (Đã tích hợp cột hiển thị số dòng)
- [x] Folder: click file → preview nội dung + nút Back quay lại danh sách folder
- [x] Image: hiển thị metadata EXIF (camera, ISO, f/, ngày chụp)
- [x] Archive: cho phép xem trước file bên trong archive (click vào file → preview nội dung)
- [x] Folder: sắp xếp theo tên/kích thước/ngày (bằng cách click vào header từng cột)

---

### Phase 2 — Multimedia & Document (v0.3.0)

> Mục tiêu: Hỗ trợ media playback và các định dạng tài liệu văn phòng.

#### 2.1 Video Preview

- [x] Hiển thị thông tin video (định dạng, thời lượng)
- [x] Placeholder đen + nút play/pause + thanh seek trong Slint UI
- [x] Phát video trực tiếp trong cửa sổ preview (sử dụng `mpv` background controller)
- [x] Tự động phát khi mở, dừng khi ẩn cửa sổ
- **Đã triển khai:** `ffprobe` parse info; Tích hợp `mpv` client API để điều khiển phát video (audio qua null video output) và đồng bộ thanh tiến trình (seek, time) trực tiếp từ giao diện Slint.

#### 2.2 Audio Preview

- [x] Hiển thị metadata: tiêu đề, nghệ sĩ, album, thời lượng
- [x] Nút play/pause + waveform container trong Slint UI
- [ ] Hiển thị waveform (cần decode PCM samples từ `symphonia`)
- [ ] Phát audio (cần `rodio` hoặc `cpal` output)
- **Đã triển khai:** `symphonia` probe → metadata; UI buttons sẵn sàng

#### 2.3 Office Document Preview

- [x] DOCX: extract text từ `word/document.xml`
- [x] XLSX: đọc bảng tính qua `calamine`
- [x] Fallback: LibreOffice headless `soffice --convert-to png`
- [ ] PPTX parser direct (hiện fallback LO)
- [ ] ODF direct parser
- **Đã triển khai:** hybrid direct+LO; text content + định dạng name trong UI

#### 2.4 Markdown Rendering

- [x] Render Markdown thành rich text (heading, bold, italic, code block, link)
- [x] Hỗ trợ code blocks với syntax highlighting (syntect)
- [x] Hỗ trợ hình ảnh nhúng (extract ImageRef)
- **Đã triển khai:** `pulldown-cmark` parse events → syntect highlight code blocks → Slint `StyledText::from_markdown()`

#### 2.5 Font Preview

- [x] Hiển thị mẫu font (TTF, OTF, WOFF) với text "The quick brown fox..."
- [x] Hiển thị thông tin font: tên, glyph count, units per EM, ascender/descender, file size
- [x] Render sample text thành ảnh RGBA bằng `fontdue`
- **Đã triển khai:** `fontdue` rasterize + `image` buffer → Slint display

---

### Phase 3 — Preview Mode nâng cao (v0.4.0)

> Mục tiêu: Vượt qua QuickLook, tiến tới khả năng tương tác như Preview.app.

#### 3.1 Tương tác với hình ảnh

- [x] Zoom in/out (cuộn chuột)
- [x] Pan (kéo để di chuyển khi zoom)
- [x] Xoay ảnh (90°, 180°, 270°)
- [x] Thông tin EXIF chi tiết trong sidebar

#### 3.2 Tương tác với PDF

- [x] Cuộn liên tục qua nhiều trang
- [x] Zoom tự do
- [x] Sidebar thumbnail các trang
- [ ] Tìm kiếm text trong PDF (`Ctrl+F`)
- [ ] Copy text từ PDF

#### 3.3 Tương tác với Text/Code

- [x] Tìm kiếm trong file (`Ctrl+F`)
- [x] Copy nội dung
- [ ] Chọn syntax highlighting theme
- [x] Word wrap toggle

#### 3.4 Multi-file Preview

- [x] Xem trước nhiều file liên tiếp (mũi tên trái/phải để chuyển file trong thư mục)
- [ ] Grid view cho thư mục ảnh (gallery mode)
- [ ] Slideshow tự động cho thư mục ảnh

#### 3.5 Navigation History

- [x] History stack: back/forward qua các file đã xem
- [x] Back button cho mọi file preview (không chỉ folder): quay lại thư mục cha
- [x] Phím tắt Backspace/Escape để quay lại, Forward mouse button để tiến tới

---

### Phase 4 — Hệ sinh thái Plugin (v0.5.0)

> Mục tiêu: Cho phép cộng đồng mở rộng khả năng preview.

#### 4.1 Plugin Architecture

- [ ] Định nghĩa trait `PreviewPlugin` chuẩn hóa
- [ ] Hỗ trợ plugin dạng shared library (`.so`) load lúc runtime (`libloading`)
- [ ] Mỗi plugin đăng ký MIME types mà nó hỗ trợ
- [ ] Thứ tự ưu tiên: plugin người dùng > plugin mặc định

```rust
/// Giao diện plugin mẫu (hướng thiết kế)
pub trait PreviewPlugin: Send + Sync {
    /// Tên plugin
    fn name(&self) -> &str;
    /// Danh sách MIME type hỗ trợ
    fn supported_mimes(&self) -> &[&str];
    /// Parse file và trả về nội dung preview
    fn parse(&self, path: &std::path::Path) -> Result<PreviewContent, PluginError>;
}
```

#### 4.2 Plugin Directory

- [ ] Quét plugin từ `~/.local/share/kglance/plugins/`
- [ ] Quét plugin hệ thống từ `/usr/lib/kglance/plugins/`
- [ ] Hot-reload: phát hiện plugin mới mà không cần restart daemon

#### 4.3 Plugin SDK

- [ ] Cung cấp crate `kglance-plugin-sdk` trên crates.io
- [ ] Template project cho plugin mới
- [ ] Tài liệu hướng dẫn viết plugin

---

### Phase 5 — Tích hợp sâu Desktop (v0.6.0)

> Mục tiêu: Trở thành công dân hạng nhất của KDE Plasma.

#### 5.1 Thumbnail Provider

- [ ] Đăng ký làm KDE Thumbnail Provider cho các định dạng Kglance hỗ trợ
- [ ] Tạo thumbnail nhanh cho Dolphin grid view

#### 5.2 Freedesktop Portal

- [ ] Tích hợp `xdg-desktop-portal` cho file picker
- [ ] Hỗ trợ drag & drop từ cửa sổ preview

#### 5.3 Wayland Protocol

- [ ] Sử dụng `layer-shell` protocol để hiển thị cửa sổ preview phía trên (always on top)
- [ ] Positioning thông minh: cửa sổ preview xuất hiện gần file đang chọn trong Dolphin

#### 5.4 Notification & Status

- [ ] System tray icon khi daemon đang chạy
- [ ] Thông báo khi daemon khởi động/dừng
- [ ] Cấu hình qua file `~/.config/kglance/config.toml`

#### 5.5 Phím tắt toàn cục

- [ ] Đăng ký global shortcut qua KDE Global Shortcuts (ngoài phạm vi Dolphin)
- [x] Hỗ trợ preview file từ các file manager khác (Thunar, Nautilus, terminal) (kglance <file_name>)

---

### Phase 6 — Chất lượng phát hành (v1.0.0)

> Mục tiêu: Sẵn sàng cho người dùng cuối.

#### 6.1 Đóng gói

- [ ] Flatpak manifest
- [ ] AUR package (Arch Linux)
- [ ] `.deb` package (Debian/Ubuntu/KDE Neon)
- [ ] `.rpm` package (Fedora)
- [ ] KDE Store (KPackage format)
- [ ] AppImage

#### 6.2 Kiểm thử

- [ ] Unit test cho tất cả parser
- [ ] Integration test cho luồng DBus
- [ ] Benchmark hiệu năng (thời gian phản hồi < 10ms cho text/image)
- [ ] Test trên Wayland + X11
- [ ] Fuzzing cho các parser (file đầu vào bất thường)

#### 6.3 Tài liệu

- [ ] README song ngữ (Tiếng Việt + English)
- [ ] Trang web dự án (GitHub Pages hoặc tương tự)
- [ ] Wiki hướng dẫn cài đặt chi tiết cho từng distro
- [ ] Hướng dẫn đóng góp (CONTRIBUTING.md)
- [ ] Changelog (CHANGELOG.md)

#### 6.4 Accessibility

- [ ] Hỗ trợ screen reader (AT-SPI qua Slint)
- [ ] Keyboard navigation đầy đủ
- [ ] High contrast mode

---

## So sánh tính năng mục tiêu

| Tính năng                 | macOS QuickLook | macOS Preview | Kglance (mục tiêu) |
| ------------------------- | :-------------: | :-----------: | :----------------: |
| Xem trước tức thì (Space) |       ✅        |       —       |         ✅         |
| Text + syntax highlight   |       ✅        |       —       |         ✅         |
| Hình ảnh + EXIF           |       ✅        |      ✅       |         ✅         |
| Archive inner preview     |       ✅        |       —       |         ✅         |
| Dark/Light mode           |       ✅        |      ✅       |         ✅         |
| Folder sorting            |       ✅        |       —       |         ✅         |
| PDF render                |       ✅        |      ✅       |         ✅         |
| Video placeholder + info  |       ✅        |       —       |         ✅         |
| Audio metadata            |       ✅        |       —       |         ✅         |
| Office text extraction    |       ✅        |       —       |         ✅         |
| Markdown render           |       ✅        |       —       |         ✅         |
| Font preview              |       ✅        |       —       |         ✅         |

> - Video/Audio: metadata + play/pause placeholder buttons done; actual playback engine pending
>   | Zoom / Pan ảnh | — | ✅ | ✅ |
>   | Multi-file navigation | ✅ | ✅ | ✅ |
>   | Tìm kiếm text | — | ✅ | ✅ |
>   | Plugin system | ✅ | — | 🔲 Phase 4 |
>   | Thumbnail provider | — | — | 🔲 Phase 5 |
>   | Global shortcut | ✅ | — | 🔲 Phase 5 |
>   | Đóng gói đa distro | N/A | N/A | 🔲 Phase 6 |

---

## Nguyên tắc phát triển

1. **Hiệu năng là ưu tiên số 1** — Mọi thao tác preview phải hoàn thành dưới 10ms cho text/image. Daemon luôn thường trực trong bộ nhớ.
2. **Rust thuần túy khi có thể** — Ưu tiên crate pure-Rust để giảm dependency hệ thống và tăng tính portable.
3. **Tương thích Wayland-first** — Không phụ thuộc vào X11-specific API. Sử dụng `zbus` thay vì `xdotool`.
4. **Modular và extensible** — Mỗi parser là một module độc lập. Kiến trúc hướng tới plugin system.
5. **KDE-native** — Tôn trọng Breeze theme, KDE settings, Freedesktop standards.
6. **Đơn binary** — Toàn bộ ứng dụng biên dịch thành một file thực thi duy nhất (ngoại trừ plugin).
