use crate::features::pdf::selection::PdfLine;

/// Sorts lines into natural reading order:
/// Top-to-bottom for full-width sections (headers, titles, footers),
/// and column-by-column (left-to-right, top-to-bottom within each column) for multi-column regions.
pub fn sort_lines_reading_order(lines: &mut Vec<PdfLine>) {
    if lines.len() <= 1 {
        return;
    }

    if let Some(sorted) = try_sort_multi_column(lines) {
        *lines = sorted;
        return;
    }

    sort_lines_single_column(lines);
}

pub fn sort_lines_single_column(lines: &mut Vec<PdfLine>) {
    if lines.len() <= 1 {
        return;
    }

    lines.sort_by(|a, b| {
        let y_a = a.rect[1].min(a.rect[3]);
        let y_b = b.rect[1].min(b.rect[3]);
        let x_a = a.rect[0].min(a.rect[2]);
        let x_b = b.rect[0].min(b.rect[2]);
        let diff_y = y_a - y_b;
        if diff_y.abs() > 4.0 {
            y_a.partial_cmp(&y_b).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            x_a.partial_cmp(&x_b).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    struct LineBand {
        y0: f32,
        y1: f32,
        lines: Vec<PdfLine>,
    }

    let mut bands: Vec<LineBand> = Vec::new();

    for line in lines.drain(..) {
        let ly0 = line.rect[1].min(line.rect[3]);
        let ly1 = line.rect[1].max(line.rect[3]);
        let lh = (ly1 - ly0).max(4.0);

        let mut matched_band = None;
        for (idx, band) in bands.iter().enumerate() {
            let bh = (band.y1 - band.y0).max(4.0);
            let min_h = lh.min(bh);
            let overlap_y = (band.y1.min(ly1) - band.y0.max(ly0)).max(0.0);
            let y_diff = (ly0 - band.y0).abs();

            if overlap_y >= 0.4 * min_h || y_diff <= 4.0 {
                matched_band = Some(idx);
                break;
            }
        }

        if let Some(idx) = matched_band {
            let band = &mut bands[idx];
            band.y0 = band.y0.min(ly0);
            band.y1 = band.y1.max(ly1);
            band.lines.push(line);
        } else {
            bands.push(LineBand {
                y0: ly0,
                y1: ly1,
                lines: vec![line],
            });
        }
    }

    bands.sort_by(|a, b| a.y0.partial_cmp(&b.y0).unwrap_or(std::cmp::Ordering::Equal));

    for mut band in bands {
        band.lines.sort_by(|a, b| {
            let x_a = a.rect[0].min(a.rect[2]);
            let x_b = b.rect[0].min(b.rect[2]);
            x_a.partial_cmp(&x_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        lines.extend(band.lines);
    }
}

fn try_sort_multi_column(lines: &[PdfLine]) -> Option<Vec<PdfLine>> {
    if lines.len() < 4 {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;

    for l in lines {
        let x0 = l.rect[0].min(l.rect[2]);
        let x1 = l.rect[0].max(l.rect[2]);
        min_x = min_x.min(x0);
        max_x = max_x.max(x1);
    }

    let page_w = max_x - min_x;
    if page_w < 120.0 {
        return None;
    }

    let search_start = min_x + page_w * 0.15;
    let search_end = min_x + page_w * 0.85;

    let mut x_boundaries: Vec<f32> = lines
        .iter()
        .flat_map(|l| [l.rect[0].min(l.rect[2]), l.rect[0].max(l.rect[2])])
        .filter(|&x| x >= search_start && x <= search_end)
        .collect();
    x_boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    x_boundaries.dedup();

    #[derive(Clone, Copy)]
    struct CandidateGutter {
        g_left: f32,
        g_right: f32,
        gutter_w: f32,
    }

    let mut candidate_gutters = Vec::new();
    for window in x_boundaries.windows(2) {
        let (g_left, g_right) = (window[0], window[1]);
        let gutter_w = g_right - g_left;
        if gutter_w < 8.0 {
            continue;
        }

        let mut left_lines = 0;
        let mut right_lines = 0;
        let mut left_y0 = f32::MAX;
        let mut left_y1 = f32::MIN;
        let mut right_y0 = f32::MAX;
        let mut right_y1 = f32::MIN;

        for l in lines {
            let x0 = l.rect[0].min(l.rect[2]);
            let x1 = l.rect[0].max(l.rect[2]);
            let y0 = l.rect[1].min(l.rect[3]);
            let y1 = l.rect[1].max(l.rect[3]);

            if x1 <= g_left + 1.0 {
                left_lines += 1;
                left_y0 = left_y0.min(y0);
                left_y1 = left_y1.max(y1);
            } else if x0 >= g_right - 1.0 {
                right_lines += 1;
                right_y0 = right_y0.min(y0);
                right_y1 = right_y1.max(y1);
            }
        }

        if left_lines < 2 || right_lines < 2 {
            continue;
        }

        let overlap_top = left_y0.max(right_y0);
        let overlap_bottom = left_y1.min(right_y1);
        if overlap_bottom - overlap_top < 20.0 {
            continue;
        }

        let overlaps_in_col = lines.iter().any(|l| {
            let x0 = l.rect[0].min(l.rect[2]);
            let x1 = l.rect[0].max(l.rect[2]);
            let y0 = l.rect[1].min(l.rect[3]);
            let y1 = l.rect[1].max(l.rect[3]);

            let x_overlaps = x0 < g_right - 1.0 && x1 > g_left + 1.0;
            let y_overlaps = y0 < overlap_bottom - 2.0 && y1 > overlap_top + 2.0;
            x_overlaps && y_overlaps
        });

        if !overlaps_in_col {
            candidate_gutters.push(CandidateGutter {
                g_left,
                g_right,
                gutter_w,
            });
        }
    }

    if candidate_gutters.is_empty() {
        return None;
    }

    candidate_gutters.sort_by(|a, b| {
        a.g_left
            .partial_cmp(&b.g_left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut selected_gutters: Vec<CandidateGutter> = Vec::new();
    for &cand in &candidate_gutters {
        if let Some(prev) = selected_gutters.last()
            && cand.g_left < prev.g_right + 30.0
        {
            if cand.gutter_w > prev.gutter_w {
                let last_idx = selected_gutters.len() - 1;
                selected_gutters[last_idx] = cand;
            }
            continue;
        }
        selected_gutters.push(cand);
    }

    let mut crossing_y_ranges: Vec<(f32, f32)> = Vec::new();
    for l in lines {
        let x0 = l.rect[0].min(l.rect[2]);
        let x1 = l.rect[0].max(l.rect[2]);
        let crosses_any = selected_gutters
            .iter()
            .any(|g| x0 < g.g_left - 2.0 && x1 > g.g_right + 2.0);
        if crosses_any {
            let y0 = l.rect[1].min(l.rect[3]);
            let y1 = l.rect[1].max(l.rect[3]);
            crossing_y_ranges.push((y0, y1));
        }
    }

    crossing_y_ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut merged_crossing: Vec<(f32, f32)> = Vec::new();
    for r in crossing_y_ranges {
        if let Some(last) = merged_crossing.last_mut()
            && r.0 <= last.1 + 8.0
        {
            last.1 = last.1.max(r.1);
            continue;
        }
        merged_crossing.push(r);
    }

    let mut slices: Vec<(f32, f32, bool)> = Vec::new();
    let mut current_y = f32::MIN;
    for (cross_y0, cross_y1) in merged_crossing {
        if cross_y0 > current_y {
            slices.push((current_y, cross_y0, false));
        }
        slices.push((cross_y0, cross_y1, true));
        current_y = cross_y1;
    }
    slices.push((current_y, f32::MAX, false));

    let num_cols = selected_gutters.len() + 1;
    let mut result = Vec::with_capacity(lines.len());
    let mut found_multicolumn = false;

    for (slice_y0, slice_y1, is_crossing) in slices {
        let mut slice_lines: Vec<PdfLine> = Vec::new();
        for l in lines {
            let mid_y = (l.rect[1].min(l.rect[3]) + l.rect[1].max(l.rect[3])) * 0.5;
            if mid_y >= slice_y0 && mid_y < slice_y1 {
                slice_lines.push(l.clone());
            }
        }

        if slice_lines.is_empty() {
            continue;
        }

        if is_crossing {
            sort_lines_single_column(&mut slice_lines);
            result.extend(slice_lines);
            continue;
        }

        let mut cols: Vec<Vec<PdfLine>> = (0..num_cols).map(|_| Vec::new()).collect();
        for l in slice_lines {
            let mid_x = (l.rect[0].min(l.rect[2]) + l.rect[0].max(l.rect[2])) * 0.5;
            let col_idx = selected_gutters
                .iter()
                .position(|g| mid_x < (g.g_left + g.g_right) * 0.5)
                .unwrap_or(selected_gutters.len());
            cols[col_idx].push(l);
        }

        let non_empty_cols: Vec<&Vec<PdfLine>> = cols.iter().filter(|c| c.len() >= 2).collect();
        let is_valid_multicol = if non_empty_cols.len() >= 2 {
            let mut overlap_top = f32::MIN;
            let mut overlap_bottom = f32::MAX;
            for col in &non_empty_cols {
                let col_min_y = col
                    .iter()
                    .map(|l| l.rect[1].min(l.rect[3]))
                    .fold(f32::MAX, f32::min);
                let col_max_y = col
                    .iter()
                    .map(|l| l.rect[1].max(l.rect[3]))
                    .fold(f32::MIN, f32::max);
                overlap_top = overlap_top.max(col_min_y);
                overlap_bottom = overlap_bottom.min(col_max_y);
            }
            overlap_bottom - overlap_top >= 20.0
        } else {
            false
        };

        if is_valid_multicol {
            found_multicolumn = true;
            for mut col in cols {
                sort_lines_single_column(&mut col);
                result.extend(col);
            }
        } else {
            let mut flattened: Vec<PdfLine> = cols.into_iter().flatten().collect();
            sort_lines_single_column(&mut flattened);
            result.extend(flattened);
        }
    }

    if !found_multicolumn {
        return None;
    }

    Some(result)
}
