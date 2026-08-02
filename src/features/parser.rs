use crate::features::{self, common::parser::traits::ParserRegistry};

pub fn build_registry() -> ParserRegistry {
    let mut r = ParserRegistry::new();
    r.register(Box::new(features::markdown::parser::MarkdownParser::new()));
    r.register(Box::new(features::json::parser::JsonParser));
    r.register(Box::new(features::font::parser::FontParser));
    r.register(Box::new(features::text::parser::TextParser::new()));
    r.register(Box::new(features::image::parser::ImageParser));
    r.register(Box::new(features::svg::parser::SvgParser));
    r.register(Box::new(features::pdf::parser::PdfParser));
    r.register(Box::new(features::archive::ArchiveParser));
    r.register(Box::new(features::folder::parser::FolderParser));
    r.register(Box::new(features::audio::parser::AudioParser));
    r.register(Box::new(features::video::parser::VideoParser));
    r.register(Box::new(features::epub::parser::EpubParser));
    r.register(Box::new(features::csv::parser::CsvParser));
    r.register(Box::new(features::office::parser::OfficeParser));
    r.register(Box::new(features::typst::parser::TypstParser));
    r
}
