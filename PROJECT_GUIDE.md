# Hướng dẫn Tìm hiểu Dự án Kglance (Oxiview) cho Người mới Bắt đầu

Chào mừng bạn đến với dự án **Kglance** (còn gọi là Oxiview). Đây là hệ thống xem trước tệp tin (Quick Preview) hiệu năng cao được thiết kế dành riêng cho môi trường KDE Plasma 6. Dự án được phát triển bằng ngôn ngữ Rust, sử dụng giao diện đồ họa **Iced** và cơ chế giao tiếp liên tiến trình qua **DBus**.

Tài liệu này được thiết kế đặc biệt dành cho bạn: người đã có kiến thức nền tảng về **Rust** nhưng bắt đầu từ **con số 0 đối với Iced và dự án này**.

---

## 1. Tổng quan Kiến trúc Client-Daemon

Để đạt được mục tiêu hiển thị xem trước gần như tức thời ($< 10\text{ms}$) và tiết kiệm tài nguyên RAM, Kglance hoạt động theo mô hình **Client-Daemon** thông qua DBus:

1. **Daemon Mode (`kglance daemon`):**
   - Chạy nền ngay khi bạn đăng nhập vào hệ thống.
   - Luôn giữ một cửa sổ giao diện ẩn trong bộ nhớ.
   - Lắng nghe yêu cầu xem trước file từ DBus. Khi nhận được yêu cầu, nó sẽ hiển thị cửa sổ, lấy dữ liệu đã parse và vẽ lên màn hình ngay lập tức.
2. **Standalone Mode (`kglance <file>`):**
   - Chạy như một ứng dụng độc lập bình thường để hiển thị một file cụ thể.
   - Khi chạy, nó sẽ cố gắng gửi yêu cầu qua DBus tới Daemon trước. Nếu Daemon chưa chạy, nó tự động fallback sang chế độ độc lập (tự khởi tạo cửa sổ mới và đóng hoàn toàn khi bạn tắt).

---

## 2. Cấu trúc Thư mục và Các File Quan trọng

Dưới đây là cấu trúc thư mục của Kglance và vai trò của từng thành phần:

```text
Kglance/
├── Cargo.toml               # Quản lý dependencies (iced, zbus, syntect, image, etc.)
├── AGENTS.md                # Quy tắc phát triển và hướng dẫn dành cho AI Agent
├── ARCHITECTURE.md          # Chi tiết thiết kế kiến trúc hệ thống
├── PERFORMANCE.md           # Hướng dẫn tối ưu hóa hiệu năng và kết quả benchmark
├── Justfile                 # Định nghĩa các lệnh build, chạy nhanh (thay thế cho Makefile)
└── src/
    ├── main.rs              # Entry point của ứng dụng (điều hướng daemon / standalone / client)
    ├── lib.rs               # Khai báo thư viện dùng chung
    ├── app/                 # State chính, Message enum, update() dispatcher và view()
    │   ├── mod.rs           # KglanceApp struct, update() dispatcher, new(), view()
    │   ├── keyboard.rs      # Xử lý phím tắt (KeyPressed, modifiers)
    │   ├── window.rs        # Quản lý cửa sổ (open/close/focus/resize)
    │   └── media.rs         # Điều khiển media (play/pause/seek, scroll, zoom)
    ├── logger.rs            # Định nghĩa các macro ghi log (log_info, log_error)
    ├── core/                # Business logic lõi độc lập với giao diện (Clean Architecture)
    │   ├── mod.rs
    │   ├── preview.rs       # PreviewData enum, populate_state(), trait FilePreviewer
    │   ├── types.rs         # KglanceState và các sub-state
    │   └── handlers/        # Xử lý nền (background tasks)
    │       ├── mod.rs
    │       └── pdf.rs       # Lazy-load stream cho render PDF nhiều trang
    ├── dbus/                # Module giao tiếp IPC qua DBus (sử dụng zbus)
    │   ├── mod.rs           # Client API (gửi tín hiệu xem trước)
    │   ├── recipe.rs        # DaemonRecipe: subscription recipe nhận command từ DBus
    │   └── service.rs       # Daemon API (lắng nghe tín hiệu và đẩy command về Iced)
    ├── parsers/             # Các bộ trích xuất dữ liệu từ file vật lý thành dữ liệu xem trước
    │   ├── mod.rs           # Registry tự động chọn parser phù hợp dựa trên MIME type
    │   ├── text.rs          # Đọc văn bản thường
    │   ├── markdown.rs      # Phân tích cú pháp Markdown và vẽ khối văn bản
    │   ├── image.rs         # Đọc ảnh (PNG, JPG, WebP)
    │   ├── svg.rs           # Đọc và render SVG bằng resvg
    │   ├── pdf.rs           # Render PDF (trang đầu tiên) bằng poppler/pdf-rs
    │   ├── archive.rs       # Đọc danh sách file trong file nén (.zip, .7z) không cần giải nén
    │   ├── folder.rs        # Liệt kê các file trong thư mục
    │   ├── spreadsheet.rs   # Đọc bảng tính (Excel/CSV)
    │   ├── video.rs / audio.rs # Trích xuất thông tin media
    │   └── ...
    └── ui/                  # Tầng hiển thị giao diện người dùng (Iced UI Components)
        ├── mod.rs
        ├── window.rs        # Layout khung cửa sổ lớn
        ├── handlers/        # Xử lý media bất đồng bộ
        │   ├── mod.rs
        │   ├── image.rs     # Tính toán kích thước cửa sổ cho ảnh
        │   └── video.rs     # Video player + VideoRecipe subscription recipe
        ├── theme/           # Định nghĩa màu sắc, phong cách giao diện (Theme Glass)
        └── views/           # UI con hiển thị cụ thể cho từng loại dữ liệu (image_view, text_view, pdf_view, etc.)
```

---

## 3. Dành cho Người mới: Hiểu về Iced & Kiến trúc Elm (TEA)

**Iced** là một GUI library viết bằng Rust lấy cảm hứng từ **The Elm Architecture (TEA)**. Nó hoạt động dựa trên vòng lặp dữ liệu một chiều (one-way data flow) chia làm 4 phần chính:

1. **State (Trạng thái):** Struct lưu giữ toàn bộ dữ liệu cấu thành giao diện của ứng dụng tại thời điểm hiện tại.
2. **Message (Thông điệp):** Một `enum` đại diện cho mọi tương tác từ người dùng (click chuột, gõ phím) hoặc sự kiện từ hệ thống (file đã tải xong, sự kiện từ DBus).
3. **Update (Cập nhật):** Một hàm nhận vào trạng thái hiện tại (`&mut self`) và một `Message`, thực hiện thay đổi giá trị trong `State` và tùy chọn trả về một `Task` (tác vụ bất đồng bộ như đọc file, render ảnh).
4. **View (Hiển thị):** Một hàm nhận vào trạng thái hiện tại (`&self`) và xây dựng cây widget (nút bấm, ô nhập liệu, hình ảnh) để Iced vẽ lên màn hình. Hàm này trả về một `Element<'_, Message>`.

### Áp dụng TEA trong Kglance:
Nhìn vào [src/app/mod.rs](file:///home/mintori/Projects/temp/Kglance/src/app/mod.rs):
- **State**: Đại diện bởi `KglanceApp` (chứa `KglanceState`).
- **Message**: Đại diện bởi enum `Message`.
- **Update**: Hàm `KglanceApp::update` là bộ điều phối, delegate xử lý cụ thể sang các module con:
  - `app/keyboard.rs` — phím tắt, modifiers
  - `app/window.rs` — mở/đóng cửa sổ
  - `app/media.rs` — điều khiển video, scroll, zoom
- **View**: Hàm `KglanceApp::view` trả về cây Widgets.
- **Subscription**: Lắng nghe sự kiện từ DBus (`dbus/recipe.rs`) và video (`ui/handlers/video.rs`) qua các `Recipe` riêng biệt.

---

## 4. Cách Quản lý Trạng thái (State Management)

Toàn bộ thông tin trạng thái được tập trung tại struct `KglanceState` định nghĩa trong [src/core/types.rs](file:///home/mintori/Projects/temp/Kglance/src/core/types.rs).

Khi một file được chọn để xem trước, quá trình sau sẽ xảy ra:
1. Một `Task` bất đồng bộ gọi hàm `FilePreviewer::parse()` của parser tương ứng để trả về `PreviewData`.
2. Khi hoàn thành, `Message::FileLoaded` được kích hoạt.
3. Trong `handle_file_loaded` (định nghĩa tại `app/mod.rs`), ứng dụng gọi `PreviewData::populate_state()` (định nghĩa tại [core/preview.rs](file:///home/mintori/Projects/temp/Kglance/src/core/preview.rs)) để ánh xạ `PreviewData` sang `KglanceState`:
   - **`ImageState`**: Zoom, pan, EXIF metadata của ảnh.
   - **`TextState`**: Cấu trúc soạn thảo/hiển thị text, chế độ wrap dòng, trạng thái tìm kiếm (từ khóa, vị trí khớp).
   - **`PdfState`**: Số lượng trang, thumbnail cached, trang hiện tại.
   - **`TableState`**: Danh sách file khi xem thư mục hoặc file nén (hỗ trợ sort theo tên, dung lượng, ngày tháng).
   - **`MarkdownState`**: Chứa danh sách các khối Markdown đã phân tích và handle ảnh Mermaid được render bất đồng bộ.
   - **`SpreadsheetState`**: Dữ liệu bảng tính Excel/CSV chia theo từng sheet tab.
   - **`MediaState`**: Trạng thái phát (Play/Pause), thời gian (Time), tiến trình chạy (Progress) của Audio/Video.

---

## 5. Quản lý Phím tắt (Shortcuts) ở đâu?

Toàn bộ sự kiện phím bấm từ bàn phím được Iced gửi về ứng dụng thông qua cơ chế `subscription` lắng nghe sự kiện hệ thống. Các sự kiện này được ánh xạ thành `Message::KeyPressed(key, modifiers)` và được xử lý tập trung tại `KglanceApp::handle_key_pressed()` trong [app/keyboard.rs](file:///home/mintori/Projects/temp/Kglance/src/app/keyboard.rs).

Dưới đây là các phím tắt hiện có trong dự án:

### Phím tắt Chung (Mọi loại tệp):
- `Space` / `Escape` / `Backspace`: Ẩn cửa sổ xem trước (nếu đang chạy chế độ Daemon) hoặc Thoát ứng dụng (nếu đang ở Standalone mode).
- `Ctrl + C`: Sao chép đường dẫn tuyệt đối của tệp tin đang xem trước vào Clipboard hệ thống.

### Phím tắt cho Xem trước Ảnh (Image Viewer):
- `Ctrl + =` hoặc `Ctrl + +`: Phóng to hình ảnh (tối đa 10x).
- `Ctrl + -`: Thu nhỏ hình ảnh (tối thiểu 0.1x).
- `Ctrl + 0`: Đặt lại Zoom về mặc định (1.0x).

### Phím tắt cho Xem trước Văn bản (Text Viewer):
- `ArrowDown` / `j`: Cuộn nội dung xuống dưới.
- `ArrowUp` / `k`: Cuộn nội dung lên trên.

### Phím tắt cho Thư mục & File Nén (Folder/Archive Table):
Khi xem trước một thư mục hoặc file nén, Kglance hiển thị danh sách dạng bảng. Bạn có thể sử dụng các phím tắt sau để di chuyển lựa chọn:
- `ArrowDown`: Di chuyển chọn tệp tin tiếp theo bên dưới.
- `ArrowUp`: Di chuyển chọn tệp tin phía trên.
- `Home`: Nhảy nhanh về tệp đầu tiên trong danh sách.
- `End`: Nhảy nhanh về tệp cuối cùng trong danh sách.
- `PageDown`: Cuộn nhanh xuống dưới 10 dòng.
- `PageUp`: Cuộn nhanh lên trên 10 dòng.

---

## 6. Luồng đi của dữ liệu qua DBus (Daemon Flow)

Khi bạn tương tác với trình quản lý tệp Dolphin (nhấn `Space` vào một file):
1. Dolphin chạy lệnh `kglance <đường_dẫn_file>` (được cấu hình qua tệp KIO Service Menu `.desktop`).
2. Lệnh này khởi chạy một tiến trình client ngắn hạn. Client này kiểm tra xem Daemon `org.mintori.Kglance` có đang hoạt động trên DBus hay không.
3. Nếu daemon đang chạy: Client gửi yêu cầu `ShowPreview(<đường_dẫn_file>)` qua DBus rồi kết thúc ngay lập tức ($< 2\text{ms}$).
4. Ở phía Daemon, hàm `run_zbus` (trong [src/dbus/service.rs](file:///home/mintori/Projects/temp/Kglance/src/dbus/service.rs)) nhận được yêu cầu, gửi một `DaemonCommand::ShowPreview` qua kênh giao tiếp `tokio::sync::mpsc`.
5. `DaemonRecipe` (trong [src/dbus/recipe.rs](file:///home/mintori/Projects/temp/Kglance/src/dbus/recipe.rs)) lắng nghe channel này như một `Subscription` recipe, chuyển hóa `DaemonCommand` thành `Message::DaemonOpenWindow` hoặc `Message::FileLoaded`.
6. Cửa sổ Iced GUI đang chạy ngầm lập tức mở ra, hiển thị dữ liệu xem trước của file mới.

---

## 7. Các lệnh phát triển thông dụng

Dự án sử dụng `just` làm công cụ chạy lệnh tiện ích. Bạn có thể gõ các lệnh sau trong thư mục dự án:
- `just run`: Chạy thử ứng dụng ở chế độ Standalone.
- `just check`: Chạy kiểm tra cú pháp nhanh bằng `cargo check`.
- `just fmt`: Định dạng lại toàn bộ mã nguồn theo chuẩn Rust.
- `just clippy`: Kiểm tra chất lượng mã nguồn bằng `cargo clippy`.
- `just test`: Chạy toàn bộ các ca kiểm thử tự động.
