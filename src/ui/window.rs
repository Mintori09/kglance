use crate::log_info;
use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;

use image::GenericImageView;
use slint::{ComponentHandle, Global, Model};

use crate::parser::pdf::render_pdf_page;
use crate::parser::{DirEntry, ExtractedFile, ImageFormat, PageData, ParsedContent, extract_entry};
use crate::ui::helpers::{
    FileHandler, ThumbnailData, breadcrumb, copy_to_clipboard, find_file_index,
    make_image_from_rgba, scan_dir_for_files, set_file_info,
};
use crate::ui::image_handler::{
    format_exif_sidebar, reset_image_state, update_image_display, update_image_display_raw,
};
use crate::ui::table::{SortField, SortState, build_archive_rows, build_folder_rows};
use crate::ui::theme::{DARK, LIGHT, detect_dark_mode, slint_color};
use crate::ui::video_player;

#[allow(dead_code)]
pub struct PreviewWindow {
    ui: super::generated::PreviewWindow,
    pdf_path: Rc<RefCell<String>>,
    pdf_page_count: Rc<RefCell<u32>>,
    pdf_index: Rc<RefCell<usize>>,
    sort_state: Rc<RefCell<SortState>>,
    folder_entries: Rc<RefCell<Vec<DirEntry>>>,
    back_target: Rc<RefCell<Option<String>>>,
    file_selected_handler: Rc<RefCell<Option<FileHandler>>>,
    archive_context: Rc<RefCell<Option<String>>>,
    current_extracted: Rc<RefCell<Option<ExtractedFile>>>,

    // Phase 3: Image interaction
    original_image: Rc<RefCell<Option<image::DynamicImage>>>,
    image_exif: Rc<RefCell<Option<crate::parser::ExifData>>>,
    image_format: Rc<RefCell<Option<ImageFormat>>>,
    image_zoom: Rc<RefCell<f32>>,
    image_rotation: Rc<RefCell<i32>>,
    image_pan_x: Rc<RefCell<f32>>,
    image_pan_y: Rc<RefCell<f32>>,

    // Phase 3: Navigation history
    history: Rc<RefCell<Vec<String>>>,
    history_index: Rc<RefCell<isize>>,

    // Phase 3: Multi-file
    dir_files: Rc<RefCell<Vec<String>>>,
    current_dir_idx: Rc<RefCell<Option<usize>>>,
    current_path: Rc<RefCell<String>>,

    // Phase 3: Text search
    full_text: Rc<RefCell<String>>,
    search_matches: Rc<RefCell<Vec<(usize, usize)>>>,
    search_match_idx: Rc<RefCell<usize>>,

    // Phase 3: PDF zoom
    pdf_zoom: Rc<RefCell<f32>>,
    pdf_thumbnails: Rc<RefCell<ThumbnailData>>,
    video_player: Rc<video_player::VideoPlayerController>,
}

impl PreviewWindow {
    pub fn new(standalone: bool) -> Result<Self, Box<dyn Error>> {
        let ui = super::generated::PreviewWindow::new()?;
        let pdf_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let pdf_page_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let pdf_index: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

        let sort_state: Rc<RefCell<SortState>> = Rc::new(RefCell::new(SortState {
            field: SortField::Name,
            ascending: true,
        }));
        let folder_entries: Rc<RefCell<Vec<DirEntry>>> = Rc::new(RefCell::new(Vec::new()));
        let back_target: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let file_selected_handler: Rc<RefCell<Option<FileHandler>>> = Rc::new(RefCell::new(None));
        let archive_context: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let current_extracted: Rc<RefCell<Option<ExtractedFile>>> = Rc::new(RefCell::new(None));

        let original_image: Rc<RefCell<Option<image::DynamicImage>>> = Rc::new(RefCell::new(None));
        let image_exif: Rc<RefCell<Option<crate::parser::ExifData>>> = Rc::new(RefCell::new(None));
        let image_format: Rc<RefCell<Option<ImageFormat>>> = Rc::new(RefCell::new(None));
        let image_zoom: Rc<RefCell<f32>> = Rc::new(RefCell::new(1.0));
        let image_rotation: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
        let image_pan_x: Rc<RefCell<f32>> = Rc::new(RefCell::new(0.0));
        let image_pan_y: Rc<RefCell<f32>> = Rc::new(RefCell::new(0.0));

        let history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let history_index: Rc<RefCell<isize>> = Rc::new(RefCell::new(-1));

        let dir_files: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let current_dir_idx: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let current_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let full_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let search_matches: Rc<RefCell<Vec<(usize, usize)>>> = Rc::new(RefCell::new(Vec::new()));
        let search_match_idx: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

        let pdf_zoom: Rc<RefCell<f32>> = Rc::new(RefCell::new(1.0));
        let pdf_thumbnails: Rc<RefCell<ThumbnailData>> = Rc::new(RefCell::new(Vec::new()));
        let video_player = Rc::new(video_player::VideoPlayerController::spawn(ui.as_weak()));

        // ---- Wire callbacks ----

        let weak = ui.as_weak();
        ui.on_open_clicked(move || {
            if let Some(handle) = weak.upgrade() {
                let path = handle.get_file_name();
                if !path.is_empty() {
                    log_info!("UI: Open clicked for file: {}", path);
                    let _ = std::process::Command::new("xdg-open")
                        .arg(path.as_str())
                        .spawn();
                    let _ = handle.window().hide();
                    if standalone {
                        let _ = slint::quit_event_loop();
                    }
                }
            }
        });

        let weak_copy = ui.as_weak();
        ui.on_copy_path_clicked(move || {
            if let Some(handle) = weak_copy.upgrade() {
                let path = handle.get_file_name();
                if !path.is_empty() {
                    log_info!("UI: Copy path clicked for: {}", path);
                    copy_to_clipboard(path.as_str());
                }
            }
        });

        // ---- Close requested (Esc key) ----
        let weak_close = ui.as_weak();
        ui.on_close_requested(move || {
            log_info!("UI: Close requested (Esc key pressed)");
            if let Some(handle) = weak_close.upgrade() {
                let _ = handle.window().hide();
                if standalone {
                    let _ = slint::quit_event_loop();
                }
            }
        });

        // ---- PDF page navigation ----
        let prev_path = pdf_path.clone();
        let prev_index = pdf_index.clone();
        let weak_prev = ui.as_weak();
        ui.on_prev_page_clicked(move || {
            let mut idx = prev_index.borrow_mut();
            if *idx > 0 {
                let new_idx = *idx - 1;
                if let Some(handle) = weak_prev.upgrade() {
                    let path = prev_path.borrow().clone();
                    if !path.is_empty()
                        && let Ok(page) = render_pdf_page(Path::new(&path), new_idx as u32)
                    {
                        let img = make_image_from_rgba(&page.data, page.width, page.height);
                        handle.set_preview_image(img);
                        *idx = new_idx;
                        handle.set_pdf_current_page((new_idx + 1) as i32);
                    }
                }
            }
        });

        let next_path = pdf_path.clone();
        let next_page_count = pdf_page_count.clone();
        let next_index = pdf_index.clone();
        let weak_next = ui.as_weak();
        ui.on_next_page_clicked(move || {
            let mut idx = next_index.borrow_mut();
            let max = *next_page_count.borrow();
            if *idx + 1 < max as usize {
                let new_idx = *idx + 1;
                if let Some(handle) = weak_next.upgrade() {
                    let path = next_path.borrow().clone();
                    if !path.is_empty()
                        && let Ok(page) = render_pdf_page(Path::new(&path), new_idx as u32)
                    {
                        let img = make_image_from_rgba(&page.data, page.width, page.height);
                        handle.set_preview_image(img);
                        *idx = new_idx;
                        handle.set_pdf_current_page((new_idx + 1) as i32);
                    }
                }
            }
        });

        // ---- Table file click ----
        let bt = back_target.clone();
        let handler = file_selected_handler.clone();
        let weak_fc = ui.as_weak();
        let ac = archive_context.clone();
        let ce = current_extracted.clone();
        ui.on_file_clicked(move |index| {
            if let Some(handle) = weak_fc.upgrade()
                && let model = handle.get_table_data()
                && index >= 0
                && (index as usize) < model.row_count()
                && let Some(row) = model.row_data(index as usize)
                && let Some(h) = handler.borrow().as_ref()
            {
                if let Some(archive_path) = ac.borrow().as_ref() {
                    if !row.path.is_empty() {
                        let entry_path = row.path.to_string();
                        if let Ok(extracted) = extract_entry(Path::new(archive_path), &entry_path) {
                            let tmp_path = extracted.path.to_string_lossy().to_string();
                            *ce.borrow_mut() = Some(extracted);
                            *bt.borrow_mut() = Some(archive_path.clone());
                            handle.set_show_back_button(true);
                            h(tmp_path);
                        }
                    }
                } else if !row.path.is_empty() {
                    let path = row.path.to_string();
                    *bt.borrow_mut() = Some(handle.get_file_name().to_string());
                    handle.set_show_back_button(true);
                    h(path);
                }
            }
        });

        // ---- Sort ----
        let ws = sort_state.clone();
        let fe = folder_entries.clone();
        let weak_sort = ui.as_weak();
        ui.on_sort_by_field_clicked(move |field| {
            if let Some(handle) = weak_sort.upgrade() {
                let mut state = ws.borrow_mut();
                let new_field = match field {
                    0 => SortField::Name,
                    1 => SortField::Kind,
                    2 => SortField::Modified,
                    _ => SortField::Size,
                };
                if state.field == new_field {
                    state.ascending = !state.ascending;
                } else {
                    state.field = new_field;
                    state.ascending = true;
                }
                let entries = fe.borrow();
                if !entries.is_empty() {
                    let path = handle.get_file_name().to_string();
                    let rows = build_folder_rows(&entries, &path, &state);
                    handle.set_table_data(slint::ModelRc::from(std::rc::Rc::new(rows)));
                    handle.set_show_table(true);
                }
            }
        });

        // ---- Back button ----
        let bt_back = back_target.clone();
        let handler_back = file_selected_handler.clone();
        let weak_back = ui.as_weak();
        ui.on_back_clicked(move || {
            if let Some(handle) = weak_back.upgrade() {
                handle.set_show_back_button(false);
            }

            if let Some(path) = bt_back.borrow().as_ref()
                && let Some(h) = handler_back.borrow().as_ref()
            {
                h(path.clone());
            }
        });

        // ---- History back/forward ----
        let hist = history.clone();
        let hist_idx = history_index.clone();
        let weak_hb = ui.as_weak();
        let handler_hb = file_selected_handler.clone();
        let cp = current_path.clone();
        let df = dir_files.clone();
        let cdi = current_dir_idx.clone();
        let oi = original_image.clone();
        let ie = image_exif.clone();
        let ifmt = image_format.clone();
        let iz = image_zoom.clone();
        let ir = image_rotation.clone();
        let ipx = image_pan_x.clone();
        let ipy = image_pan_y.clone();
        let ft = full_text.clone();
        let sm = search_matches.clone();
        let smi = search_match_idx.clone();

        ui.on_history_back(move || {
            let mut idx = hist_idx.borrow_mut();
            let h = hist.borrow();
            if *idx > 0 {
                *idx -= 1;
                if let Some(path) = h.get(*idx as usize) {
                    *cp.borrow_mut() = path.clone();
                    if let Some(handler) = handler_hb.borrow().as_ref() {
                        reset_image_state(&oi, &ie, &ifmt, &iz, &ir, &ipx, &ipy);
                        reset_search_state(&ft, &sm, &smi);
                        reset_dir_state(&df, &cdi);
                        handler(path.clone());
                        if let Some(handle) = weak_hb.upgrade() {
                            handle.set_can_go_back(*idx > 0);
                            handle.set_can_go_forward(*idx + 1 < h.len() as isize);
                        }
                    }
                }
            }
        });

        let hist2 = history.clone();
        let hist_idx2 = history_index.clone();
        let weak_hf = ui.as_weak();
        let handler_hf = file_selected_handler.clone();
        let cp2 = current_path.clone();
        let df2 = dir_files.clone();
        let cdi2 = current_dir_idx.clone();
        let oi2 = original_image.clone();
        let ie2 = image_exif.clone();
        let ifmt2 = image_format.clone();
        let iz2 = image_zoom.clone();
        let ir2 = image_rotation.clone();
        let ipx2 = image_pan_x.clone();
        let ipy2 = image_pan_y.clone();
        let ft2 = full_text.clone();
        let sm2 = search_matches.clone();
        let smi2 = search_match_idx.clone();

        ui.on_history_forward(move || {
            let mut idx = hist_idx2.borrow_mut();
            let h = hist2.borrow();
            if *idx + 1 < h.len() as isize {
                *idx += 1;
                if let Some(path) = h.get(*idx as usize) {
                    *cp2.borrow_mut() = path.clone();
                    if let Some(handler) = handler_hf.borrow().as_ref() {
                        reset_image_state(&oi2, &ie2, &ifmt2, &iz2, &ir2, &ipx2, &ipy2);
                        reset_search_state(&ft2, &sm2, &smi2);
                        reset_dir_state(&df2, &cdi2);
                        handler(path.clone());
                        if let Some(handle) = weak_hf.upgrade() {
                            handle.set_can_go_back(*idx > 0);
                            handle.set_can_go_forward(*idx + 1 < h.len() as isize);
                        }
                    }
                }
            }
        });

        // ---- Image interaction callbacks ----
        let iz_in = image_zoom.clone();
        let ir_in = image_rotation.clone();
        let ipx_in = image_pan_x.clone();
        let ipy_in = image_pan_y.clone();
        let oi_in = original_image.clone();
        let weak_im = ui.as_weak();
        ui.on_image_zoom_in(move || {
            let z = *iz_in.borrow() * 1.25;
            *iz_in.borrow_mut() = z.min(10.0);
            if let Some(handle) = weak_im.upgrade() {
                handle.set_image_zoom(*iz_in.borrow());
            }
            update_image_display(&oi_in, &iz_in, &ir_in, &ipx_in, &ipy_in, &weak_im);
        });

        let iz_out = image_zoom.clone();
        let ir_out = image_rotation.clone();
        let ipx_out = image_pan_x.clone();
        let ipy_out = image_pan_y.clone();
        let oi_out = original_image.clone();
        let weak_im2 = ui.as_weak();
        ui.on_image_zoom_out(move || {
            let z = *iz_out.borrow() / 1.25;
            *iz_out.borrow_mut() = z.max(0.1);
            if let Some(handle) = weak_im2.upgrade() {
                handle.set_image_zoom(*iz_out.borrow());
            }
            update_image_display(&oi_out, &iz_out, &ir_out, &ipx_out, &ipy_out, &weak_im2);
        });

        let iz_rl = image_zoom.clone();
        let ir_rl = image_rotation.clone();
        let ipx_rl = image_pan_x.clone();
        let ipy_rl = image_pan_y.clone();
        let oi_rl = original_image.clone();
        let weak_rl = ui.as_weak();
        ui.on_image_rotate_l(move || {
            let r = (*ir_rl.borrow() + 3) % 4;
            *ir_rl.borrow_mut() = r;
            *ipx_rl.borrow_mut() = 0.0;
            *ipy_rl.borrow_mut() = 0.0;
            if let Some(handle) = weak_rl.upgrade() {
                handle.set_image_rotation(*ir_rl.borrow());
                handle.set_image_pan_x(0.0);
                handle.set_image_pan_y(0.0);
            }
            update_image_display(&oi_rl, &iz_rl, &ir_rl, &ipx_rl, &ipy_rl, &weak_rl);
        });

        let iz_rr = image_zoom.clone();
        let ir_rr = image_rotation.clone();
        let ipx_rr = image_pan_x.clone();
        let ipy_rr = image_pan_y.clone();
        let oi_rr = original_image.clone();
        let weak_rr = ui.as_weak();
        ui.on_image_rotate_r(move || {
            let r = (*ir_rr.borrow() + 1) % 4;
            *ir_rr.borrow_mut() = r;
            *ipx_rr.borrow_mut() = 0.0;
            *ipy_rr.borrow_mut() = 0.0;
            if let Some(handle) = weak_rr.upgrade() {
                handle.set_image_rotation(*ir_rr.borrow());
                handle.set_image_pan_x(0.0);
                handle.set_image_pan_y(0.0);
            }
            update_image_display(&oi_rr, &iz_rr, &ir_rr, &ipx_rr, &ipy_rr, &weak_rr);
        });

        let iz_rs = image_zoom.clone();
        let ir_rs = image_rotation.clone();
        let ipx_rs = image_pan_x.clone();
        let ipy_rs = image_pan_y.clone();
        let oi_rs = original_image.clone();
        let weak_rs = ui.as_weak();
        ui.on_image_reset(move || {
            *iz_rs.borrow_mut() = 1.0;
            *ir_rs.borrow_mut() = 0;
            *ipx_rs.borrow_mut() = 0.0;
            *ipy_rs.borrow_mut() = 0.0;
            if let Some(handle) = weak_rs.upgrade() {
                handle.set_image_zoom(1.0);
                handle.set_image_rotation(0);
                handle.set_image_pan_x(0.0);
                handle.set_image_pan_y(0.0);
            }
            update_image_display(&oi_rs, &iz_rs, &ir_rs, &ipx_rs, &ipy_rs, &weak_rs);
        });

        let ipx_drag = image_pan_x.clone();
        let ipy_drag = image_pan_y.clone();
        let weak_drag = ui.as_weak();
        ui.on_image_drag_moved(move |dx, dy| {
            *ipx_drag.borrow_mut() += dx;
            *ipy_drag.borrow_mut() += dy;
            if let Some(handle) = weak_drag.upgrade() {
                handle.set_image_pan_x(*ipx_drag.borrow());
                handle.set_image_pan_y(*ipy_drag.borrow());
            }
        });

        let iz_scroll = image_zoom.clone();
        let ir_scroll = image_rotation.clone();
        let ipx_scroll = image_pan_x.clone();
        let ipy_scroll = image_pan_y.clone();
        let oi_scroll = original_image.clone();
        let weak_scroll = ui.as_weak();
        ui.on_image_scroll_zoom(move |dy| {
            let z = *iz_scroll.borrow();
            let nz = if dy > 0.0 {
                (z / 1.1).max(0.1)
            } else {
                (z * 1.1).min(10.0)
            };
            *iz_scroll.borrow_mut() = nz;
            if let Some(handle) = weak_scroll.upgrade() {
                handle.set_image_zoom(nz);
            }
            update_image_display(
                &oi_scroll,
                &iz_scroll,
                &ir_scroll,
                &ipx_scroll,
                &ipy_scroll,
                &weak_scroll,
            );
        });

        // ---- Multi-file ----
        let df_prev = dir_files.clone();
        let cdi_prev = current_dir_idx.clone();
        let _weak_prev_f = ui.as_weak();
        let handler_prev = file_selected_handler.clone();
        ui.on_prev_file_clicked(move || {
            let path = {
                let files = df_prev.borrow();
                let idx = cdi_prev.borrow();
                let i = match *idx {
                    Some(i) if i > 0 => i,
                    _ => return,
                };
                match files.get(i - 1) {
                    Some(p) => p.clone(),
                    None => return,
                }
            };
            let h = handler_prev.borrow();
            if let Some(h) = h.as_ref() {
                h(path);
            }
        });

        let df_next = dir_files.clone();
        let cdi_next = current_dir_idx.clone();
        let _weak_next_f = ui.as_weak();
        let handler_next = file_selected_handler.clone();
        ui.on_next_file_clicked(move || {
            let path = {
                let files = df_next.borrow();
                let idx = cdi_next.borrow();
                let i = match *idx {
                    Some(i) => i,
                    _ => return,
                };
                if i + 1 >= files.len() {
                    return;
                }
                match files.get(i + 1) {
                    Some(p) => p.clone(),
                    None => return,
                }
            };
            let h = handler_next.borrow();
            if let Some(h) = h.as_ref() {
                h(path);
            }
        });

        // ---- Play/Pause clicked (Video/Audio) ----
        let video_player_play = video_player.clone();
        let play_path = current_path.clone();
        let weak_play = ui.as_weak();
        ui.on_play_pause_clicked(move || {
            if let Some(handle) = weak_play.upgrade() {
                let is_video = handle.get_show_video();
                if is_video {
                    let playing = handle.get_video_playing();
                    if playing {
                        video_player_play.pause();
                    } else {
                        let path = play_path.borrow().clone();
                        video_player_play.load(path);
                        video_player_play.play();
                    }
                } else {
                    let path = play_path.borrow().clone();
                    if !path.is_empty() {
                        if std::process::Command::new("mpv")
                            .arg(&path)
                            .spawn()
                            .is_err()
                        {
                            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                        }
                        let _ = handle.window().hide();
                    }
                }
            }
        });

        // ---- Seek clicked ----
        let video_player_seek = video_player.clone();
        ui.on_seek_clicked(move |percent| {
            let clamped = (percent as f64).clamp(0.0, 1.0);
            video_player_seek.seek(clamped);
        });

        // ---- Seek relative clicked ----
        let video_player_seek_relative = video_player.clone();
        ui.on_seek_relative_clicked(move |seconds| {
            video_player_seek_relative.seek_relative(seconds as f64);
        });

        // ---- Text search ----
        let _ft_ts = full_text.clone();
        let sm_ts = search_matches.clone();
        let smi_ts = search_match_idx.clone();
        let weak_ts = ui.as_weak();
        ui.on_text_search_next(move || {
            let matches = sm_ts.borrow();
            if matches.is_empty() {
                return;
            }
            let mut idx = smi_ts.borrow_mut();
            *idx = (*idx + 1) % matches.len();
            if let Some(handle) = weak_ts.upgrade() {
                handle.set_text_search_info(format!("{}/{}", *idx + 1, matches.len()).into());
            }
        });

        let _ft_tsp = full_text.clone();
        let sm_tsp = search_matches.clone();
        let smi_tsp = search_match_idx.clone();
        let weak_tsp = ui.as_weak();
        ui.on_text_search_prev(move || {
            let matches = sm_tsp.borrow();
            if matches.is_empty() {
                return;
            }
            let mut idx = smi_tsp.borrow_mut();
            *idx = if *idx == 0 {
                matches.len() - 1
            } else {
                *idx - 1
            };
            if let Some(handle) = weak_tsp.upgrade() {
                handle.set_text_search_info(format!("{}/{}", *idx + 1, matches.len()).into());
            }
        });

        let _ft_tsc = full_text.clone();
        let sm_tsc = search_matches.clone();
        let smi_tsc = search_match_idx.clone();
        let weak_tsc = ui.as_weak();
        ui.on_text_search_closed(move || {
            *sm_tsc.borrow_mut() = Vec::new();
            *smi_tsc.borrow_mut() = 0;
            if let Some(handle) = weak_tsc.upgrade() {
                handle.set_show_text_search(false);
                handle.set_text_search_query("".into());
                handle.set_text_search_info("".into());
            }
        });

        let _ft_wrap = full_text.clone();
        let weak_wrap = ui.as_weak();
        ui.on_text_wrap_toggled(move || {
            if let Some(handle) = weak_wrap.upgrade() {
                handle.set_text_wrap(!handle.get_text_wrap());
            }
        });

        let ft_copy = full_text.clone();
        let _weak_copy2 = ui.as_weak();
        ui.on_copy_content_clicked(move || {
            let text = ft_copy.borrow();
            if !text.is_empty() {
                copy_to_clipboard(&text);
            }
        });

        // ---- Close handling ----
        if standalone {
            let weak2 = ui.as_weak();
            ui.window().on_close_requested(move || {
                let _ = weak2;
                let _ = slint::quit_event_loop();
                slint::CloseRequestResponse::HideWindow
            });
        }

        let w = Self {
            ui,
            pdf_path,
            pdf_page_count,
            pdf_index,
            sort_state,
            folder_entries,
            back_target,
            file_selected_handler,
            archive_context,
            current_extracted,
            original_image,
            image_exif,
            image_format,
            image_zoom,
            image_rotation,
            image_pan_x,
            image_pan_y,
            history,
            history_index,
            dir_files,
            current_dir_idx,
            current_path,
            full_text,
            search_matches,
            search_match_idx,
            pdf_zoom,
            pdf_thumbnails,
            video_player,
        };
        w.apply_theme();
        Ok(w)
    }

    fn show_pdf(&self, path: &str, page_count: u32, first_page: PageData) {
        self.ui.set_title_text(path.into());
        self.pdf_path.replace(path.to_string());
        self.pdf_page_count.replace(page_count);
        self.pdf_index.replace(0);
        let img = make_image_from_rgba(&first_page.data, first_page.width, first_page.height);
        self.ui.set_preview_image(img);
        self.ui.set_show_image(true);
        self.ui.set_show_pdf_nav(true);
        self.ui.set_pdf_current_page(1);
        self.ui.set_pdf_page_count(page_count as i32);
        self.ui
            .set_status_text(format!("PDF  |  {page_count} pages").into());

        // Render thumbnails
        let mut thumbs = Vec::new();
        for i in 0..page_count.min(20) {
            if let Ok(page) = render_pdf_page(Path::new(path), i) {
                let thumb = image::imageops::thumbnail(
                    &image::RgbaImage::from_raw(page.width, page.height, page.data)
                        .unwrap_or_default(),
                    80,
                    110,
                );
                let (tw, th) = thumb.dimensions();
                thumbs.push((thumb.into_raw(), tw, th));
            }
        }
        *self.pdf_thumbnails.borrow_mut() = thumbs;
    }

    pub fn show(&self, path: &str, content: &ParsedContent) {
        log_info!("PreviewWindow: show() path: {}", path);
        self.video_player.stop();
        self.ui.set_file_name(path.into());
        self.ui.set_content_ready(false);
        self.ui.set_show_text(false);
        self.ui.set_show_highlighted(false);
        self.ui.set_show_image(false);
        self.ui.set_show_markdown(false);
        self.ui.set_show_table(false);
        self.ui.set_show_pdf_nav(false);
        self.ui.set_show_file_info(false);
        self.ui.set_show_video(false);
        self.ui.set_show_audio(false);
        self.ui.set_show_office(false);
        self.ui.set_show_font(false);
        self.ui.set_show_exif_sidebar(false);
        self.ui.set_line_numbers_content("".into());
        set_file_info(&self.ui, path);

        // Scan directory for multi-file nav
        let files = scan_dir_for_files(path);
        let dir_idx = find_file_index(&files, path);
        *self.dir_files.borrow_mut() = files.clone();
        *self.current_dir_idx.borrow_mut() = dir_idx;
        *self.current_path.borrow_mut() = path.to_string();
        if let Some(i) = dir_idx {
            self.ui.set_has_prev_file(i > 0);
            self.ui.set_has_next_file(i + 1 < files.len());
        } else {
            self.ui.set_has_prev_file(false);
            self.ui.set_has_next_file(false);
        }

        match content {
            ParsedContent::Text {
                content,
                language,
                line_count,
                highlighted_html,
            } => {
                log_info!(
                    "PreviewWindow: Rendering Text (language={}, line_count={})",
                    language,
                    line_count
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                self.ui.set_text_content(content.into());
                *self.full_text.borrow_mut() = content.clone();
                let lines = (1..=*line_count)
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.ui.set_line_numbers_content(lines.into());
                if let Some(html) = highlighted_html {
                    if let Ok(st) = slint::StyledText::from_markdown(html) {
                        self.ui.set_styled_content(st);
                        self.ui.set_show_highlighted(true);
                    } else {
                        self.ui.set_show_text(true);
                    }
                } else {
                    self.ui.set_show_text(true);
                }
                self.ui
                    .set_status_text(format!("{language}  |  {line_count} lines").into());
            }
            ParsedContent::Image {
                data,
                width,
                height,
                format,
                exif,
            } => {
                log_info!(
                    "PreviewWindow: Rendering Image (size={}x{}, format={:?})",
                    width,
                    height,
                    format
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());

                // Decode and store original image
                if let Ok(img) = image::load_from_memory(data) {
                    *self.original_image.borrow_mut() = Some(img);
                    *self.image_zoom.borrow_mut() = 1.0;
                    *self.image_rotation.borrow_mut() = 0;
                    *self.image_pan_x.borrow_mut() = 0.0;
                    *self.image_pan_y.borrow_mut() = 0.0;
                    self.ui.set_image_zoom(1.0);
                    self.ui.set_image_rotation(0);
                    self.ui.set_image_pan_x(0.0);
                    self.ui.set_image_pan_y(0.0);

                    if let Some(exif_data) = exif.as_ref() {
                        *self.image_exif.borrow_mut() = Some(*exif_data.clone());
                        let sidebar = format_exif_sidebar(exif_data);
                        if !sidebar.is_empty() {
                            self.ui.set_exif_content(sidebar.into());
                            self.ui.set_show_exif_sidebar(true);
                        }
                    }
                    *self.image_format.borrow_mut() = Some(*format);
                    let ui_weak = self.ui.as_weak();
                    update_image_display_raw(
                        &self.original_image,
                        &self.image_zoom,
                        &self.image_rotation,
                        &self.image_pan_x,
                        &self.image_pan_y,
                        &ui_weak,
                    );
                }
                self.ui.set_show_image(true);
                let fmt = match format {
                    ImageFormat::Png => "PNG",
                    ImageFormat::Jpeg => "JPEG",
                    ImageFormat::WebP => "WebP",
                    ImageFormat::Gif => "GIF",
                    ImageFormat::Bmp => "BMP",
                };
                let mut exif_text = format!("{fmt}  |  {width}x{height}");
                if let Some(exif) = exif.as_ref() {
                    if let Some(ref cam) = exif.camera_make {
                        exif_text.push_str(&format!("  |  Camera: {cam}"));
                    }
                    if let Some(ref model) = exif.camera_model {
                        exif_text.push_str(&format!(" {model}"));
                    }
                    if let Some(ref date) = exif.date_taken {
                        exif_text.push_str(&format!("  |  {date}"));
                    }
                    if let Some(ref iso) = exif.iso {
                        exif_text.push_str(&format!("  |  ISO {iso}"));
                    }
                    if let Some(ref fnum) = exif.f_number {
                        exif_text.push_str(&format!("  |  f/{fnum}"));
                    }
                }
                self.ui.set_status_text(exif_text.into());
            }
            ParsedContent::Pdf {
                page_count,
                first_page,
            } => {
                log_info!("PreviewWindow: Rendering PDF (page_count={})", page_count);
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.show_pdf(path, *page_count, first_page.clone());
            }
            ParsedContent::Archive {
                entries,
                total_files,
            } => {
                log_info!(
                    "PreviewWindow: Rendering Archive (entries={}, total_files={})",
                    entries.len(),
                    total_files
                );
                self.archive_context.borrow_mut().replace(path.to_string());
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                let rows = build_archive_rows(entries);
                self.ui
                    .set_table_data(slint::ModelRc::from(std::rc::Rc::new(rows)));
                self.ui.set_show_table(true);
                self.ui
                    .set_status_text(format!("Archive  |  {total_files} files").into());
            }
            ParsedContent::Markdown { content, images } => {
                log_info!(
                    "PreviewWindow: Rendering Markdown (images={})",
                    images.len()
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());

                let line_count = content.lines().count();
                if line_count > 1000 {
                    self.ui.set_text_content(content.as_str().into());
                    let lines = (1..=line_count)
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.ui.set_line_numbers_content(lines.into());
                    self.ui.set_show_markdown(false);
                    self.ui.set_show_text(true);
                    self.ui.set_status_text(
                        format!(
                            "Markdown  |  {} lines (showing as plain text for performance)",
                            line_count
                        )
                        .into(),
                    );
                } else {
                    let (blocks, mermaid_tasks) = parse_markdown_to_slint_blocks(content);
                    let model = std::rc::Rc::new(slint::VecModel::from(blocks));

                    for (block_idx, code) in mermaid_tasks {
                        let cache_path = get_mermaid_cache_path(&code);
                        if cache_path.exists() {
                            if let (Ok(img), Some(mut block)) = (
                                slint::Image::load_from_path(&cache_path),
                                model.row_data(block_idx),
                            ) {
                                block.image_content = img;
                                model.set_row_data(block_idx, block);
                            }
                        } else {
                            let raw_ptr = std::rc::Rc::into_raw(model.clone()) as usize;
                            let code_clone = code.clone();
                            let cp_clone = cache_path.clone();
                            std::thread::spawn(move || {
                                if render_mermaid_to_svg(&code_clone, &cp_clone).is_some() {
                                    let _ = slint::invoke_from_event_loop(move || {
                                        let model_rc = unsafe {
                                            std::rc::Rc::from_raw(
                                                raw_ptr
                                                    as *const slint::VecModel<
                                                        super::generated::SlintMarkdownBlock,
                                                    >,
                                            )
                                        };
                                        if let (Ok(img), Some(mut block)) = (
                                            slint::Image::load_from_path(&cp_clone),
                                            model_rc.row_data(block_idx),
                                        ) {
                                            block.image_content = img;
                                            model_rc.set_row_data(block_idx, block);
                                        }
                                    });
                                } else {
                                    // Make sure we drop the raw pointer reference even if rendering fails
                                    let _ = slint::invoke_from_event_loop(move || {
                                        let _ = unsafe {
                                            std::rc::Rc::from_raw(
                                                raw_ptr
                                                    as *const slint::VecModel<
                                                        super::generated::SlintMarkdownBlock,
                                                    >,
                                            )
                                        };
                                    });
                                }
                            });
                        }
                    }

                    self.ui.set_markdown_blocks(slint::ModelRc::from(model));
                    self.ui.set_show_markdown(true);
                    self.ui
                        .set_status_text(format!("Markdown  |  {} images", images.len()).into());
                }
            }
            ParsedContent::Folder { entries } => {
                log_info!(
                    "PreviewWindow: Rendering Folder (entries={})",
                    entries.len()
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(breadcrumb(path).into());
                self.folder_entries.borrow_mut().clear();
                self.folder_entries
                    .borrow_mut()
                    .extend(entries.iter().cloned());
                let sort = self.sort_state.borrow();
                let rows = build_folder_rows(entries, path, &sort);
                drop(sort);
                self.ui
                    .set_table_data(slint::ModelRc::from(std::rc::Rc::new(rows)));
                self.ui.set_show_table(true);
                self.ui
                    .set_status_text(format!("Folder  |  {} items", entries.len()).into());
            }
            ParsedContent::Video {
                path: _vp,
                duration,
                thumbnail,
            } => {
                log_info!("PreviewWindow: Rendering Video (duration={}s)", duration);
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                self.ui.set_show_video(true);
                if let Ok(img) = image::load_from_memory(thumbnail) {
                    let (w, h) = img.dimensions();
                    let slint_img = make_image_from_rgba(&img.to_rgba8(), w, h);
                    self.ui.set_preview_image(slint_img);
                }
                let mins = (duration / 60.0) as u64;
                let secs = (duration % 60.0) as u64;
                let dur = if *duration > 0.0 {
                    format!("{mins}:{secs:02}")
                } else {
                    "Unknown".into()
                };
                self.ui.set_video_time(dur.clone().into());
                self.ui.set_video_playing(true);
                self.ui.set_status_text(format!("Video  |  {dur}").into());
                self.video_player.load(path.to_string());
                self.video_player.play();
            }
            ParsedContent::Audio {
                metadata,
                waveform,
                waveform_width,
                waveform_height,
            } => {
                log_info!("PreviewWindow: Rendering Audio ({})", metadata);
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                self.ui.set_show_audio(true);
                self.ui.set_audio_metadata(metadata.into());
                self.ui.set_video_playing(false);
                if !waveform.is_empty() && *waveform_width > 0 && *waveform_height > 0 {
                    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(
                        *waveform_width,
                        *waveform_height,
                    );
                    let pixel_slice = buffer.make_mut_slice();
                    for (i, pixel) in pixel_slice.iter_mut().enumerate() {
                        let offset = i * 4;
                        if offset + 3 < waveform.len() {
                            pixel.r = waveform[offset];
                            pixel.g = waveform[offset + 1];
                            pixel.b = waveform[offset + 2];
                            pixel.a = waveform[offset + 3];
                        }
                    }
                    self.ui.set_audio_waveform(slint::Image::from_rgba8(buffer));
                }
                self.ui.set_status_text(metadata.into());
            }
            ParsedContent::Office {
                content,
                format: ofmt,
                page_count,
            } => {
                log_info!(
                    "PreviewWindow: Rendering Office Document (format={}, page_count={})",
                    ofmt,
                    page_count
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                self.ui.set_show_office(true);
                self.ui.set_text_content(content.into());
                self.ui
                    .set_status_text(format!("{ofmt}  |  {page_count} pages").into());
            }
            ParsedContent::Font {
                name,
                metadata: fm,
                sample,
                sample_width,
                sample_height,
            } => {
                log_info!(
                    "PreviewWindow: Rendering Font (name={}, metadata={})",
                    name,
                    fm
                );
                self.archive_context.borrow_mut().take();
                self.current_extracted.borrow_mut().take();
                self.ui.set_title_text(path.into());
                self.ui.set_show_font(true);
                let display = format!("{name}\n\n{fm}");
                self.ui.set_text_content(display.into());
                if !sample.is_empty() && *sample_width > 0 && *sample_height > 0 {
                    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(
                        *sample_width,
                        *sample_height,
                    );
                    let pixel_slice = buffer.make_mut_slice();
                    for (i, pixel) in pixel_slice.iter_mut().enumerate() {
                        let offset = i * 4;
                        if offset + 3 < sample.len() {
                            pixel.r = sample[offset];
                            pixel.g = sample[offset + 1];
                            pixel.b = sample[offset + 2];
                            pixel.a = sample[offset + 3];
                        }
                    }
                    self.ui.set_font_sample(slint::Image::from_rgba8(buffer));
                }
                self.ui.set_status_text(format!("Font  |  {name}").into());
            }
        }

        // Push to history
        {
            let mut hist = self.history.borrow_mut();
            let mut idx = self.history_index.borrow_mut();
            if *idx >= 0 && (*idx as usize) < hist.len() {
                let pos = *idx as usize;
                // If we went back and now navigate to a new file, truncate forward history
                if pos + 1 < hist.len() {
                    hist.truncate(pos + 1);
                }
            }
            hist.push(path.to_string());
            *idx = (hist.len() - 1) as isize;
            self.ui.set_can_go_back(*idx > 0);
            self.ui.set_can_go_forward(false);
        }

        self.ui.set_content_ready(true);
        let _ = self.ui.window().show();
    }

    pub fn apply_theme(&self) {
        let is_dark = detect_dark_mode();
        let p = if is_dark { &DARK } else { &LIGHT };
        let theme = super::generated::AppTheme::get(&self.ui);
        theme.set_background(slint_color(p.bg));
        theme.set_text(slint_color(p.text));
        theme.set_text_muted(slint_color(p.text_muted));
        theme.set_text_faint(slint_color(p.text_faint));
        theme.set_text_mono(slint_color(p.text_mono));
        theme.set_border(slint_color(p.border));
        theme.set_row_even(slint_color(p.row_even));
        theme.set_row_odd(slint_color(p.row_odd));
        theme.set_row_hover(slint_color(p.row_hover));
        theme.set_status_bg(slint_color(p.status_bg));
        theme.set_line_number(slint_color(p.text_faint));
        theme.set_header_hover(if is_dark {
            slint_color(0x3d8bfd)
        } else {
            slint_color(0x0056b3)
        });
    }

    pub fn set_file_selected_handler<F: Fn(String) + 'static>(&self, handler: F) {
        *self.file_selected_handler.borrow_mut() = Some(Box::new(handler));
    }

    pub fn hide(&self) {
        log_info!("PreviewWindow: Hiding window");
        let _ = self.ui.window().hide();
    }

    pub fn window(&self) -> &slint::Window {
        self.ui.window()
    }
}

// ---- Reset helper functions ----

fn reset_search_state(
    full_text: &Rc<RefCell<String>>,
    matches: &Rc<RefCell<Vec<(usize, usize)>>>,
    idx: &Rc<RefCell<usize>>,
) {
    *full_text.borrow_mut() = String::new();
    *matches.borrow_mut() = Vec::new();
    *idx.borrow_mut() = 0;
}

fn reset_dir_state(files: &Rc<RefCell<Vec<String>>>, idx: &Rc<RefCell<Option<usize>>>) {
    *files.borrow_mut() = Vec::new();
    *idx.borrow_mut() = None;
}

fn get_mermaid_cache_path(code: &str) -> std::path::PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    let hash_val = hasher.finish();
    let filename = format!("{:016x}.svg", hash_val);

    let cache_dir = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".cache/kglance/mermaid"))
        .unwrap_or_else(|_| std::env::temp_dir().join("kglance-mermaid"));

    cache_dir.join(filename)
}

fn render_mermaid_to_svg(code: &str, cache_path: &Path) -> Option<()> {
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut child = std::process::Command::new("mmdr")
        .args(["-e", "svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(code.as_bytes());
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }

    std::fs::write(cache_path, &output.stdout).ok()?;
    Some(())
}

fn parse_markdown_to_slint_blocks(
    content: &str,
) -> (
    Vec<super::generated::SlintMarkdownBlock>,
    Vec<(usize, String)>,
) {
    use super::generated::{SlintMarkdownBlock, SlintTableRow};
    let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());
    let mut blocks = Vec::new();
    let mut mermaid_tasks = Vec::new();

    struct TableState {
        rows: Vec<Vec<String>>,
        current_row: Vec<String>,
        current_cell: String,
    }
    let mut table_state: Option<TableState> = None;
    let mut mermaid_state: Option<String> = None;
    let mut current_text = String::new();

    let flush_text = |text: &mut String, blocks: &mut Vec<SlintMarkdownBlock>| {
        if !text.trim().is_empty() {
            let styled = slint::StyledText::from_markdown(text).unwrap_or_default();
            blocks.push(SlintMarkdownBlock {
                is_table: false,
                is_image: false,
                text_content: styled,
                image_content: slint::Image::default(),
                table_rows: slint::ModelRc::default(),
                col_count: 0,
            });
            text.clear();
        }
    };

    for event in parser {
        if let Some(ref mut code) = mermaid_state {
            match event {
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                    let mermaid_code = code.clone();
                    mermaid_state = None;

                    mermaid_tasks.push((blocks.len(), mermaid_code));
                    blocks.push(SlintMarkdownBlock {
                        is_table: false,
                        is_image: true,
                        text_content: slint::StyledText::default(),
                        image_content: slint::Image::default(),
                        table_rows: slint::ModelRc::default(),
                        col_count: 0,
                    });
                }
                pulldown_cmark::Event::Text(text) => {
                    code.push_str(&text);
                }
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                    code.push('\n');
                }
                _ => {}
            }
            continue;
        }

        if let Some(ref mut state) = table_state {
            match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::TableCell) => {
                    state.current_cell.clear();
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableCell) => {
                    state.current_row.push(state.current_cell.clone());
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableRow)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableHead) => {
                    state.rows.push(state.current_row.clone());
                    state.current_row.clear();
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Table) => {
                    let max_cols = state.rows.iter().map(|r| r.len()).max().unwrap_or(0);
                    let col_count = max_cols.max(1) as i32;

                    let slint_rows: Vec<SlintTableRow> = state
                        .rows
                        .iter()
                        .map(|row| {
                            let mut cells = row.clone();
                            while cells.len() < max_cols {
                                cells.push(String::new());
                            }
                            let slint_cells: Vec<slint::SharedString> = cells
                                .iter()
                                .map(|c| slint::SharedString::from(c.as_str()))
                                .collect();
                            SlintTableRow {
                                cells: slint::ModelRc::from(std::rc::Rc::new(
                                    slint::VecModel::from(slint_cells),
                                )),
                            }
                        })
                        .collect();

                    blocks.push(SlintMarkdownBlock {
                        is_table: true,
                        is_image: false,
                        text_content: slint::StyledText::default(),
                        image_content: slint::Image::default(),
                        table_rows: slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(
                            slint_rows,
                        ))),
                        col_count,
                    });
                    table_state = None;
                }
                pulldown_cmark::Event::Text(text) => {
                    state.current_cell.push_str(&text);
                }
                pulldown_cmark::Event::Code(code) => {
                    state.current_cell.push('`');
                    state.current_cell.push_str(&code);
                    state.current_cell.push('`');
                }
                _ => {}
            }
            continue;
        }

        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Table(_) => {
                    flush_text(&mut current_text, &mut blocks);
                    table_state = Some(TableState {
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        current_cell: String::new(),
                    });
                }
                pulldown_cmark::Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(ref lang))
                    if lang.as_ref() == "mermaid" =>
                {
                    flush_text(&mut current_text, &mut blocks);
                    mermaid_state = Some(String::new());
                }
                pulldown_cmark::Tag::Heading { level, .. } => {
                    current_text.push_str("\n**");
                    let prefix = match level {
                        pulldown_cmark::HeadingLevel::H1 => "# ",
                        pulldown_cmark::HeadingLevel::H2 => "## ",
                        pulldown_cmark::HeadingLevel::H3 => "### ",
                        _ => "#### ",
                    };
                    current_text.push_str(prefix);
                }
                pulldown_cmark::Tag::Strong => {
                    current_text.push_str("**");
                }
                pulldown_cmark::Tag::Emphasis => {
                    current_text.push('*');
                }
                pulldown_cmark::Tag::Item => {
                    current_text.push_str("  • ");
                }
                pulldown_cmark::Tag::CodeBlock(_) => {
                    current_text.push('\n');
                }
                _ => {}
            },
            pulldown_cmark::Event::End(tag) => match tag {
                pulldown_cmark::TagEnd::Heading(_) => {
                    current_text.push_str("**\n");
                }
                pulldown_cmark::TagEnd::Strong => {
                    current_text.push_str("**");
                }
                pulldown_cmark::TagEnd::Emphasis => {
                    current_text.push('*');
                }
                pulldown_cmark::TagEnd::Item => {
                    current_text.push('\n');
                }
                pulldown_cmark::TagEnd::CodeBlock => {
                    current_text.push('\n');
                }
                pulldown_cmark::TagEnd::Paragraph => {
                    current_text.push_str("\n\n");
                }
                _ => {}
            },
            pulldown_cmark::Event::Text(text) => {
                current_text.push_str(&text);
            }
            pulldown_cmark::Event::Code(code) => {
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    flush_text(&mut current_text, &mut blocks);
    (blocks, mermaid_tasks)
}
