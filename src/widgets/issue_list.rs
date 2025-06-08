use std::sync::{Arc, RwLock};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        palette::{material::{AMBER, BLUE, BLUE_GRAY, ORANGE, WHITE}, tailwind::SLATE}, Modifier, Style, Stylize
    },
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{
    api::LinearClient, iconmap, queries::{
        my_issues_query::{self, MyIssuesQueryIssues}, MyIssuesQuery
    }, LTWidget, LoadingState, LtEvent
};

#[derive(Debug, Default)]
pub struct MyIssuesWidgetState {
    loading_state: LoadingState,
    pub list_state: ListState,
    pub issues: MyIssuesQueryIssues,
}

#[derive(Debug, Clone, Default)]
pub struct MyIssuesWidget {
    pub state: Arc<RwLock<MyIssuesWidgetState>>,
}

impl MyIssuesWidget {
    async fn fetch(self) {
        self.set_loading_state(LoadingState::Loading);
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = my_issues_query::Variables {};
        match client.query(MyIssuesQuery, variables).await {
            Ok(data) => {
                self.state.write().unwrap().issues = data.issues;
                // TODO: Add cacheing implementation
                //println!("{:#?}", serde_json::to_string(&self.state.read().unwrap().issues).unwrap());
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
        if let Some(index) = state.list_state.selected() {
            if index >= state.issues.nodes.len() - 1 {
                return state.list_state.select_first();
            }
        }
        state.list_state.select_next()
    }

    pub fn scroll_up(&self) {
        let mut state = self.state.write().unwrap();

        match state.list_state.selected() {
            Some(0) | None => {
                let max_index = state.issues.nodes.len() - 1;
                state.list_state.select(Some(max_index));
            }
            _ => state.list_state.select_previous(),
        }
    }

    pub fn copy_branch_name(&self) {
        let state = self.state.read().unwrap();
        let selected_issue = &state.issues.nodes[state.list_state.selected().unwrap()];
        cli_clipboard::set_contents(selected_issue.branch_name.clone()).unwrap();
    }

    pub fn open_url(&self) -> std::result::Result<(), std::io::Error> {
        let state = self.state.read().unwrap();
        let selected_issue = &state.issues.nodes[state.list_state.selected().unwrap()];
        open::that(&selected_issue.url)
    }
}

impl LTWidget for MyIssuesWidget {
    fn run(&self) {
        let this = self.clone();
        tokio::spawn(this.fetch());
    }

    fn handle_event(&self, event: &Event) -> LtEvent {
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
                    KeyCode::Char('r') => {
                        self.run();
                        // TODO: Figure out how to get state to update better
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
const SELECTED_STYLE: Style = Style::new()
    .bg(SLATE.c100)
    .fg(BLUE_GRAY.c900);
    //.add_modifier(Modifier::BOLD);

impl Widget for &MyIssuesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        //println!("NOT HER");
        let mut block = Block::bordered()
            .title("My Issues")
            .title_bottom(Line::from(vec![
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
        let rows = state.issues.nodes.iter().map(|item| {
            let mut text = Text::default();
            let priority_icon = iconmap::p_to_nf(item.priority);
            let status_icon = iconmap::state_to_nf(&item.state.type_);
            let identifier = item.identifier.clone();
            // gives the effect of right aligning icons and left aligning the text
            let spaces = (area_width as usize) - identifier.len() - 7;
            let line = format!("{}{}{} {}", identifier.fg(AMBER.c700), " ".repeat(spaces), status_icon, priority_icon);
            text.extend([
                item.title.clone().white(),
                line.add_modifier(Modifier::BOLD).blue(),
            ]);
            ListItem::new(text)
        });

        let list = List::new(rows).highlight_style(SELECTED_STYLE).block(block);
        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
