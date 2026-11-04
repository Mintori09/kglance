pub fn strip_ansi_codes(input: &str) -> String {
    if !input.contains('\x1b') {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next(); // consume '['
                    // CSI sequence: consumes parameters until final byte (0x40..=0x7E)
                    for next_ch in chars.by_ref() {
                        if (0x40..=0x7E).contains(&(next_ch as u32)) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next(); // consume ']'
                    // OSC sequence: ends with BEL ('\x07') or ST ('\x1b\\')
                    while let Some(next_ch) = chars.next() {
                        if next_ch == '\x07' {
                            break;
                        }
                        if next_ch == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some('(' | ')' | '*' | '+') => {
                    chars.next();
                    chars.next();
                }
                _ => {}
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_clean_string() {
        assert_eq!(strip_ansi_codes("Hello World"), "Hello World");
    }

    #[test]
    fn test_strip_ansi_color_codes() {
        let text = "\x1b[0;31mTraceback (most recent call last):\x1b[0m\n\x1b[1;32mZeroDivisionError\x1b[0m";
        assert_eq!(
            strip_ansi_codes(text),
            "Traceback (most recent call last):\nZeroDivisionError"
        );
    }

    #[test]
    fn test_strip_ansi_bold_underline() {
        let text = "\x1b[1m\x1b[4mUnderlined bold\x1b[0m normal";
        assert_eq!(strip_ansi_codes(text), "Underlined bold normal");
    }
}
