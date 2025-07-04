use std::sync::{Arc, RwLock};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        Modifier, Style, Stylize,
        palette::{
            material::{AMBER, BLUE_GRAY},
            tailwind::SLATE,
        },
    },
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use std::collections::HashMap;

use crate::{
    IssueFragment, LoadingState, LtEvent, TabChangeEvent,
    api::LinearClient,
    iconmap,
    queries::{
        CustomViewQuery, MyIssuesQuery, custom_view_query, custom_views_query,
        my_issues_query::{self},
    },
};

#[derive(Debug, Default)]
pub struct MyIssuesWidgetState {
    loading_state: LoadingState,
    pub list_state: ListState,
    pub issue_map: HashMap<String, Vec<IssueFragment>>,
}

#[derive(Debug, Clone, Default)]
pub struct MyIssuesWidget {
    pub state: Arc<RwLock<MyIssuesWidgetState>>,
    pub selected_view_id: String,
}

impl MyIssuesWidget {
    async fn fetch_my_issues(self) {
        self.set_loading_state(LoadingState::Loading);
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
            state.issue_map.get(&self.selected_view_id),
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
            state.issue_map.get(&self.selected_view_id),
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
            state.issue_map.get(&self.selected_view_id),
        ) {
            let branch_name = map[index].branch_name.clone();
            cli_clipboard::set_contents(branch_name).unwrap();
        }
    }

    pub fn open_url(&self) -> std::result::Result<(), std::io::Error> {
        let state = self.state.read().unwrap();
        if let (Some(index), Some(map)) = (
            state.list_state.selected(),
            state.issue_map.get(&self.selected_view_id),
        ) {
            let url = map[index].url.clone();
            open::that(&url)
        } else {
            Ok(())
        }
    }

    pub fn run(&mut self, tab_change_event: TabChangeEvent) {
        let this = self.clone();
        match tab_change_event {
            TabChangeEvent::FetchMyIssues => {
                self.selected_view_id = String::from("my_issues");
                tokio::spawn(this.fetch_my_issues());
            }
            TabChangeEvent::FetchCustomViewIssues(view) => {
                self.selected_view_id = view.id.clone();
                tokio::spawn(this.fetch_custom_view(view));
            }
            _ => (),
        }
    }

    pub fn handle_event(&self, event: &Event) -> LtEvent {
        if self.get_loading_state() != LoadingState::Loaded {
            return LtEvent::None;
        }
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('j') => {
                        self.scroll_down();
                        return LtEvent::SelectIssue;
                    }
                    KeyCode::Char('k') => {
                        self.scroll_up();
                        return LtEvent::SelectIssue;
                    }
                    KeyCode::Char('o') => {
                        let _ = self.open_url();
                        return LtEvent::None;
                    }
                    KeyCode::Char('c') | KeyCode::Char('y') => {
                        self.copy_branch_name();
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
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c100).fg(BLUE_GRAY.c900);

impl Widget for &MyIssuesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::bordered().title_bottom(Line::from(vec![
            Span::from(" <j/k> ").blue(),
            Span::from("to select issue "),
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
        let rows: Vec<ListItem> = match state.issue_map.get(&self.selected_view_id) {
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
                        item.title.clone().white(),
                        line.add_modifier(Modifier::BOLD).blue(),
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
        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::{Arc, RwLock}};

    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend, widgets::ListState};

    use crate::{
        widgets::{self, selected_issue::tests::make_issue, MyIssuesWidget}, LtEvent
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
        let app = MyIssuesWidget::default();
        let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
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
        let app = MyIssuesWidget {
            selected_view_id: String::from("my_issues"),
            state: Arc::new(RwLock::new(widgets::issue_list::MyIssuesWidgetState {
                loading_state: crate::LoadingState::Loaded,
                list_state: ListState::default(),
                issue_map: HashMap::from([(String::from("my_issues"), issues)]),
            })),
        };
        let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
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
        app.handle_event(&create_key_event('y'));
        assert_eq!(cli_clipboard::get_contents().unwrap(), "test-1-branch-name");
    }
}
