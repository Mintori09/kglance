use std::cell::Cell;

use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::Inline;
use crate::ui::theme::font::{get_code_font, get_main_font};
use iced::font::Weight;
use iced::widget::text::Span;
use iced::{Color, Font};

use crate::ui::theme::AppTheme;

pub(crate) struct SpanCtx<'a> {
    pub font_family: Option<&'a str>,
    pub font_family_mono: Option<&'a str>,
    pub search_query: &'a str,
    pub active_match: usize,
    pub counter: &'a Cell<usize>,
    pub theme: AppTheme,
}

fn search_highlight_color(is_active: bool, theme: AppTheme) -> Color {
    let mp = theme.palette().markdown;
    if is_active {
        mp.search_active_bg
    } else {
        mp.search_inactive_bg
    }
}

fn highlight_search_in_text<'a>(
    text: &'a str,
    span_ctx: &SpanCtx,
    font: Font,
    normal_color: Option<Color>,
) -> Vec<Span<'a, (), Font>> {
    let mut spans = Vec::new();
    let lower = text.to_lowercase();
    let query_lower = span_ctx.search_query.to_lowercase();
    let mut pos = 0;

    while let Some(match_pos) = lower[pos..].find(&query_lower) {
        let abs_pos = pos + match_pos;
        let end_pos = abs_pos + query_lower.len();

        if abs_pos > pos {
            let mut span = Span::new(&text[pos..abs_pos]).font(font);
            if let Some(color) = normal_color {
                span = span.color(color);
            }
            spans.push(span);
        }

        let bg = search_highlight_color(
            span_ctx.counter.get() == span_ctx.active_match,
            span_ctx.theme,
        );
        spans.push(Span::new(&text[abs_pos..end_pos]).font(font).background(bg));

        span_ctx.counter.set(span_ctx.counter.get() + 1);
        pos = end_pos;
    }

    if pos < text.len() {
        let mut span = Span::new(&text[pos..]).font(font);
        if let Some(color) = normal_color {
            span = span.color(color);
        }
        spans.push(span);
    }

    spans
}

fn inlines_to_spans_core<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
) -> Vec<Span<'a, (), Font>> {
    let main_font = get_main_font(span_ctx.font_family);
    let code_font = get_code_font(span_ctx.font_family_mono);
    let mut spans = Vec::new();
    let search_query = span_ctx.search_query;

    for inline in children {
        match inline {
            Inline::Text(t) => {
                if search_query.is_empty() {
                    spans.push(Span::new(t.as_str()).font(main_font));
                } else {
                    spans.extend(highlight_search_in_text(t, span_ctx, main_font, None));
                }
            }
            Inline::Bold(children) => {
                spans.extend(apply_style_to_children(
                    children,
                    span_ctx,
                    main_font,
                    |f| Font {
                        weight: Weight::Bold,
                        ..f
                    },
                ));
            }
            Inline::Italic(children) => {
                spans.extend(apply_style_to_children(
                    children,
                    span_ctx,
                    main_font,
                    |f| Font {
                        style: iced::font::Style::Italic,
                        ..f
                    },
                ));
            }
            Inline::Strikethrough(children) => {
                for s in inlines_to_spans_core(children, span_ctx) {
                    spans.push(s.font(main_font).strikethrough(true));
                }
            }
            Inline::Code(code) => {
                if search_query.is_empty() {
                    spans.push(
                        Span::new(code.as_str())
                            .font(code_font)
                            .color(STYLE.inline.inline_code_color),
                    );
                } else {
                    spans.extend(highlight_search_in_text(
                        code,
                        span_ctx,
                        code_font,
                        Some(STYLE.inline.inline_code_color),
                    ));
                }
            }
            Inline::Link {
                text: link_text, ..
            } => {
                let link_color = span_ctx.theme.palette().roles.link;
                for s in inlines_to_spans_core(link_text, span_ctx) {
                    spans.push(s.color(link_color).underline(true));
                }
            }
            Inline::SoftBreak => {
                spans.push(Span::new(" ").font(main_font));
            }
            Inline::Image { alt, .. } => {
                spans.push(
                    Span::new(format!("[{alt}]"))
                        .font(main_font)
                        .color(STYLE.inline.image_alt_color),
                );
            }
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => {
                let display_text = render_latex_to_text(latex);
                let math_color = span_ctx.theme.palette().markdown.math;
                spans.push(Span::new(display_text).font(main_font).color(math_color));
            }
        }
    }

    spans
}

fn apply_style_to_children<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
    main_font: Font,
    style: fn(Font) -> Font,
) -> Vec<Span<'a, (), Font>> {
    inlines_to_spans_core(children, span_ctx)
        .into_iter()
        .map(|span| span.font(style(main_font)))
        .collect()
}

pub(crate) fn inlines_to_spans<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
) -> Vec<Span<'a, (), Font>> {
    inlines_to_spans_core(children, span_ctx)
}

pub(crate) fn render_latex_to_text(latex: &str) -> String {
    let mut result = replace_text_macros(latex.to_string());
    result = replace_environments(result);
    result = replace_blackboard(result);
    result = replace_fractions(result);
    result = replace_sqrt(result);
    result = replace_latex_commands(&result);
    replace_scripts(&result)
}

fn replace_environments(mut s: String) -> String {
    const ENVS: &[(&str, &str)] = &[
        ("\\begin{bmatrix}", "[\n"),
        ("\\end{bmatrix}", "\n]"),
        ("\\begin{pmatrix}", "(\n"),
        ("\\end{pmatrix}", "\n)"),
        ("\\begin{matrix}", "\n"),
        ("\\end{matrix}", "\n"),
        ("\\begin{aligned}", ""),
        ("\\end{aligned}", ""),
        ("\\begin{cases}", "{\n"),
        ("\\end{cases}", "\n}"),
    ];
    for &(env, rep) in ENVS {
        if s.contains(env) {
            s = s.replace(env, rep);
        }
    }
    // Replace alignment & with space
    s = s.replace('&', " ");
    s
}

fn replace_blackboard(mut s: String) -> String {
    const BB: &[(&str, &str)] = &[
        ("\\mathbb{R}", "ℝ"),
        ("\\mathbb{N}", "ℕ"),
        ("\\mathbb{Z}", "ℤ"),
        ("\\mathbb{Q}", "ℚ"),
        ("\\mathbb{C}", "ℂ"),
        ("\\mathbb{H}", "ℍ"),
    ];
    for &(cmd, unicode) in BB {
        if s.contains(cmd) {
            s = s.replace(cmd, unicode);
        }
    }
    s
}

fn replace_latex_commands(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\\' && i + 1 < len {
            let next_ch = chars[i + 1];
            if next_ch.is_alphabetic() {
                let start = i;
                i += 1;
                while i < len && chars[i].is_alphabetic() {
                    i += 1;
                }
                let cmd_str: String = chars[start..i].iter().collect();
                if let Some(replacement) = latex_cmd_to_unicode(&cmd_str) {
                    out.push_str(replacement);
                    if replacement.is_empty() && i < len && chars[i] == ' ' {
                        i += 1;
                    }
                } else {
                    out.push_str(&cmd_str);
                }
                continue;
            } else {
                match next_ch {
                    '{' => {
                        out.push('{');
                        i += 2;
                    }
                    '}' => {
                        out.push('}');
                        i += 2;
                    }
                    ',' | ';' | ':' => {
                        out.push(' ');
                        i += 2;
                    }
                    '!' => {
                        i += 2;
                    }
                    '\\' => {
                        out.push('\n');
                        i += 2;
                    }
                    _ => {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

fn latex_cmd_to_unicode(cmd: &str) -> Option<&'static str> {
    match cmd {
        // Arrows
        "\\Leftrightarrow" | "\\iff" => Some("⇔"),
        "\\Rightarrow" | "\\implies" => Some("⇒"),
        "\\Leftarrow" => Some("⇐"),
        "\\leftrightarrow" => Some("↔"),
        "\\rightarrow" | "\\to" => Some("→"),
        "\\leftarrow" | "\\gets" => Some("←"),
        "\\mapsto" => Some("↦"),
        "\\uparrow" => Some("↑"),
        "\\downarrow" => Some("↓"),

        // Logic & Sets
        "\\forall" => Some("∀"),
        "\\exists" => Some("∃"),
        "\\nexists" => Some("∄"),
        "\\neg" | "\\lnot" => Some("¬"),
        "\\land" => Some("∧"),
        "\\lor" => Some("∨"),
        "\\notin" => Some("∉"),
        "\\in" => Some("∈"),
        "\\ni" | "\\owns" => Some("∋"),
        "\\emptyset" | "\\varnothing" => Some("∅"),
        "\\subseteq" => Some("⊆"),
        "\\subset" => Some("⊂"),
        "\\supseteq" => Some("⊇"),
        "\\supset" => Some("⊃"),
        "\\setminus" => Some("\\"),
        "\\cup" => Some("∪"),
        "\\cap" => Some("∩"),

        // Relations & Comparisons
        "\\approx" => Some("≈"),
        "\\equiv" => Some("≡"),
        "\\sim" => Some("∼"),
        "\\simeq" => Some("≃"),
        "\\cong" => Some("≅"),
        "\\neq" | "\\ne" => Some("≠"),
        "\\leq" | "\\le" => Some("≤"),
        "\\geq" | "\\ge" => Some("≥"),
        "\\ll" => Some("≪"),
        "\\gg" => Some("≫"),
        "\\propto" => Some("∝"),
        "\\perp" | "\\bot" => Some("⊥"),
        "\\parallel" => Some("∥"),
        "\\mid" => Some("|"),

        // Arithmetic & Operations
        "\\times" => Some("×"),
        "\\div" => Some("÷"),
        "\\pm" => Some("±"),
        "\\mp" => Some("∓"),
        "\\cdot" => Some("·"),
        "\\circ" => Some("°"),
        "\\oplus" => Some("⊕"),
        "\\ominus" => Some("⊖"),
        "\\otimes" => Some("⊗"),
        "\\odot" => Some("⊙"),
        "\\ast" | "\\star" => Some("★"),

        // Calculus & Analysis
        "\\infty" => Some("∞"),
        "\\partial" => Some("∂"),
        "\\nabla" => Some("∇"),
        "\\sum" => Some("∑"),
        "\\prod" => Some("∏"),
        "\\oiint" => Some("∯"),
        "\\iiint" => Some("∭"),
        "\\iint" => Some("∬"),
        "\\oint" => Some("∮"),
        "\\int" => Some("∫"),
        "\\hbar" => Some("ℏ"),
        "\\ell" => Some("ℓ"),
        "\\dots" | "\\ldots" => Some("…"),
        "\\cdots" => Some("⋯"),
        "\\vdots" => Some("⋮"),
        "\\ddots" => Some("⋱"),

        // Math functions
        "\\exp" => Some("exp"),
        "\\ln" => Some("ln"),
        "\\log" => Some("log"),
        "\\sin" => Some("sin"),
        "\\cos" => Some("cos"),
        "\\tan" => Some("tan"),
        "\\arcsin" => Some("arcsin"),
        "\\arccos" => Some("arccos"),
        "\\arctan" => Some("arctan"),
        "\\det" => Some("det"),
        "\\arg" => Some("arg"),
        "\\max" => Some("max"),
        "\\min" => Some("min"),

        // Greek uppercase
        "\\Gamma" => Some("Γ"),
        "\\Delta" => Some("Δ"),
        "\\Theta" => Some("Θ"),
        "\\Lambda" => Some("Λ"),
        "\\Xi" => Some("Ξ"),
        "\\Pi" => Some("Π"),
        "\\Sigma" => Some("Σ"),
        "\\Upsilon" => Some("Υ"),
        "\\Phi" => Some("Φ"),
        "\\Psi" => Some("Ψ"),
        "\\Omega" => Some("Ω"),

        // Greek lowercase
        "\\alpha" => Some("α"),
        "\\beta" => Some("β"),
        "\\gamma" => Some("γ"),
        "\\delta" => Some("δ"),
        "\\epsilon" | "\\varepsilon" => Some("ε"),
        "\\zeta" => Some("ζ"),
        "\\eta" => Some("η"),
        "\\theta" | "\\vartheta" => Some("θ"),
        "\\iota" => Some("ι"),
        "\\kappa" => Some("κ"),
        "\\lambda" => Some("λ"),
        "\\mu" => Some("μ"),
        "\\nu" => Some("ν"),
        "\\xi" => Some("ξ"),
        "\\pi" | "\\varpi" => Some("π"),
        "\\rho" | "\\varrho" => Some("ρ"),
        "\\sigma" | "\\varsigma" => Some("σ"),
        "\\tau" => Some("τ"),
        "\\upsilon" => Some("υ"),
        "\\phi" | "\\varphi" => Some("φ"),
        "\\chi" => Some("χ"),
        "\\psi" => Some("ψ"),
        "\\omega" => Some("ω"),

        // Delimiters and helpers
        "\\therefore" => Some("∴"),
        "\\because" => Some("∵"),
        "\\angle" => Some("∠"),
        "\\triangle" => Some("△"),
        "\\langle" => Some("⟨"),
        "\\rangle" => Some("⟩"),
        "\\qquad" => Some("  "),
        "\\quad" => Some(" "),
        "\\left" => Some(""),
        "\\right" => Some(""),
        _ => None,
    }
}

fn extract_brace_group(s: &str, start_idx: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if start_idx >= bytes.len() || bytes[start_idx] != b'{' {
        return None;
    }
    let mut depth = 0;
    let mut i = start_idx;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            depth += 1;
        } else if bytes[i] == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some((&s[start_idx + 1..i], i + 1));
            }
        }
        i += 1;
    }
    None
}

fn replace_fractions(mut s: String) -> String {
    while let Some(pos) = s.find("\\frac") {
        let after_cmd = pos + 5;
        let rem = &s[after_cmd..];
        let trimmed_offset = rem.len() - rem.trim_start().len();
        let num_start = after_cmd + trimmed_offset;
        if let Some((num, den_pos)) = extract_brace_group(&s, num_start) {
            let rem_den = &s[den_pos..];
            let den_trimmed_offset = rem_den.len() - rem_den.trim_start().len();
            let den_start = den_pos + den_trimmed_offset;
            if let Some((den, end_pos)) = extract_brace_group(&s, den_start) {
                let num_fmt = render_latex_to_text(num);
                let den_fmt = render_latex_to_text(den);
                let replacement = format!("({num_fmt})/({den_fmt})");
                s.replace_range(pos..end_pos, &replacement);
                continue;
            }
        }
        break;
    }
    s
}

fn replace_sqrt(mut s: String) -> String {
    while let Some(pos) = s.find("\\sqrt") {
        let after_cmd = pos + 5;
        let rem = &s[after_cmd..];
        let trimmed_offset = rem.len() - rem.trim_start().len();
        let arg_start = after_cmd + trimmed_offset;
        if let Some((arg, end_pos)) = extract_brace_group(&s, arg_start) {
            let arg_fmt = render_latex_to_text(arg);
            let replacement = format!("√({arg_fmt})");
            s.replace_range(pos..end_pos, &replacement);
            continue;
        }
        break;
    }
    s
}

fn replace_text_macros(mut s: String) -> String {
    const MACROS: &[&str] = &[
        "\\text",
        "\\mathbf",
        "\\mathrm",
        "\\mathit",
        "\\boldsymbol",
        "\\bm",
        "\\operatorname",
        "\\mathcal",
        "\\hat",
        "\\bar",
        "\\tilde",
        "\\vec",
        "\\dot",
        "\\ddot",
    ];
    for mac in MACROS {
        while let Some(pos) = s.find(mac) {
            let after_cmd = pos + mac.len();
            let rem = &s[after_cmd..];
            let trimmed_offset = rem.len() - rem.trim_start().len();
            let arg_start = after_cmd + trimmed_offset;
            if let Some((arg, end_pos)) = extract_brace_group(&s, arg_start) {
                let arg_str = arg.to_string();
                s.replace_range(pos..end_pos, &arg_str);
                continue;
            }
            break;
        }
    }
    s
}

fn char_to_superscript(c: char) -> Option<char> {
    match c {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        '+' => Some('⁺'),
        '-' | '−' => Some('⁻'),
        '=' => Some('⁼'),
        '(' => Some('⁽'),
        ')' => Some('⁾'),
        'a' => Some('ᵃ'),
        'b' => Some('ᵇ'),
        'c' => Some('ᶜ'),
        'd' => Some('ᵈ'),
        'e' => Some('ᵉ'),
        'f' => Some('ᶠ'),
        'g' => Some('ᵍ'),
        'h' => Some('ʰ'),
        'i' => Some('ⁱ'),
        'j' => Some('ʲ'),
        'k' => Some('ᵏ'),
        'l' => Some('ˡ'),
        'm' => Some('ᵐ'),
        'n' => Some('ⁿ'),
        'o' => Some('ᵒ'),
        'p' => Some('ᵖ'),
        'r' => Some('ʳ'),
        's' => Some('ˢ'),
        't' => Some('ᵗ'),
        'u' => Some('ᵘ'),
        'v' => Some('ᵛ'),
        'w' => Some('ʷ'),
        'x' | '×' | '*' => Some('ˣ'),
        'y' => Some('ʸ'),
        'z' => Some('ᶻ'),
        'A' => Some('ᴬ'),
        'B' => Some('ᴮ'),
        'D' => Some('ᴰ'),
        'E' => Some('ᴱ'),
        'G' => Some('ᴳ'),
        'H' => Some('ᴴ'),
        'I' => Some('ᴵ'),
        'J' => Some('ᴶ'),
        'K' => Some('ᴷ'),
        'L' => Some('ᴸ'),
        'M' => Some('ᴹ'),
        'N' => Some('ᴺ'),
        'O' => Some('ᴼ'),
        'P' => Some('ᴾ'),
        'R' => Some('ᴿ'),
        'T' => Some('ᵀ'),
        'U' => Some('ᵁ'),
        'V' => Some('ⱽ'),
        'W' => Some('ᵂ'),
        '∞' => Some('∞'),
        ' ' => Some(' '),
        _ => None,
    }
}

fn char_to_subscript(c: char) -> Option<char> {
    match c {
        '0' => Some('₀'),
        '1' => Some('₁'),
        '2' => Some('₂'),
        '3' => Some('₃'),
        '4' => Some('₄'),
        '5' => Some('₅'),
        '6' => Some('₆'),
        '7' => Some('₇'),
        '8' => Some('₈'),
        '9' => Some('₉'),
        '+' => Some('₊'),
        '-' | '−' => Some('₋'),
        '=' => Some('₌'),
        '(' => Some('₍'),
        ')' => Some('₎'),
        'a' => Some('ₐ'),
        'e' => Some('ₑ'),
        'h' => Some('ₕ'),
        'i' => Some('ᵢ'),
        'j' => Some('ⱼ'),
        'k' => Some('ₖ'),
        'l' => Some('ₗ'),
        'm' => Some('ₘ'),
        'n' => Some('ₙ'),
        'o' => Some('ₒ'),
        'p' => Some('ₚ'),
        'r' => Some('ᵣ'),
        's' => Some('ₛ'),
        't' => Some('ₜ'),
        'u' => Some('ᵤ'),
        'v' => Some('ᵥ'),
        'x' | '×' | '*' => Some('ₓ'),
        ' ' => Some(' '),
        _ => None,
    }
}

fn is_single_script_char(c: char) -> bool {
    c.is_alphanumeric() || c == '+' || c == '-' || c == '−'
}

fn extract_brace_group_chars(chars: &[char]) -> Option<(&[char], usize)> {
    if chars.is_empty() || chars[0] != '{' {
        return None;
    }
    let mut depth = 0;
    for (idx, &c) in chars.iter().enumerate() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some((&chars[1..idx], idx + 1));
            }
        }
    }
    None
}

fn replace_scripts(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '^' && i + 1 < len {
            if chars[i + 1] == '{' {
                if let Some((inner_chars, end_len)) = extract_brace_group_chars(&chars[i + 1..]) {
                    if let Some(super_str) = inner_chars
                        .iter()
                        .copied()
                        .map(char_to_superscript)
                        .collect::<Option<String>>()
                    {
                        out.push_str(&super_str);
                        i += 1 + end_len;
                        continue;
                    } else {
                        out.push('^');
                        for &c in inner_chars {
                            out.push(c);
                        }
                        i += 1 + end_len;
                        continue;
                    }
                }
            } else if is_single_script_char(chars[i + 1])
                && let Some(super_c) = char_to_superscript(chars[i + 1])
            {
                out.push(super_c);
                i += 2;
                continue;
            }
        } else if chars[i] == '_' && i + 1 < len {
            if chars[i + 1] == '{' {
                if let Some((inner_chars, end_len)) = extract_brace_group_chars(&chars[i + 1..]) {
                    if let Some(sub_str_converted) = inner_chars
                        .iter()
                        .copied()
                        .map(char_to_subscript)
                        .collect::<Option<String>>()
                    {
                        out.push_str(&sub_str_converted);
                        i += 1 + end_len;
                        continue;
                    } else {
                        out.push('_');
                        for &c in inner_chars {
                            out.push(c);
                        }
                        i += 1 + end_len;
                        continue;
                    }
                }
            } else if is_single_script_char(chars[i + 1])
                && let Some(sub_c) = char_to_subscript(chars[i + 1])
            {
                out.push(sub_c);
                i += 2;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}
