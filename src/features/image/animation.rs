use image::AnimationDecoder;
use image::codecs::gif::GifDecoder;
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct AnimatedGifFrame {
    pub handle: iced::widget::image::Handle,
    pub delay_ms: u64,
}

#[derive(Clone, Debug)]
pub struct AnimatedGif {
    pub frames: Vec<AnimatedGifFrame>,
    pub total_duration_ms: u64,
    pub width: u32,
    pub height: u32,
}

impl AnimatedGif {
    pub fn current_frame(&self, elapsed_ms: u64) -> &iced::widget::image::Handle {
        if self.frames.is_empty() {
            panic!("AnimatedGif must have at least one frame");
        }
        if self.frames.len() == 1 || self.total_duration_ms == 0 {
            return &self.frames[0].handle;
        }
        let pos = elapsed_ms % self.total_duration_ms;
        let mut acc = 0;
        for frame in &self.frames {
            acc += frame.delay_ms;
            if pos < acc {
                return &frame.handle;
            }
        }
        &self.frames.last().unwrap().handle
    }
}

pub fn is_gif(bytes: &[u8]) -> bool {
    bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")
}

pub fn decode_animated_gif(bytes: &[u8]) -> Option<AnimatedGif> {
    if !is_gif(bytes) {
        return None;
    }
    let decoder = GifDecoder::new(Cursor::new(bytes)).ok()?;
    let frames = decoder.into_frames().collect_frames().ok()?;
    if frames.is_empty() {
        return None;
    }

    let (first_w, first_h) = (frames[0].buffer().width(), frames[0].buffer().height());
    let mut total_duration_ms = 0;
    let mut anim_frames = Vec::with_capacity(frames.len());

    for frame in frames {
        let (num, denom) = frame.delay().numer_denom_ms();
        let delay_ms = if denom == 0 {
            100
        } else {
            ((num as u64) / (denom as u64)).max(20)
        };
        total_duration_ms += delay_ms;
        let buf = frame.into_buffer();
        let (w, h) = (buf.width(), buf.height());
        let handle = iced::widget::image::Handle::from_rgba(w, h, buf.into_raw());
        anim_frames.push(AnimatedGifFrame { handle, delay_ms });
    }

    Some(AnimatedGif {
        frames: anim_frames,
        total_duration_ms,
        width: first_w,
        height: first_h,
    })
}

#[derive(Clone, Debug)]
pub struct DecodedImagePayload {
    pub handle: iced::widget::image::Handle,
    pub width: u32,
    pub height: u32,
    pub animated_gif: Option<std::sync::Arc<AnimatedGif>>,
}

pub fn decode_image_bytes(bytes: Vec<u8>) -> Option<DecodedImagePayload> {
    if is_gif(&bytes)
        && let Some(anim) = decode_animated_gif(&bytes)
    {
        if anim.frames.len() > 1 {
            let (w, h) = (anim.width, anim.height);
            let first_handle = anim.frames[0].handle.clone();
            return Some(DecodedImagePayload {
                handle: first_handle,
                width: w,
                height: h,
                animated_gif: Some(std::sync::Arc::new(anim)),
            });
        } else if let Some(frame) = anim.frames.first() {
            return Some(DecodedImagePayload {
                handle: frame.handle.clone(),
                width: anim.width,
                height: anim.height,
                animated_gif: None,
            });
        }
    }

    if let Some((handle, w, h)) = crate::features::image::png_to_rgba_handle_with_size(bytes) {
        return Some(DecodedImagePayload {
            handle,
            width: w,
            height: h,
            animated_gif: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gif() {
        assert!(is_gif(b"GIF89a\x01\x00\x01\x00"));
        assert!(is_gif(b"GIF87a\x01\x00\x01\x00"));
        assert!(!is_gif(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn test_decode_animated_gif() {
        use image::codecs::gif::GifEncoder;
        use image::{Delay, Frame, RgbaImage};

        let mut buf = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut buf);
            let frame1 = Frame::from_parts(
                RgbaImage::new(10, 10),
                0,
                0,
                Delay::from_numer_denom_ms(50, 1),
            );
            let frame2 = Frame::from_parts(
                RgbaImage::new(10, 10),
                0,
                0,
                Delay::from_numer_denom_ms(50, 1),
            );
            encoder.encode_frame(frame1).unwrap();
            encoder.encode_frame(frame2).unwrap();
        }

        let anim = decode_animated_gif(&buf).expect("should decode animated GIF");
        assert_eq!(anim.frames.len(), 2);
        assert_eq!(anim.width, 10);
        assert_eq!(anim.height, 10);
        assert_eq!(anim.total_duration_ms, 100);

        // Frame at 20ms should be frame 0
        let f0 = anim.current_frame(20);
        assert_eq!(f0.id(), anim.frames[0].handle.id());

        // Frame at 70ms should be frame 1
        let f1 = anim.current_frame(70);
        assert_eq!(f1.id(), anim.frames[1].handle.id());

        // Frame at 120ms (loops back: 120 % 100 = 20) should be frame 0
        let f2 = anim.current_frame(120);
        assert_eq!(f2.id(), anim.frames[0].handle.id());
    }
}
