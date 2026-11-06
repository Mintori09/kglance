# Markdown Optimization & Feature Development Tasks

Tài liệu theo dõi tiến độ các tác vụ tối ưu hóa và phát triển tính năng Markdown trong worktree `.worktrees/markdown-optimization`.

---

## 1. Danh sách 10 Tác vụ Tối ưu hóa Hiệu năng (Đã hoàn thành)

- [x] **Task 1: Layout Recompute Fast-path Guard**
  - **Mô tả:** Tránh tính toán lại layout toàn bộ khối markdown (`recompute_markdown_layout`) khi nội dung, độ rộng hiển thị (`content_width`), cỡ chữ và trạng thái bảng mục lục không thay đổi.
  - **Files:** `src/features/markdown/update.rs`, `src/features/markdown/state.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 2: Resource Cache Management & Cleanup**
  - **Mô tả:** Giới hạn dung lượng cache và chủ động dọn dẹp bộ nhớ đệm image handle / mermaid preview handle khi đóng file hoặc chuyển đổi tài liệu nhằm tránh rò rỉ RAM.
  - **Files:** `src/features/markdown/state.rs`, `src/features/markdown/view/components/image.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 3: Batch Block Measurements & Storm Elimination**
  - **Mô tả:** Gom cụm sự kiện đo lường kích thước khối (`BlockMeasured`) để tránh gây nghẽn hàng đợi thông điệp UI loop (`Message`).
  - **Files:** `src/features/markdown/measure.rs`, `src/app/update/mod.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 4: Incremental Layout Shift Updates**
  - **Mô tả:** Cập nhật vị trí Y các khối phía sau một khối thay đổi kích thước theo dạng delta / prefix sum thay vì tính toán lại toàn bộ O(N) blocks.
  - **Files:** `src/features/markdown/state.rs`, `src/features/markdown/update.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 5: Directional Overscan & Virtual Window Jitter Removal**
  - **Mô tả:** Cải thiện logic virtual windowing với overscan định hướng cuộn, loại bỏ hiện tượng giật spacer đầu trang khi cuộn nhanh.
  - **Files:** `src/features/markdown/view/window.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 6: Zero-Copy Text Search Matching**
  - **Mô tả:** Tối ưu thuật toán tìm kiếm nội dung markdown, loại bỏ các phép clone chuỗi trung gian từ code block và bảng.
  - **Files:** `src/features/markdown/update.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 7: Large Table Rendering Virtualization / Cap**
  - **Mô tả:** Giới hạn số lượng hàng render tối đa cho các bảng markdown kích thước khổng lồ để tránh treo UI.
  - **Files:** `src/features/markdown/view/components/table.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 8: Modular Architecture Refactoring (< 800 LoC)**
  - **Mô tả:** Tách nhỏ file `handle.rs` (> 1000 LoC) thành các submodule chuyên biệt `split.rs` và `frontmatter.rs` tuân thủ quy chuẩn kiến trúc Kglance.
  - **Files:** `src/features/markdown/parser/handle.rs`, `src/features/markdown/parser/split.rs`, `src/features/markdown/parser/frontmatter.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 9: TOC Sidebar Character Width & Accent Calibration**
  - **Mô tả:** Hiệu chuẩn bề rộng ký tự CJK và dấu thanh tiếng Việt trong thuật toán ước tính chiều cao và dòng wrap của sidebar TOC.
  - **Files:** `src/parsers/markdown/mod.rs`, `src/features/markdown/update.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

- [x] **Task 10: Selection & Highlight Bounds Safety**
  - **Mô tả:** Đảm bảo an toàn phạm vi selection point, xử lý các trường hợp drag ra ngoài khung nhìn và giải phóng đúng tài nguyên.
  - **Files:** `src/features/markdown/update.rs`
  - **Trạng thái:** Hoàn thành (Commit `e309c1a`).

---

## 2. Tính năng mới: Mang Trải Nghiệm Tìm Kiếm EPUB sang Markdown (Đã hoàn thành)

- [x] **Task 11: Sidebar Search with Context Snippets (Tương tự EPUB)**
  - **Mô tả:**
    1. Xây dựng module tìm kiếm `search.rs` trích xuất context snippet 1–2 dòng quanh từ khóa khớp với `...`.
    2. Tích hợp thanh tab chuyển đổi giữa `Contents` (Mục lục) và `Search` (Tìm kiếm) ở đầu sidebar.
    3. Thẻ snippet hiển thị danh sách kết quả, click vào nhảy mượt (`smooth_scroll`) và đồng bộ virtual window tới khối văn bản.
    4. Thay thế floating search bar trên đỉnh, gán <kbd>Ctrl</kbd>+<kbd>F</kbd> mở và focus ô tìm kiếm trong sidebar.
    5. Đầy đủ unit tests cho tìm kiếm không phân biệt hoa thường, trích xuất snippet UTF-8 và điều hướng next/prev.
  - **Files:**
    - `src/features/markdown/search.rs`
    - `src/features/markdown/types.rs`
    - `src/features/markdown/state.rs`
    - `src/features/markdown/update.rs`
    - `src/features/markdown/view/toc.rs`
    - `src/features/markdown/view/mod.rs`
    - `src/app/keyboard/search.rs`
    - `src/app/keyboard/ctrl.rs`
  - **Trạng thái:** Hoàn thành (Commit `784ba83`).
