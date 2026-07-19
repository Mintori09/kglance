use crate::parser::{ExifData, ImageFormat};
use image::GenericImageView;
use std::cell::RefCell;
use std::rc::Rc;

pub fn format_exif_sidebar(exif: &ExifData) -> String {
    let mut lines = Vec::new();
    if let Some(ref m) = exif.camera_make {
        lines.push(format!("Make: {m}"));
    }
    if let Some(ref m) = exif.camera_model {
        lines.push(format!("Model: {m}"));
    }
    if let Some(ref d) = exif.date_taken {
        lines.push(format!("Date: {d}"));
    }
    if let Some(ref i) = exif.iso {
        lines.push(format!("ISO: {i}"));
    }
    if let Some(ref f) = exif.f_number {
        lines.push(format!("Aperture: f/{f}"));
    }
    if let Some(ref e) = exif.exposure {
        lines.push(format!("Exposure: {e}"));
    }
    if let Some(ref fl) = exif.focal_length {
        lines.push(format!("Focal: {fl}mm"));
    }
    if let (Some(gps_lat), Some(gps_lon)) = (exif.gps_lat.as_ref(), exif.gps_lon.as_ref()) {
        lines.push(format!("GPS: {gps_lat}, {gps_lon}"));
    }
    lines.join("\n")
}

pub fn update_image_display(
    original: &RefCell<Option<image::DynamicImage>>,
    zoom: &RefCell<f32>,
    rotation: &RefCell<i32>,
    pan_x: &RefCell<f32>,
    pan_y: &RefCell<f32>,
    weak: &slint::Weak<super::generated::PreviewWindow>,
) {
    update_image_display_raw(original, zoom, rotation, pan_x, pan_y, weak);
}

pub fn update_image_display_raw(
    original: &RefCell<Option<image::DynamicImage>>,
    zoom: &RefCell<f32>,
    rotation: &RefCell<i32>,
    _pan_x: &RefCell<f32>,
    _pan_y: &RefCell<f32>,
    weak: &slint::Weak<super::generated::PreviewWindow>,
) {
    let img_opt = original.borrow();
    let Some(ref img) = *img_opt else { return };
    let z = *zoom.borrow();
    let r = *rotation.borrow();

    let mut processed = img.clone();
    for _ in 0..r {
        processed = processed.rotate90();
    }
    let (w, h) = processed.dimensions();
    let nw = (w as f32 * z).max(1.0) as u32;
    let nh = (h as f32 * z).max(1.0) as u32;
    let scaled = processed.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgba = scaled.to_rgba8();
    let (rw, rh) = rgba.dimensions();
    let raw = rgba.into_raw();

    if let Some(handle) = weak.upgrade() {
        let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(rw, rh);
        let pixel_slice = buffer.make_mut_slice();
        for (i, pixel) in pixel_slice.iter_mut().enumerate() {
            let offset = i * 4;
            pixel.r = raw[offset];
            pixel.g = raw[offset + 1];
            pixel.b = raw[offset + 2];
            pixel.a = raw[offset + 3];
        }
        handle.set_preview_image(slint::Image::from_rgba8(buffer));
    }
}

pub fn reset_image_state(
    original: &Rc<RefCell<Option<image::DynamicImage>>>,
    exif: &Rc<RefCell<Option<ExifData>>>,
    fmt: &Rc<RefCell<Option<ImageFormat>>>,
    zoom: &Rc<RefCell<f32>>,
    rotation: &Rc<RefCell<i32>>,
    pan_x: &Rc<RefCell<f32>>,
    pan_y: &Rc<RefCell<f32>>,
) {
    *original.borrow_mut() = None;
    *exif.borrow_mut() = None;
    *fmt.borrow_mut() = None;
    *zoom.borrow_mut() = 1.0;
    *rotation.borrow_mut() = 0;
    *pan_x.borrow_mut() = 0.0;
    *pan_y.borrow_mut() = 0.0;
}
