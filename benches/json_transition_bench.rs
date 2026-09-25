use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kglance::core::preview::PreviewData;
use kglance::core::types::KglanceState;
use kglance::features::json::parser::JsonParser;
use kglance::features::json::state::populate_state as populate_json_state;
use kglance::features::json::view::tree::render_tree;
use kglance::features::json::view_json;
use kglance::features::text::populate_state as populate_text_state;
use kglance::ui::theme::AppTheme;
use serde_json::json;

fn generate_sample_json(depth: usize, items_per_level: usize) -> serde_json::Value {
    if depth == 0 {
        return json!({
            "id": 101,
            "title": "Kglance Performance Benchmark Item",
            "active": true,
            "count": 42,
            "score": 98.75,
            "tags": ["rust", "iced", "benchmark", "fast"]
        });
    }

    let mut obj = serde_json::Map::new();
    for i in 0..items_per_level {
        let key = format!("node_{depth}_{i}");
        if i % 2 == 0 {
            obj.insert(
                key,
                generate_sample_json(depth.saturating_sub(1), items_per_level),
            );
        } else {
            let arr: Vec<serde_json::Value> = (0..items_per_level)
                .map(|_| generate_sample_json(depth.saturating_sub(1), items_per_level))
                .collect();
            obj.insert(key, serde_json::Value::Array(arr));
        }
    }
    serde_json::Value::Object(obj)
}

fn bench_json_parser_and_flatten(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_and_flatten");

    for &(depth, items, label) in &[
        (2, 3, "small_~50_nodes"),
        (3, 4, "medium_~500_nodes"),
        (4, 4, "large_~2500_nodes"),
    ] {
        let val = generate_sample_json(depth, items);
        let raw_str = serde_json::to_string_pretty(&val).unwrap();

        group.bench_with_input(
            BenchmarkId::new("parse_content", label),
            &raw_str,
            |b, s| {
                b.iter(|| {
                    let (mut nodes, pretty, has_err) = JsonParser::parse_content(s, "json");
                    JsonParser::assign_parent_indices(&mut nodes);
                    (nodes, pretty, has_err)
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("flatten_only", label), &val, |b, v| {
            b.iter(|| {
                let mut nodes = JsonParser::flatten_json(v, None, 0);
                JsonParser::assign_parent_indices(&mut nodes);
                nodes
            })
        });
    }
    group.finish();
}

fn bench_json_populate_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/populate_state");

    for &(depth, items, label) in &[
        (2, 3, "small_~50_nodes"),
        (3, 4, "medium_~500_nodes"),
        (4, 4, "large_~2500_nodes"),
    ] {
        let val = generate_sample_json(depth, items);
        let raw_str = serde_json::to_string_pretty(&val).unwrap();
        let (mut nodes, pretty, has_err) = JsonParser::parse_content(&raw_str, "json");
        JsonParser::assign_parent_indices(&mut nodes);

        group.bench_with_input(
            BenchmarkId::new("tree_mode_true", label),
            &(&nodes, &pretty, has_err),
            |b, &(nds, prt, err)| {
                b.iter(|| {
                    let mut state = KglanceState::default();
                    state.json.tree_mode = true;
                    populate_json_state(&mut state, nds, prt, err);
                    state
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("tree_mode_false_editor_content", label),
            &(&nodes, &pretty, has_err),
            |b, &(nds, prt, err)| {
                b.iter(|| {
                    let mut state = KglanceState::default();
                    state.json.tree_mode = false;
                    populate_json_state(&mut state, nds, prt, err);
                    state
                })
            },
        );
    }
    group.finish();
}

fn bench_json_view_tree_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view_tree_generation");

    for &(depth, items, label) in &[
        (2, 3, "small_~50_nodes"),
        (3, 4, "medium_~500_nodes"),
        (4, 4, "large_~2500_nodes"),
    ] {
        let val = generate_sample_json(depth, items);
        let raw_str = serde_json::to_string_pretty(&val).unwrap();
        let (mut nodes, pretty, has_err) = JsonParser::parse_content(&raw_str, "json");
        JsonParser::assign_parent_indices(&mut nodes);

        let mut state = KglanceState::default();
        state.json.tree_mode = true;
        populate_json_state(&mut state, &nodes, &pretty, has_err);

        group.bench_with_input(
            BenchmarkId::new("render_tree_expanded", label),
            &state.json,
            |b, json_state| b.iter(|| render_tree(json_state, AppTheme::Light, 14.0)),
        );

        // Fake Virtualization experiment: Only project/build 50 visible rows
        group.bench_with_input(
            BenchmarkId::new("render_tree_virtual_window_50_nodes", label),
            &state.json,
            |b, json_state| {
                b.iter(|| {
                    let indices =
                        kglance::features::json::view::tree::visible_node_indices(json_state);
                    let window_indices = &indices[..indices.len().min(50)];
                    let nodes: Vec<iced::Element<'_, kglance::app::Message>> = window_indices
                        .iter()
                        .map(|&i| {
                            let node = &json_state.nodes[i];
                            let expanded = json_state.expanded.contains(&i);
                            let is_active = json_state.active_node == Some(i);
                            kglance::features::json::view::tree::render_tree_node(
                                i,
                                node,
                                kglance::features::json::view::tree::TreeNodeOptions {
                                    theme: AppTheme::Light,
                                    font_size: 14.0,
                                    is_expanded: expanded,
                                    is_active,
                                    search_query: "",
                                },
                            )
                        })
                        .collect();
                    iced::widget::column(nodes)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("view_json_full_layout", label),
            &state.json,
            |b, json_state| {
                b.iter(|| view_json(json_state, 14.0, AppTheme::Light, Some("monospace"), false))
            },
        );
    }
    group.finish();
}

fn bench_json_transition_switch(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/navigation_transition");

    let val = generate_sample_json(3, 4);
    let json_str = serde_json::to_string_pretty(&val).unwrap();
    let (mut nodes, pretty, has_err) = JsonParser::parse_content(&json_str, "json");
    JsonParser::assign_parent_indices(&mut nodes);

    let json_preview = PreviewData::Json {
        nodes: nodes.clone(),
        content: json_str.clone(),
        pretty: pretty.clone(),
        has_parse_error: has_err,
    };

    let text_content = "fn main() {\n    println!(\"Switching between files\");\n}\n".repeat(50);
    let text_lines = (1..=150)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let text_preview = PreviewData::Text {
        content: text_content.clone(),
        line_numbers: text_lines.clone(),
        language: "Rust".into(),
    };

    // Benchmark 1: Simulating transition Json -> Text
    group.bench_function("transition_json_to_text", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();
            // Step 1: In JSON state
            populate_json_state(&mut state, &nodes, &pretty, has_err);
            // Step 2: Transition to Text
            populate_text_state(&mut state, text_content.clone(), text_lines.clone(), "Rust");
            state
        })
    });

    // Benchmark 2: Simulating transition Text -> Json
    group.bench_function("transition_text_to_json", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();
            // Step 1: In Text state
            populate_text_state(&mut state, text_content.clone(), text_lines.clone(), "Rust");
            // Step 2: Transition to Json
            populate_json_state(&mut state, &nodes, &pretty, has_err);
            state
        })
    });

    // Benchmark 3: Simulating PreviewData::populate_state dispatch on navigation
    group.bench_function("populate_state_dispatch_json", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();
            json_preview.populate_state(&mut state);
            state
        })
    });

    group.bench_function("populate_state_dispatch_text", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();
            text_preview.populate_state(&mut state);
            state
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_parser_and_flatten,
    bench_json_populate_state,
    bench_json_view_tree_generation,
    bench_json_transition_switch,
);
criterion_main!(benches);
