use super::*;

#[test]
fn test_sanitize_for_iced_math() {
    let input = r"x && y \text{hello world} z";
    let sanitized = sanitize_for_iced_math(input);
    assert_eq!(sanitized, r"x \quad  y \text{hello world} z");

    // Vietnamese diacritics in \text{...} transliterated to ASCII
    let non_ascii = r"\text{chào thế giới} + x";
    let sanitized_non_ascii = sanitize_for_iced_math(non_ascii);
    assert_eq!(sanitized_non_ascii, r"\text{chao the gioi} + x");

    // Non-Vietnamese unconvertible non-ascii (like CJK/emoji) stripped if not ASCII
    let cjk = r"\text{你好世界} + y";
    let sanitized_cjk = sanitize_for_iced_math(cjk);
    assert_eq!(sanitized_cjk, " + y");

    // \tag{...} stripped in sanitize_for_iced_math
    let with_tag = r"\begin{aligned} a &= b \end{aligned} \tag{3.1}";
    let sanitized_tag = sanitize_for_iced_math(with_tag);
    assert_eq!(
        sanitized_tag.trim(),
        r"\begin{aligned} a &= b \end{aligned}"
    );
}

#[test]
fn test_desugar_substack() {
    let input = r"V(k) = \max_{\substack{c, k' \\ c + k' \le f(k)}} \{ u(c) + \beta V(k') \}";
    let desugared = desugar_substack(input);
    assert_eq!(
        desugared,
        r"V(k) = \max_{\begin{matrix}c, k' \\ c + k' \le f(k)\end{matrix}} \{ u(c) + \beta V(k') \}"
    );

    // Literal \substack without brace should be preserved
    assert_eq!(desugar_substack(r"\substack foo"), r"\substack foo");
}

#[test]
fn test_complex_equation_bellman_with_aligned_and_substack() {
    // User equation 1: Bellman equation
    let equation = r"\begin{aligned}
V(k_t) = \max_{\substack{c_t, k_{t+1} \\ c_t + k_{t+1} \le f(k_t) + (1-\delta)k_t}} \left\{ u(c_t) + \beta V(k_{t+1}) \right\}
\end{aligned}";

    let (cleaned, _) = extract_tag_from_latex(equation);
    let res = get_or_render_math_svg(&cleaned, 16.0, (255, 255, 255));
    assert!(
        matches!(res, CachedMathResult::Success(_)),
        "Bellman equation with aligned and substack should render successfully to SVG"
    );
}

#[test]
fn test_complex_equation_euler_with_vietnamese_substack_and_tag() {
    // User equation 2: Euler equation with underbrace, substack, vietnamese text, and tag
    let equation = r"u'(c_t) = \beta \mathbb{E}_t \left[ u'(c_{t+1}) (1 + r_{t+1} - \delta) \right] \underbrace{=}_{\substack{\text{Chi phí} \\ \text{thỏa dụng}}} 0 \tag{3.2}";

    let (cleaned, tag) = extract_tag_from_latex(equation);
    assert_eq!(tag, Some("(3.2)".to_string()));

    let res = get_or_render_math_svg(&cleaned, 16.0, (255, 255, 255));
    assert!(
        matches!(res, CachedMathResult::Success(_)),
        "Euler equation with underbrace, substack, vietnamese annotations, and tag should render successfully to SVG"
    );
}

#[test]
fn test_extract_tag_from_latex() {
    let formula_with_tag = r"\begin{aligned} y &= f(k) \end{aligned} \tag{3.1}";
    let (cleaned, tag) = extract_tag_from_latex(formula_with_tag);
    assert_eq!(cleaned.trim(), r"\begin{aligned} y &= f(k) \end{aligned}");
    assert_eq!(tag, Some("(3.1)".to_string()));

    // Starred tag \tag*{3.1} should not add surrounding parentheses
    let formula_with_starred_tag = r"k_{t+1} - k_t = s f(k_t) \tag*{[Eq. 1]}";
    let (cleaned_star, tag_star) = extract_tag_from_latex(formula_with_starred_tag);
    assert_eq!(cleaned_star.trim(), r"k_{t+1} - k_t = s f(k_t)");
    assert_eq!(tag_star, Some("[Eq. 1]".to_string()));

    // Formula without tag
    let formula_no_tag = r"E = mc^2";
    let (cleaned_no_tag, tag_none) = extract_tag_from_latex(formula_no_tag);
    assert_eq!(cleaned_no_tag, "E = mc^2");
    assert_eq!(tag_none, None);
}

#[test]
fn test_aligned_with_tag_renders_successfully() {
    let formula = r"\begin{aligned}
y_t &= f(k_t) \\
k_{t+1} &= (1 - \delta) k_t + s f(k_t)
\end{aligned} \tag{3.1}";

    let (cleaned, tag) = extract_tag_from_latex(formula);
    assert_eq!(tag, Some("(3.1)".to_string()));

    let res = get_or_render_math_svg(&cleaned, 16.0, (255, 255, 255));
    assert!(
        matches!(res, CachedMathResult::Success(_)),
        "Formula with aligned and tag should render successfully to SVG without error"
    );
}

#[test]
fn test_enhance_svg_stroke() {
    let fake_svg = br##"<svg><g fill="#ffffff"><path d="M0 0"/></g></svg>"##;
    let enhanced = enhance_svg_stroke(fake_svg, "#ffffff", 0.35);
    let enhanced_str = String::from_utf8(enhanced).expect("valid utf-8");
    assert!(enhanced_str.contains(r##"stroke="#ffffff""##));
    assert!(enhanced_str.contains(r#"stroke-width="0.35""#));
    assert!(enhanced_str.contains(r#"stroke-linejoin="round""#));
}

#[test]
fn test_math_cache_hit_and_miss() {
    reset_math_cache();

    let formula = r"E = mc^2";
    let font_size = 16.0;
    let color = (255, 255, 255);

    // Initial render -> cache miss
    let res1 = get_or_render_math_svg(formula, font_size, color);
    assert!(matches!(res1, CachedMathResult::Success(_)));
    let (hits1, misses1) = math_cache_stats();
    assert_eq!(hits1, 0);
    assert_eq!(misses1, 1);

    // Repeated render with same parameters -> cache hit
    let res2 = get_or_render_math_svg(formula, font_size, color);
    assert_eq!(res1, res2);
    let (hits2, misses2) = math_cache_stats();
    assert_eq!(hits2, 1);
    assert_eq!(misses2, 1);

    // Render with different color -> cache miss
    let res3 = get_or_render_math_svg(formula, font_size, (200, 200, 200));
    assert!(matches!(res3, CachedMathResult::Success(_)));
    let (hits3, misses3) = math_cache_stats();
    assert_eq!(hits3, 1);
    assert_eq!(misses3, 2);

    // Render with different font size -> cache miss
    let res4 = get_or_render_math_svg(formula, 18.0, color);
    assert!(matches!(res4, CachedMathResult::Success(_)));
    let (hits4, misses4) = math_cache_stats();
    assert_eq!(hits4, 1);
    assert_eq!(misses4, 3);

    // Re-render original -> cache hit
    let res5 = get_or_render_math_svg(formula, font_size, color);
    assert_eq!(res1, res5);
    let (hits5, misses5) = math_cache_stats();
    assert_eq!(hits5, 2);
    assert_eq!(misses5, 3);
}

#[test]
fn test_math_cache_error_handling() {
    reset_math_cache();

    // Invalid LaTeX formula that fails to render in iced_math
    let invalid_latex = r"\begin{invalid_env_that_does_not_exist";
    let res1 = get_or_render_math_svg(invalid_latex, 14.0, (255, 0, 0));
    assert!(matches!(res1, CachedMathResult::Error(_)));

    let (hits1, misses1) = math_cache_stats();
    assert_eq!(hits1, 0);
    assert_eq!(misses1, 1);

    // Repeated call should hit cache for error result as well
    let res2 = get_or_render_math_svg(invalid_latex, 14.0, (255, 0, 0));
    assert_eq!(res1, res2);

    let (hits2, misses2) = math_cache_stats();
    assert_eq!(hits2, 1);
    assert_eq!(misses2, 1);
}
