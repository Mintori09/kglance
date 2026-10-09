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

    pub const CATPPUCCIN_MOCHA: SymbolColors = SymbolColors {
        function: primitive::MOCHA_BLUE,
        r#struct: primitive::MOCHA_YELLOW,
        class: primitive::MOCHA_YELLOW,
        r#enum: primitive::MOCHA_MAUVE,
        r#trait: primitive::MOCHA_TEAL,
        module: primitive::MOCHA_LAVENDER,
        r#type: primitive::MOCHA_SKY,
    };

    pub const CATPPUCCIN_LATTE: SymbolColors = SymbolColors {
        function: primitive::LATTE_BLUE,
        r#struct: primitive::LATTE_YELLOW,
        class: primitive::LATTE_YELLOW,
        r#enum: primitive::LATTE_LAVENDER,
        r#trait: primitive::LATTE_SAPPHIRE,
        module: primitive::LATTE_LAVENDER,
        r#type: primitive::LATTE_SKY,
    };

    pub const TOKYO_NIGHT: SymbolColors = SymbolColors {
        function: primitive::TOKYO_BLUE,
        r#struct: primitive::TOKYO_YELLOW,
        class: primitive::TOKYO_YELLOW,
        r#enum: primitive::TOKYO_PURPLE,
        r#trait: primitive::TOKYO_CYAN,
        module: primitive::TOKYO_CYAN,
        r#type: primitive::TOKYO_BLUE,
    };

    pub const GRUVBOX_DARK: SymbolColors = SymbolColors {
        function: primitive::GRUVBOX_GREEN,
        r#struct: primitive::GRUVBOX_YELLOW,
        class: primitive::GRUVBOX_YELLOW,
        r#enum: primitive::GRUVBOX_PURPLE,
        r#trait: primitive::GRUVBOX_AQUA,
        module: primitive::GRUVBOX_ORANGE,
        r#type: primitive::GRUVBOX_AQUA,
    };

    pub const DRACULA: SymbolColors = SymbolColors {
        function: primitive::DRACULA_GREEN,
        r#struct: primitive::DRACULA_CYAN,
        class: primitive::DRACULA_CYAN,
        r#enum: primitive::DRACULA_PURPLE,
        r#trait: primitive::DRACULA_PINK,
        module: primitive::DRACULA_ORANGE,
        r#type: primitive::DRACULA_YELLOW,
    };
}
