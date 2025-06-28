use ratatui::buffer::Buffer;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};

use crate::api::LinearClient;
use crate::queries::{custom_views_query, CustomViewsQuery};
use crate::LTWidget;

#[derive(Debug, Clone, Default)]
pub struct TabWidget {
    selected_index: u16
}

impl TabWidget {
    async fn fetch(self) {
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = custom_views_query::Variables {};
        match client.query(CustomViewsQuery, variables).await {
            Ok(data) => {
                //self.state.write().unwrap().issues = data.issues.nodes.iter().map(|issue| issue.to_owned().into()).collect();
                println!("Yes {:#?}", data);
            }
            Err(e) => {
                panic!("Error {:#?}", e);
                //return;
            }
        }
        //self.set_loading_state(LoadingState::Loaded);
    }
    pub fn next(&mut self) {
        self.selected_index += 1;
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

impl LTWidget for TabWidget {
    fn run(&self) {
        let this = self.clone();
        tokio::spawn(this.fetch());
    }

    fn handle_event(&self, _event: &crossterm::event::Event) -> crate::LtEvent {
        crate::LtEvent::None
    }
}

impl Widget for &TabWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(["My Issues", "Other Issues"])
            .select(self.selected_index as usize)
            .padding(" ", " ")
            .divider("  ")
            .render(area, buf);
    }
}
