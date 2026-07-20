use crate::ui::types::{Message, SortField, TableState};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

pub fn view_table<'a>(state: &'a TableState) -> Element<'a, Message> {
    // Header row with sorting buttons
    let header = row![
        button(text("Name"))
            .on_press(Message::SortByFieldClicked(SortField::Name))
            .style(crate::ui::theme::breeze_button)
            .width(Length::FillPortion(3)),
        button(text("Kind"))
            .on_press(Message::SortByFieldClicked(SortField::Kind))
            .style(crate::ui::theme::breeze_button)
            .width(Length::FillPortion(1)),
        button(text("Size"))
            .on_press(Message::SortByFieldClicked(SortField::Size))
            .style(crate::ui::theme::breeze_button)
            .width(Length::FillPortion(1)),
        button(text("Modified"))
            .on_press(Message::SortByFieldClicked(SortField::Modified))
            .style(crate::ui::theme::breeze_button)
            .width(Length::FillPortion(2)),
    ]
    .spacing(10)
    .padding(5);

    // List of rows
    let mut rows_list = column![].spacing(5);
    for (idx, row_data) in state.rows.iter().enumerate() {
        let name_widget = if row_data.is_dir {
            button(text(format!("📁 {}", row_data.name)))
                .on_press(Message::FileClicked(idx))
                .style(crate::ui::theme::breeze_button)
        } else {
            button(text(format!("📄 {}", row_data.name)))
                .on_press(Message::FileClicked(idx))
                .style(crate::ui::theme::breeze_button)
        };

        let row_item = row![
            container(name_widget).width(Length::FillPortion(3)),
            container(text(&row_data.kind)).width(Length::FillPortion(1)),
            container(text(&row_data.size)).width(Length::FillPortion(1)),
            container(text(&row_data.modified)).width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .padding(5);

        rows_list = rows_list.push(row_item);
    }

    let main_view = column![header, scrollable(rows_list).height(Length::Fill)].spacing(5);

    main_view.into()
}
