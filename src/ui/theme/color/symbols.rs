//! Semantic colour tokens for code symbol kinds (Outline, AST viewers).

use iced::Color;

use super::primitive;

#[derive(Clone, Copy)]
pub struct SymbolColors {
    pub function: Color,
    pub r#struct: Color,
    pub class: Color,
    pub r#enum: Color,
    pub r#trait: Color,
    pub module: Color,
    pub r#type: Color,
}

impl SymbolColors {
    pub const DARK: SymbolColors = SymbolColors {
        function: primitive::SYMBOL_FUNCTION,
        r#struct: primitive::SYMBOL_STRUCT,
        class: primitive::SYMBOL_CLASS,
        r#enum: primitive::SYMBOL_ENUM,
        r#trait: primitive::SYMBOL_TRAIT,
        module: primitive::SYMBOL_MODULE,
        r#type: primitive::SYMBOL_TYPE,
    };

    pub const LIGHT: SymbolColors = Self::DARK;

    pub const NORD: SymbolColors = SymbolColors {
        function: primitive::NORD8,
        r#struct: primitive::NORD14,
        class: primitive::NORD13,
        r#enum: primitive::NORD15,
        r#trait: primitive::NORD7,
        module: primitive::NORD9,
        r#type: primitive::NORD7,
    };
}
