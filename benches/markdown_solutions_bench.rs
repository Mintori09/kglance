use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use iced::widget::text::Span;
use iced::widget::{column, container};
use iced::{Element, Font, Length, Padding};
use kglance::app::Message;
use kglance::core::types::MarkdownState;
use kglance::features::markdown::parser::{Block, BlockLayout, Inline, parse_to_blocks};
use kglance::features::markdown::state::{apply_measured_block_heights, recompute_markdown_layout};
use kglance::features::markdown::view::components::inline_spans::{SpanCtx, inlines_to_spans};
use kglance::ui::components::selectable_text::SelectableText;
use kglance::ui::theme::AppTheme;
use std::cell::Cell;
use std::sync::Arc;

const JAPANESE_LESSON_MARKDOWN: &str = r#"> **Phần II:**  
> ⑥ **1** (Thứ tự: 3 → 2 → **1** → 4: 駅員は乗客に**対して**、**電車が****遅れている****理由を**説明した。)  
> ⑦ **1** (Thứ tự: 2 → 4 → **1** → 3: 新しく買った携帯電話はおしゃれで使いやすいが、**前のが****使いやすかった****のに****比べて**使いにくい。)

---

<!-- PAGE 80 -->

## 2日目: 炊きたて (Freshly cooked rice / Cơm vừa mới nấu)

![2日目: 炊きたて](images/page_80_crop_1_image_1.png.png)

> **Mô tả tranh:**
> 「ご飯が炊き上がったよ。」「炊きたてのご飯はおいしいね。」  
> 「こら、食べかけで立っちゃだめ。」  
> *("Cơm đã nấu xong rồi đây." "Cơm vừa mới nấu ngon quá nhỉ!" / "Này, đang ăn dở mà đứng dậy là không được đấy!")*

---

### 1. 書き上げる

> [!NOTE] Ý nghĩa: Vます + 上げる / 上がる
> - **Ý nghĩa:** Hoàn thành trọn vẹn một việc gì đó / Làm xong hẳn. *(その仕事を完全に終えるという意味)*
> - **Ý nghĩa rút gọn:** (＝全部書いた / 焼き終わりました)
> - **English:** Finish doing / Complete doing something.

#### Cách kết hợp:
```
V (thể ます bỏ ます) + 上げる (tha động từ)
V (thể ます bỏ ます) + 上がる (tự động từ)
```

#### Các từ thông dụng:
- 書き上がる / 書き上げる (viết xong)
- 編み上がる / 編み上げる (đan xong)
- 作り上げる (làm xong / tạo dựng thành công)
- 調べ上げる (điều tra làm sáng tỏ hoàn toàn)
- 育て上げる (nuôi nấng khôn lớn)

#### Ví dụ:
1. やっとレポートを**書き上げた**。  
   *(＝全部書いた)*  
   I finally finished my paper.  
   Cuối cùng thì tôi cũng đã viết xong bài báo cáo.
2. ケーキが**焼き上がりました**。  
   *(＝焼き終わりました)*  
   The pastry is just out of the oven.  
   Bánh đã nướng chín rồi đấy.

---

### 2. 食べ切れない

> [!NOTE] Ý nghĩa: Vます + 切る / 切れる / 切れない
> - **Ý nghĩa:**
>   1. Làm hết sạch / Dùng cạn kiệt, không còn sót lại gì. *(全部使って、残っていないようす)*
>   2. V切れない: Không thể làm hết / Quá nhiều không xuể. *(＝完了しない)*
>   3. 疲れ切る: Vô cùng mệt mỏi, kiệt sức. *(＝ひどく疲れたようす)*
> - **English:** Finish all / Completely do; Unable to finish all; Exhausted.

#### Cách kết hợp:
```
V (thể ます bỏ ます) + 切る / 切れる
V (thể ます bỏ ます) + 切れない
```

#### Các từ thông dụng:
- 飲み切る (uống hết sạch)
- 読み切る (đọc hết cả cuốn)
- 走り切る (chạy hết quãng đường)
- 泳ぎ切る (bơi hết chặng)
- 売り切れる (bán sạch / cháy hàng)
- 疲れ切る (mệt lử / kiệt sức)

#### Ví dụ:
1. ご飯の量が多くて、**食べ切れない**よ。  
   *(＝全部食べられない)*  
   I can't finish the rice because there is so much.  
   Cơm nhiều quá nên tôi ăn không hết đâu.
2. 長い小説を、2日間で**読み切った**。  
   *(＝全部読んだ)*  
   I finished reading a long novel in two days.  
   Trong hai ngày tôi đã đọc hết một quyển tiểu thuyết dài.
3. 疲れ切ったようす。  
   *(＝ひどく疲れたようす)*  
   A very tired condition.  
   Tình trạng đang mệt lả / kiệt sức.

---

### 3. 読みかけの本

> [!NOTE] Ý nghĩa: Vます + かける / かけの N / かけだ
> - **Ý nghĩa:** Đang làm dở dang giữa chừng chưa xong; hoặc suýt nữa thì...
> - **Ý nghĩa rút gọn:** (＝読んでいる途中だ / 入ろうとしたときに)
> - **English:** Half-finished / In the middle of doing; On the verge of doing.

#### Cách kết hợp:
```
V (thể ます bỏ ます) + かける
V (thể ます bỏ ます) + かけの + N
V (thể ます bỏ ます) + かけだ
```

#### Các từ thông dụng:
- 食べかける / 食べかけ (ăn dở)
- 飲みかけ (uống dở)
- 帰りかける (đang chuẩn bị về giữa chừng)
- 落ちかける (suýt rơi)
- 失敗しかける (suýt thất bại)

#### Ví dụ:
1. この本はまだ**読みかけだ**。  
   *(＝読んでいる途中だ)*  
   I haven't finished the book yet.  
   Quyển sách này tôi còn đang đọc dở.
2. おふろに**入りかけた**ときに電話が鳴った。  
   *(＝入ろうとしたときに)*  
   The phone rang when I was about to get in the bath.  
   Lúc tôi vừa định bước vào bồn tắm thì điện thoại reo.

---

<!-- PAGE 81 -->

### 4. 焼きたてのパン

> [!NOTE] Ý nghĩa: Vます + たて (たての N / たてだ)
> - **Ý nghĩa:** Vừa mới làm xong còn tươi nguyên / nóng hổi (nhấn mạnh trạng thái tươi mới, mới toanh).
> - **Ý nghĩa rút gọn:** (＝焼いてすぐあとの / 焼いたばかりの)
> - **English:** Freshly done / Just finished.

#### Cách kết hợp:
```
V (thể ます bỏ ます) + たての + N
V (thể ます bỏ ます) + たてだ
```

#### Các từ thông dụng:
- 焼きたて (mới nướng)
- 炊きたて (mới nấu)
- 揚げたて (mới rán)
- 出来たて (mới làm xong)
- 取れたて (mới hái / mới bắt)
- 塗りたて (mới sơn)
"#;

fn extract_inlines_from_blocks(blocks: &[Block]) -> Vec<Vec<Inline>> {
    let mut result = Vec::new();
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => result.push(inlines.clone()),
            Block::Heading { content, .. } => result.push(content.clone()),
            Block::List { items, .. } => {
                for item in items {
                    result.push(item.content.clone());
                }
            }
            Block::Quote(sub_blocks) => {
                result.extend(extract_inlines_from_blocks(sub_blocks));
            }
            Block::Alert { content, .. } => {
                result.extend(extract_inlines_from_blocks(content));
            }
            _ => {}
        }
    }
    result
}

// ---------------------------------------------------------------------------
// 1. Solution 1: Japanese Lesson Real-World Markdown Retained Spans Benchmark
// ---------------------------------------------------------------------------
fn bench_solution_japanese_lesson_retained_spans(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/japanese_lesson_spans");

    let blocks = parse_to_blocks(JAPANESE_LESSON_MARKDOWN);
    let inlines_list = extract_inlines_from_blocks(&blocks);
    let counter = Cell::new(0);

    // Warm up the LRU cache once
    let warm_ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
    };
    for inlines in &inlines_list {
        let _ = inlines_to_spans(inlines, &warm_ctx);
    }

    // A. Direct Real Engine LRU Cache Hit (Production inlines_to_spans)
    group.bench_function("production_lru_cached_spans_japanese_lesson", |b| {
        let ctx = SpanCtx {
            font_family: None,
            font_family_mono: None,
            search_query: "",
            active_match: 0,
            counter: &counter,
            theme: AppTheme::Dark,
        };
        b.iter(|| {
            let mut all_spans = Vec::with_capacity(inlines_list.len());
            for inlines in &inlines_list {
                all_spans.push(inlines_to_spans(inlines, &ctx));
            }
            all_spans
        })
    });

    // B. Search Active Mode (bypasses cache for real-time highlighting)
    group.bench_function("search_active_uncached_japanese_lesson", |b| {
        let ctx = SpanCtx {
            font_family: None,
            font_family_mono: None,
            search_query: "Ý nghĩa",
            active_match: 0,
            counter: &counter,
            theme: AppTheme::Dark,
        };
        b.iter(|| {
            counter.set(0);
            let mut all_spans = Vec::with_capacity(inlines_list.len());
            for inlines in &inlines_list {
                all_spans.push(inlines_to_spans(inlines, &ctx));
            }
            all_spans
        })
    });

    // C. Retained Arc clone (baseline reference)
    let pre_built_spans: Vec<Arc<Vec<Span<'static, (), Font>>>> = inlines_list
        .iter()
        .map(|inlines| {
            let spans = inlines_to_spans(inlines, &warm_ctx);
            // Convert to static lifetime for benchmark container
            let static_spans: Vec<Span<'static, (), Font>> = spans
                .into_iter()
                .map(|s| Span::new(s.text.to_string()).font(s.font.unwrap_or(Font::DEFAULT)))
                .collect();
            Arc::new(static_spans)
        })
        .collect();

    group.bench_function("retained_arc_japanese_lesson", |b| {
        b.iter(|| {
            let cloned: Vec<Arc<Vec<Span<'static, (), Font>>>> =
                pre_built_spans.iter().map(Arc::clone).collect();
            cloned
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 2. Solution 2: Chunk Coalescing on Real Japanese Lesson Blocks
// ---------------------------------------------------------------------------
fn bench_solution_chunk_coalescing_japanese_lesson(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/chunk_coalescing_japanese_lesson");

    let blocks = parse_to_blocks(JAPANESE_LESSON_MARKDOWN);
    let inlines_list = extract_inlines_from_blocks(&blocks);
    let counter = Cell::new(0);
    let ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
    };

    let all_spans_list: Vec<Vec<Span<'static, (), Font>>> = inlines_list
        .iter()
        .map(|inlines| {
            inlines_to_spans(inlines, &ctx)
                .into_iter()
                .map(|s| Span::new(s.text.to_string()).font(s.font.unwrap_or(Font::DEFAULT)))
                .collect()
        })
        .collect();

    // A. Separate individual SelectableText widgets per block
    group.bench_function("separate_widgets_japanese_lesson", |b| {
        b.iter(|| {
            let mut elements: Vec<Element<'_, Message>> = Vec::with_capacity(all_spans_list.len());
            for (idx, spans) in all_spans_list.iter().enumerate() {
                let widget: SelectableText<'_, Message> = SelectableText::new(spans.clone(), 14.0)
                    .width(Length::Fill)
                    .block_index(idx);
                let cont = container(widget)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 4.0,
                        left: 0.0,
                    })
                    .width(Length::Fill);
                elements.push(cont.into());
            }
            let col: Element<'_, Message> = column(elements).width(Length::Fill).into();
            col
        })
    });

    // B. Coalesced composite widget holding all consecutive text spans
    group.bench_function("coalesced_widget_japanese_lesson", |b| {
        b.iter(|| {
            let mut merged_spans = Vec::with_capacity(all_spans_list.len() * 4);
            for spans in &all_spans_list {
                merged_spans.extend(spans.clone());
                merged_spans.push(Span::new("\n"));
            }
            let widget: SelectableText<'_, Message> = SelectableText::new(merged_spans, 14.0)
                .width(Length::Fill)
                .block_index(0);
            let cont: Element<'_, Message> = container(widget)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 4.0,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into();
            cont
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3. Solution 3: Dynamic Physics-Aware Overscan vs Static Overscan
// ---------------------------------------------------------------------------
fn bench_solution_dynamic_overscan(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/overscan_slicing");

    // 10,000 blocks document
    let num_blocks = 10_000;
    let offsets: Vec<f32> = (0..num_blocks).map(|i| i as f32 * 45.0).collect();
    let total_content_height: f32 = num_blocks as f32 * 45.0;
    let vh: f32 = 800.0;
    let scroll_y: f32 = 120_000.0; // Middle of document

    // Static overscan (fixed 800px)
    group.bench_function("static_overscan_slicing", |b| {
        b.iter(|| {
            let overscan_px: f32 = 800.0;
            let view_top = (scroll_y - overscan_px).max(0.0);
            let view_bottom = scroll_y + vh + overscan_px;

            let first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
            let last = offsets
                .partition_point(|&y| y <= view_bottom)
                .min(num_blocks);

            (first, last)
        })
    });

    // Dynamic overscan across 3 velocity states (Stationary, Moderate, High-speed flick)
    for (name, velocity) in [
        ("dynamic_stationary_v0", 0.0f32),
        ("dynamic_moderate_v1500", 1500.0f32),
        ("dynamic_fast_flick_v6000", 6000.0f32),
    ] {
        group.bench_with_input(
            BenchmarkId::new("dynamic_overscan", name),
            &velocity,
            |b, &v: &f32| {
                b.iter(|| {
                    let overscan_px =
                        (vh * 1.2 + v.abs() * 0.12).clamp(vh * 1.0, (vh * 4.0).max(2400.0));
                    let view_top = (scroll_y - overscan_px).max(0.0);
                    let view_bottom = scroll_y + vh + overscan_px;

                    const CHUNK_SIZE: usize = 8;
                    const OVERSCAN_CHUNKS: usize = 2;

                    let raw_first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
                    let raw_last = offsets
                        .partition_point(|&y| y <= view_bottom)
                        .min(num_blocks);

                    let remaining_blocks = num_blocks.saturating_sub(raw_last);
                    let dist_to_bottom = total_content_height - (scroll_y + vh);
                    let is_near_bottom = raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE >= num_blocks
                        || remaining_blocks <= CHUNK_SIZE * 2
                        || dist_to_bottom <= overscan_px * 1.5;

                    let first_visible = raw_first.saturating_sub(OVERSCAN_CHUNKS * CHUNK_SIZE)
                        / CHUNK_SIZE
                        * CHUNK_SIZE;

                    let last_visible = if is_near_bottom {
                        num_blocks
                    } else {
                        (raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE)
                            .min(num_blocks)
                            .div_ceil(CHUNK_SIZE)
                            * CHUNK_SIZE
                    };

                    (first_visible, last_visible.min(num_blocks))
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// 4. Solution 4: Incremental Measured Heights Update vs Full Recompute
// ---------------------------------------------------------------------------
fn bench_solution_incremental_measured_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/layout_incremental_update");

    let num_blocks = 5_000;
    let initial_layouts: Vec<BlockLayout> =
        (0..num_blocks).map(|_| BlockLayout::new(50.0)).collect();
    let initial_offsets: Vec<f32> = (0..num_blocks).map(|i| i as f32 * 50.0).collect();

    let state = MarkdownState {
        block_layouts: initial_layouts,
        block_y_offsets: initial_offsets,
        total_content_height: num_blocks as f32 * 50.0,
        scroll_y: 50_000.0,
        viewport_height: 800.0,
        ..Default::default()
    };

    let measurements = vec![
        (100, 75.0),
        (101, 85.0),
        (102, 60.0),
        (103, 110.0),
        (104, 90.0),
    ];

    // A. Incremental batched measurement update with scroll anchoring
    group.bench_function("incremental_apply_measurements_5_blocks", |b| {
        b.iter(|| {
            let mut s = state.clone();
            apply_measured_block_heights(&mut s, &measurements, 14.0);
            s
        })
    });

    // B. Recomputing all 5000 block layouts from scratch
    let dummy_blocks: Vec<Block> = (0..num_blocks)
        .map(|i| {
            Block::Paragraph(vec![Inline::Text(format!(
                "Paragraph line {i} sample text"
            ))])
        })
        .collect();

    group.bench_function("full_recompute_all_5000_blocks", |b| {
        b.iter(|| {
            let mut s = state.clone();
            recompute_markdown_layout(&mut s, &dummy_blocks, 14.0, 800.0);
            s
        })
    });

    group.finish();
}

criterion_group!(
    solutions_benches,
    bench_solution_japanese_lesson_retained_spans,
    bench_solution_chunk_coalescing_japanese_lesson,
    bench_solution_dynamic_overscan,
    bench_solution_incremental_measured_layout,
);
criterion_main!(solutions_benches);
