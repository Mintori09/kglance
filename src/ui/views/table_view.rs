use crate::app::Message;
use crate::core::{SortField, TableState};
use crate::ui::theme::{breeze_button, glass_card, glass_scrollable};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

pub fn view_table<'a>(state: &'a TableState) -> Element<'a, Message> {
    // Header row with sort buttons, styled as raised glass panel.
    let header = container(
        row![
            button(text("Name"))
                .on_press(Message::SortByFieldClicked(SortField::Name))
                .style(breeze_button)
                .width(Length::FillPortion(3)),
            button(text("Kind"))
                .on_press(Message::SortByFieldClicked(SortField::Kind))
                .style(breeze_button)
                .width(Length::FillPortion(1)),
            button(text("Size"))
                .on_press(Message::SortByFieldClicked(SortField::Size))
                .style(breeze_button)
                .width(Length::FillPortion(1)),
            button(text("Modified"))
                .on_press(Message::SortByFieldClicked(SortField::Modified))
                .style(breeze_button)
                .width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .padding(5),
    )
    .style(glass_card)
    .padding([4, 8]);

    let mut rows_list = column![].spacing(4);
    for (idx, row_data) in state.rows.iter().enumerate() {
        let icon = if row_data.is_dir { "Dir" } else { "File" };
        let name_widget = button(text(format!("[{}] {}", icon, row_data.name)))
            .on_press(Message::FileClicked(idx))
            .style(breeze_button);

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

    let main_view = column![
        header,
        scrollable(rows_list)
            .style(glass_scrollable)
            .height(Length::Fill)
    ]
    .spacing(6);

    main_view.into()
}
