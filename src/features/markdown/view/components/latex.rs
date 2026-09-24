use super::latex_commands::latex_cmd_to_unicode;

pub(crate) fn render_latex_to_text(latex: &str) -> String {
    if !latex.contains('\\') && !latex.contains('_') && !latex.contains('^') && !latex.contains('&')
    {
        return latex.to_string();
    }
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
