use super::*;
use crate::app::test_util::test_app;
use crate::features::notebook::types::{CellOutput, NotebookCell, NotebookData};

#[test]
fn test_collect_notebook_lightbox_items_from_code_output() {
    let dummy_handle = iced::widget::image::Handle::from_bytes(vec![1, 2, 3]);
    let data = NotebookData {
        cells: vec![
            NotebookCell::Markdown {
                source: "# Title".to_string(),
                blocks: vec![],
            },
            NotebookCell::Code {
                source: "import matplotlib".to_string(),
                execution_count: Some(5),
                outputs: vec![
                    CellOutput::Text {
                        text: "Plotting...".to_string(),
                        is_error: false,
                        truncated: false,
                    },
                    CellOutput::Image {
                        handle: dummy_handle,
                        data: vec![137, 80, 78, 71], // PNG header
                        format_hint: "png".to_string(),
                    },
                ],
                language: Some("python".to_string()),
            },
        ],
        kernel_language: "python".to_string(),
        total_cells: 2,
        fallback_notice: None,
    };

    let cached_handles = std::collections::HashMap::new();
    let items = collect_notebook_lightbox_items(&data, &cached_handles);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "cell_1_out_1");
    assert_eq!(items[0].title.as_deref(), Some("Out [5]"));
    assert_eq!(items[0].subtitle, "Cell 2 Output (png)");
    assert_eq!(
        items[0].in_memory_bytes.as_deref(),
        Some(&[137, 80, 78, 71][..])
    );
}

#[test]
fn test_handle_open_notebook_image_lightbox() {
    let dummy_handle = iced::widget::image::Handle::from_bytes(vec![1, 2, 3]);
    let data = NotebookData {
        cells: vec![NotebookCell::Code {
            source: "plt.show()".to_string(),
            execution_count: Some(1),
            outputs: vec![CellOutput::Image {
                handle: dummy_handle,
                // Valid 1x1 PNG bytes for lightbox decode
                data: vec![
                    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                    0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156,
                    99, 248, 207, 192, 240, 31, 0, 5, 0, 1, 255, 137, 145, 12, 138, 0, 0, 0, 0, 73,
                    69, 78, 68, 174, 66, 96, 130,
                ],
                format_hint: "png".to_string(),
            }],
            language: Some("python".to_string()),
        }],
        kernel_language: "python".to_string(),
        total_cells: 1,
        fallback_notice: None,
    };

    let mut app = test_app(Some(crate::core::PreviewData::Notebook(data)));
    assert!(app.state.lightbox.is_none());

    let _ = handle_open_notebook_image_lightbox(&mut app, "cell_0_out_0");

    assert!(app.state.lightbox.is_some());
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 0);
    assert_eq!(lb.items.len(), 1);
    assert_eq!(lb.items[0].id, "cell_0_out_0");
}
