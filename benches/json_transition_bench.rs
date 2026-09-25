use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kglance::core::preview::PreviewData;
use kglance::core::types::KglanceState;
use kglance::features::json::parser::JsonParser;
use kglance::features::json::state::populate_state as populate_json_state;
use kglance::features::json::view::tree::{
    TreeNodeOptions, render_tree, render_tree_node, visible_node_indices,
};
use kglance::features::json::view_json;
use kglance::features::text::populate_state as populate_text_state;
use kglance::ui::theme::AppTheme;
use serde_json::json;
use std::hint::black_box;

const FONT_SIZE: f32 = 14.0;
const ROW_HEIGHT: f32 = 24.0;
const THEME: AppTheme = AppTheme::Light;

#[derive(Clone, Copy)]
enum ExpansionMode {
    Collapsed,
    Root,
    Partial,
    Full,
}

impl ExpansionMode {
    fn label(self) -> &'static str {
        match self {
            Self::Collapsed => "collapsed",
            Self::Root => "root_expanded",
            Self::Partial => "partial_expanded",
            Self::Full => "fully_expanded",
        }
    }
}

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

fn prepare_json_state(depth: usize, items: usize, expansion: ExpansionMode) -> KglanceState {
    let value = generate_sample_json(depth, items);
    let raw = serde_json::to_string_pretty(&value).unwrap();

    let (mut nodes, pretty, has_err) = JsonParser::parse_content(&raw, "json");

    JsonParser::assign_parent_indices(&mut nodes);

    let mut state = KglanceState::default();
    state.json.tree_mode = true;

    populate_json_state(&mut state, &nodes, &pretty, has_err);

    match expansion {
        ExpansionMode::Collapsed => {
            state.json.expanded.clear();
        }

        ExpansionMode::Root => {
            state.json.expanded.clear();

            if !state.json.nodes.is_empty() {
                state.json.expanded.insert(0);
            }
        }

        ExpansionMode::Partial => {
            state.json.expanded.clear();

            for index in 0..state.json.nodes.len() {
                if index % 3 == 0 {
                    state.json.expanded.insert(index);
                }
            }
        }

        ExpansionMode::Full => {
            state.json.expanded.clear();

            for index in 0..state.json.nodes.len() {
                state.json.expanded.insert(index);
            }
        }
    }

    state
}

fn bench_json_parser_and_flatten(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_and_flatten");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let value = generate_sample_json(depth, items);
        let raw = serde_json::to_string_pretty(&value).unwrap();

        group.bench_with_input(
            BenchmarkId::new("parse_content", label),
            &raw,
            |b, input| {
                b.iter(|| {
                    let (mut nodes, pretty, has_err) =
                        JsonParser::parse_content(black_box(input), "json");

                    JsonParser::assign_parent_indices(&mut nodes);

                    black_box((nodes, pretty, has_err))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("flatten_only", label),
            &value,
            |b, input| {
                b.iter(|| {
                    let mut nodes = JsonParser::flatten_json(black_box(input), None, 0);

                    JsonParser::assign_parent_indices(&mut nodes);

                    black_box(nodes)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("assign_parent_indices", label),
            &value,
            |b, input| {
                let nodes = JsonParser::flatten_json(input, None, 0);

                b.iter(|| {
                    let mut nodes = nodes.clone();

                    JsonParser::assign_parent_indices(black_box(&mut nodes));

                    black_box(nodes)
                })
            },
        );
    }

    group.finish();
}

fn bench_json_populate_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/populate_state");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let value = generate_sample_json(depth, items);
        let raw = serde_json::to_string_pretty(&value).unwrap();

        let (mut nodes, pretty, has_err) = JsonParser::parse_content(&raw, "json");

        JsonParser::assign_parent_indices(&mut nodes);

        group.bench_with_input(
            BenchmarkId::new("tree_mode_true", label),
            &(&nodes, &pretty, has_err),
            |b, &(nodes, pretty, has_err)| {
                b.iter(|| {
                    let mut state = KglanceState::default();
                    state.json.tree_mode = true;

                    populate_json_state(
                        &mut state,
                        black_box(nodes),
                        black_box(pretty),
                        black_box(has_err),
                    );

                    black_box(state)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("tree_mode_false_editor_content", label),
            &(&nodes, &pretty, has_err),
            |b, &(nodes, pretty, has_err)| {
                b.iter(|| {
                    let mut state = KglanceState::default();
                    state.json.tree_mode = false;

                    populate_json_state(
                        &mut state,
                        black_box(nodes),
                        black_box(pretty),
                        black_box(has_err),
                    );

                    black_box(state)
                })
            },
        );
    }

    group.finish();
}

fn bench_visible_node_indices(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/visible_node_indices");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        for expansion in [
            ExpansionMode::Collapsed,
            ExpansionMode::Root,
            ExpansionMode::Partial,
            ExpansionMode::Full,
        ] {
            let state = prepare_json_state(depth, items, expansion);

            let benchmark_name = format!("{label}/{}", expansion.label());

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| b.iter(|| black_box(visible_node_indices(black_box(json_state)))),
            );
        }
    }

    group.finish();
}

fn bench_render_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/render_tree");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        for expansion in [
            ExpansionMode::Collapsed,
            ExpansionMode::Root,
            ExpansionMode::Partial,
            ExpansionMode::Full,
        ] {
            let state = prepare_json_state(depth, items, expansion);

            let benchmark_name = format!("{label}/{}", expansion.label());

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| {
                    b.iter(|| black_box(render_tree(black_box(json_state), THEME, FONT_SIZE)))
                },
            );
        }
    }

    group.finish();
}

fn bench_render_tree_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/render_tree_node");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let state = prepare_json_state(depth, items, ExpansionMode::Full);

        for &node_index in [
            0,
            state.json.nodes.len() / 2,
            state.json.nodes.len().saturating_sub(1),
        ]
        .iter()
        {
            if node_index >= state.json.nodes.len() {
                continue;
            }

            let node = &state.json.nodes[node_index];
            let expanded = state.json.expanded.contains(&node_index);
            let is_active = state.json.active_node == Some(node_index);

            let name = format!("{label}/node_{node_index}");

            group.bench_with_input(
                BenchmarkId::from_parameter(name),
                &(node_index, node, expanded, is_active),
                |b, &(index, node, expanded, is_active)| {
                    b.iter(|| {
                        black_box(render_tree_node(
                            index,
                            black_box(node),
                            TreeNodeOptions {
                                theme: THEME,
                                font_size: FONT_SIZE,
                                row_height: ROW_HEIGHT,
                                is_expanded: expanded,
                                is_active,
                                search_query: "",
                            },
                        ))
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_virtual_window(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/virtual_window");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let state = prepare_json_state(depth, items, ExpansionMode::Full);

        for window_size in [20usize, 50, 100, 250, 500, 1000] {
            let benchmark_name = format!("{label}/{window_size}_rows");

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| {
                    b.iter(|| {
                        let indices = visible_node_indices(black_box(json_state));

                        let limit = indices.len().min(window_size);

                        let window_indices = &indices[..limit];

                        let elements: Vec<iced::Element<'_, kglance::app::Message>> =
                            window_indices
                                .iter()
                                .map(|&index| {
                                    let node = &json_state.nodes[index];

                                    let expanded = json_state.expanded.contains(&index);

                                    let is_active = json_state.active_node == Some(index);

                                    render_tree_node(
                                        index,
                                        node,
                                        TreeNodeOptions {
                                            theme: THEME,
                                            font_size: FONT_SIZE,
                                            row_height: ROW_HEIGHT,
                                            is_expanded: expanded,
                                            is_active,
                                            search_query: "",
                                        },
                                    )
                                })
                                .collect();

                        black_box(iced::widget::column(elements))
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_virtual_window_build_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/virtual_window/build_only");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let state = prepare_json_state(depth, items, ExpansionMode::Full);

        for window_size in [20usize, 50, 100, 250, 500, 1000] {
            let benchmark_name = format!("{label}/{window_size}_rows");

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| {
                    b.iter(|| {
                        let indices = visible_node_indices(black_box(json_state));

                        let limit = indices.len().min(window_size);

                        black_box(&indices[..limit]);
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_view_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/view_json");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        for expansion in [
            ExpansionMode::Collapsed,
            ExpansionMode::Root,
            ExpansionMode::Partial,
            ExpansionMode::Full,
        ] {
            let state = prepare_json_state(depth, items, expansion);

            let benchmark_name = format!("{label}/{}", expansion.label());

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| {
                    b.iter(|| {
                        black_box(view_json(
                            black_box(json_state),
                            FONT_SIZE,
                            THEME,
                            Some("monospace"),
                            false,
                        ))
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_json_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/view/search");

    for &(depth, items, label) in &[
        (2, 3, "small"),
        (3, 4, "medium"),
        (4, 4, "large"),
        (5, 4, "xlarge"),
    ] {
        let mut state = prepare_json_state(depth, items, ExpansionMode::Full);

        for query in ["", "node", "Kglance", "benchmark", "unlikely_query"] {
            state.json.search_query = query.to_string();

            let query_label = match query {
                "" => "empty",
                "node" => "common",
                "Kglance" => "rare",
                "benchmark" => "leaf_match",
                _ => "no_match",
            };

            let benchmark_name = format!("{label}/{query_label}");

            group.bench_with_input(
                BenchmarkId::from_parameter(benchmark_name),
                &state.json,
                |b, json_state| {
                    b.iter(|| {
                        black_box(view_json(
                            black_box(json_state),
                            FONT_SIZE,
                            THEME,
                            Some("monospace"),
                            false,
                        ))
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_json_transition_switch(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/navigation_transition");

    let value = generate_sample_json(3, 4);
    let json_str = serde_json::to_string_pretty(&value).unwrap();

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

    group.bench_function("transition_json_to_text", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();

            populate_json_state(
                &mut state,
                black_box(&nodes),
                black_box(&pretty),
                black_box(has_err),
            );

            populate_text_state(
                &mut state,
                black_box(text_content.clone()),
                black_box(text_lines.clone()),
                "Rust",
            );

            black_box(state)
        })
    });

    group.bench_function("transition_text_to_json", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();

            populate_text_state(
                &mut state,
                black_box(text_content.clone()),
                black_box(text_lines.clone()),
                "Rust",
            );

            populate_json_state(
                &mut state,
                black_box(&nodes),
                black_box(&pretty),
                black_box(has_err),
            );

            black_box(state)
        })
    });

    group.bench_function("populate_state_dispatch_json", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();

            json_preview.populate_state(black_box(&mut state));

            black_box(state)
        })
    });

    group.bench_function("populate_state_dispatch_text", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();

            text_preview.populate_state(black_box(&mut state));

            black_box(state)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_parser_and_flatten,
    bench_json_populate_state,
    bench_visible_node_indices,
    bench_render_tree,
    bench_render_tree_node,
    bench_virtual_window,
    bench_virtual_window_build_only,
    bench_view_json,
    bench_json_search,
    bench_json_transition_switch,
);

criterion_main!(benches);
