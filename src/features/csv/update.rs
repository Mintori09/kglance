use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_sheet_tab_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if index < app.state.spreadsheet.sheets.len() {
        app.state.spreadsheet.active_sheet = index;
        app.state.spreadsheet.sort_col = None;
        app.state.spreadsheet.sort_ascending = None;
    }
    Task::none()
}

pub fn handle_column_clicked(app: &mut KglanceApp, col: usize) -> Task<Message> {
    let sort = &mut app.state.spreadsheet;
    if sort.sort_col == Some(col) {
        sort.sort_ascending = match sort.sort_ascending {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };
        if sort.sort_ascending.is_none() {
            sort.sort_col = None;
        }
    } else {
        sort.sort_col = Some(col);
        sort.sort_ascending = Some(true);
    }
    Task::none()
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.spreadsheet.search_query = query;
    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.spreadsheet.search_visible = false;
    app.state.spreadsheet.search_query.clear();
    Task::none()
}
