use std::sync::{Arc, RwLock};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        palette::
            material::AMBER, Color, Modifier, Style, Stylize
    },
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use std::collections::HashMap;

use crate::{
    InputMode, IssueFragment, LoadingState, LtEvent, TabChangeEvent,
    api::LinearClient,
    iconmap,
    queries::{
        CustomViewQuery, MyIssuesQuery, SearchQuery, custom_view_query, custom_views_query,
        my_issues_query::{self},
        search_query,
    },
};

#[derive(Debug, Default)]
pub struct MyIssuesWidgetState {
    loading_state: LoadingState,
    pub list_state: ListState,
    pub selected_view_id: String,
    pub issue_map: HashMap<String, Vec<IssueFragment>>,
}

#[derive(Debug, Clone, Default)]
pub struct MyIssuesWidget {
    pub state: Arc<RwLock<MyIssuesWidgetState>>,
    input: Input,
    pub search_input_value: String,
    pub show_search_input: bool,
    pub input_mode: InputMode,
}

impl MyIssuesWidget {
    async fn fetch_my_issues(self) {
        self.set_loading_state(LoadingState::Loading);
        self.set_selected_view(String::from("my_issues"));
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = my_issues_query::Variables {};
        match client.query(MyIssuesQuery, variables).await {
            Ok(data) => {
                self.state.write().unwrap().issue_map.insert(
                    String::from("my_issues"),
                    data.issues
                        .nodes
                        .iter()
                        .map(|issue| issue.to_owned().into())
                        .collect(),
                );
                self.state.write().unwrap().list_state.select(None);
            }
            Err(e) => {
                self.set_loading_state(LoadingState::Error(e.to_string()));
                return;
            }
        }
        self.set_loading_state(LoadingState::Loaded);
    }

    async fn fetch_custom_view(self, view: custom_views_query::ViewFragment) {
        self.set_loading_state(LoadingState::Loading);
        self.set_selected_view(view.id.clone());
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = custom_view_query::Variables {
            custom_view_id: view.id.clone(),
        };
        match client.query(CustomViewQuery, variables).await {
            Ok(data) => {
                self.state.write().unwrap().issue_map.insert(
                    view.id,
                    data.custom_view
                        .issues
                        .nodes
                        .iter()
                        .map(|issue| issue.to_owned().into())
                        .collect(),
                );
                self.state.write().unwrap().list_state.select(None);
            }
            Err(e) => {
                self.set_loading_state(LoadingState::Error(e.to_string()));
                return;
            }
        }
        self.set_loading_state(LoadingState::Loaded);
    }

    async fn search_issues(self, search_term: String) {
        self.set_loading_state(LoadingState::Loading);
        self.set_selected_view(String::from("search_results"));
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = search_query::Variables {
            term: search_term.to_string(),
        };
        match client.query(SearchQuery, variables).await {
            Ok(data) => {
                self.state.write().unwrap().issue_map.insert(
                    "search_results".to_string(),
                    data.search_issues
                        .nodes
                        .iter()
                        .map(|issue| issue.to_owned().into())
                        .collect(),
                );
                self.state.write().unwrap().list_state.select(None);
            }
            Err(e) => {
                self.set_loading_state(LoadingState::Error(e.to_string()));
                return;
            }
        }
        self.set_loading_state(LoadingState::Loaded);
    }

    pub fn toggle_search_mode(&mut self) {
        if self.show_search_input {
            self.show_search_input = false;
            self.input_mode = InputMode::Normal;
        } else {
            self.show_search_input = true;
            self.input.reset();
            self.input_mode = InputMode::Editing;
        }
    }

    fn set_selected_view(&self, id: String) {
        self.state.write().unwrap().selected_view_id = id;
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }

    fn get_loading_state(&self) -> LoadingState {
        self.state.read().unwrap().loading_state.clone()
    }

    pub fn scroll_down(&self) {
        let mut state = self.state.write().unwrap();
        if let (Some(index), Some(map)) = (
            state.list_state.selected(),
            state.issue_map.get(&state.selected_view_id),
        ) {
            if index >= map.len() - 1 {
                return state.list_state.select_first();
            }
        }
        state.list_state.select_next()
    }

    pub fn scroll_up(&self) {
        let mut state = self.state.write().unwrap();

        match (
            state.list_state.selected(),
            state.issue_map.get(&state.selected_view_id),
        ) {
            (Some(0) | None, Some(map)) => {
                let max_index = map.len() - 1;
                state.list_state.select(Some(max_index));
            }
            _ => state.list_state.select_previous(),
        }
    }

    pub fn copy_branch_name(&self) {
        let state = self.state.read().unwrap();
        if let (Some(index), Some(map)) = (
            state.list_state.selected(),
            state.issue_map.get(&state.selected_view_id),
        ) {
            let branch_name = map[index].branch_name.clone();
            cli_clipboard::set_contents(branch_name).unwrap();
        }
    }

    pub fn open_url(&self) -> std::result::Result<(), std::io::Error> {
        let state = self.state.read().unwrap();
        if let (Some(index), Some(map)) = (
            state.list_state.selected(),
            state.issue_map.get(&state.selected_view_id),
        ) {
            let url = map[index].url.clone();
            open::that(&url)
        } else {
            Ok(())
        }
    }

    pub fn run(&self, tab_change_event: TabChangeEvent) {
        let this = self.clone();
        match tab_change_event {
            TabChangeEvent::FetchMyIssues => {
                tokio::spawn(this.fetch_my_issues());
            }
            TabChangeEvent::FetchCustomViewIssues(view) => {
                tokio::spawn(this.fetch_custom_view(view));
            }
            TabChangeEvent::SearchIssues => {
                self.set_selected_view(String::from("search_results"));
            }
            _ => (),
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> LtEvent {
        if self.get_loading_state() != LoadingState::Loaded {
            return LtEvent::None;
        }
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                use InputMode::{Editing, Normal};

                match (self.input_mode.clone(), key.code) {
                    (Normal, KeyCode::Char('j')) => {
                        self.scroll_down();
                        return LtEvent::SelectIssue;
                    }
                    (Normal, KeyCode::Char('k')) => {
                        self.scroll_up();
                        return LtEvent::SelectIssue;
                    }
                    (Normal, KeyCode::Char('o')) => {
                        let _ = self.open_url();
                        return LtEvent::None;
                    }
                    (Normal, KeyCode::Char('c') | KeyCode::Char('y')) => {
                        self.copy_branch_name();
                        return LtEvent::None;
                    }
                    (Editing, KeyCode::Enter) => {
                        self.input_mode = InputMode::Normal;
                        // tab?
                        tokio::spawn(self.clone().search_issues(String::from(self.input.value())));
                        return LtEvent::SearchIssues(self.input.value());
                    }
                    (Editing, _) => {
                        if self.show_search_input {
                            self.input.handle_event(event);
                        }
                        return LtEvent::None;
                    }
                    _ => {
                        return LtEvent::None;
                    }
                };
            }
        }
        LtEvent::None
    }
}
const SELECTED_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC);

impl Widget for &MyIssuesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};

        let (search_area, body_area): (Option<Rect>, Rect) = {
            if self.show_search_input {
                let vertical = Layout::vertical([Length(3), Min(0)]);
                let [search_area, body_area] = vertical.areas(area);
                (Some(search_area), body_area)
            } else {
                (None, area)
            }
        };

        let mut block = Block::bordered().title_bottom(Line::from(vec![
            Span::from(" <j/k> ").blue(),
            Span::from("to select "),
            Span::from("─"),
            Span::from(" <⁄> ").blue(),
            Span::from("to search"),
        ]));

        if let LoadingState::Loading = self.get_loading_state() {
            block = block.title(Line::from("Loading…").right_aligned());
        }

        if let LoadingState::Error(e) = self.get_loading_state() {
            let p = Paragraph::new(vec![
                Line::from("Error:\n\n".red().bold()),
                Line::from(e.red().bold().underlined()),
            ])
            .wrap(Wrap { trim: true })
            .block(block);
            p.render(area, buf);
            return;
        }
        let mut state = self.state.write().unwrap();
        let area_width = area.width;
        let rows: Vec<ListItem> = match state.issue_map.get(&state.selected_view_id) {
            Some(issues) => issues
                .iter()
                .map(|item| {
                    let mut text = Text::default();
                    let priority_icon = iconmap::p_to_nf(item.priority);
                    let status_icon = iconmap::state_to_nf(&item.state.type_);
                    let identifier = item.identifier.clone();
                    // gives the effect of right aligning icons and left aligning the text
                    let spaces = (area_width as usize) - identifier.len() - 8;
                    let line = format!(
                        "{}{}{}  {}",
                        identifier.fg(AMBER.c700),
                        " ".repeat(spaces),
                        status_icon,
                        priority_icon
                    );
                    text.extend([
                        item.title.clone(),
                        line.add_modifier(Modifier::BOLD).blue().to_string(),
                    ]);
                    ListItem::new(text)
                })
                .collect(),
            None => vec![],
        };

        // tests can't see the highlighting
        let highlight_symbol = if cfg!(test) { ">" } else { "" };

        let list = List::new(rows)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(highlight_symbol)
            .block(block);
        StatefulWidget::render(list, body_area, buf, &mut state.list_state);

        if let Some(search_area) = search_area {
            let block2 = Block::bordered().padding(Padding::ZERO);
            let value = if self.input_mode == InputMode::Editing {
                (self.input.value().to_owned() + "|").to_owned()
            } else {
                self.input.value().to_string()
            };
            let input = Paragraph::new(value).style(Color::Yellow).block(block2);
            input.render(search_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend, widgets::ListState};
    use tui_input::Input;

    use crate::{
        InputMode, LtEvent,
        widgets::{self, MyIssuesWidget, selected_issue::tests::make_issue},
    };

    fn create_key_event(key: char) -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char(key),
            kind: KeyEventKind::Press,
            modifiers: KeyModifiers::empty(),
            state: KeyEventState::empty(),
        })
    }

    #[test]
    fn test_empty_state() {
        let mut app = MyIssuesWidget::default();
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());

        let ev = app.handle_event(&create_key_event('j'));
        assert_eq!(ev, LtEvent::None);
    }

    #[test]
    fn test_with_issues() {
        let issues = vec![
            make_issue("Ticket One", "TEST-1"),
            make_issue("Ticket Two", "TEST-2"),
        ];
        let mut app = MyIssuesWidget {
            show_search_input: false,
            input: Input::default(),
            input_mode: InputMode::Normal,
            search_input_value: String::from(""),
            state: Arc::new(RwLock::new(widgets::issue_list::MyIssuesWidgetState {
                loading_state: crate::LoadingState::Loaded,
                selected_view_id: String::from("my_issues"),
                list_state: ListState::default(),
                issue_map: HashMap::from([(String::from("my_issues"), issues)]),
            })),
        };
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());

        assert_eq!(app.state.read().unwrap().list_state.selected(), None);

        let ev = app.handle_event(&create_key_event('j'));
        assert_eq!(ev, LtEvent::SelectIssue);
        assert_eq!(app.state.read().unwrap().list_state.selected(), Some(0));

        app.handle_event(&create_key_event('j'));
        assert_eq!(app.state.read().unwrap().list_state.selected(), Some(1));

        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
        // test that is passes through back to 0
        app.handle_event(&create_key_event('j'));
        assert_eq!(app.state.read().unwrap().list_state.selected(), Some(0));

        // test that is passes backwards to 1
        app.handle_event(&create_key_event('k'));
        assert_eq!(app.state.read().unwrap().list_state.selected(), Some(1));

        // test that <y> yanks the branch name to the clipboard
        // disabling because CI can't run this
        //app.handle_event(&create_key_event('y'));
        //assert_eq!(cli_clipboard::get_contents().unwrap(), "test-1-branch-name");

        assert!(!app.show_search_input);
        assert_eq!(app.input_mode, InputMode::Normal);
        app.toggle_search_mode();
        assert!(app.show_search_input);
        assert_eq!(app.input_mode, InputMode::Editing);
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
        app.toggle_search_mode();
        assert!(!app.show_search_input);
        assert_eq!(app.input_mode, InputMode::Normal);
    }
}
