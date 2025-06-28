use ratatui::buffer::Buffer;
use ratatui::{
    layout::Rect,
    widgets::{Tabs, Widget},
};
use std::sync::{Arc, RwLock};

use crate::LTWidget;
use crate::api::LinearClient;
use crate::queries::{CustomViewsQuery, custom_views_query};

#[derive(Debug, Clone)]
pub struct TabWidget {
    state: Arc<RwLock<TabWidgetState>>,
}

#[derive(Debug, Clone)]
struct TabWidgetState {
    selected_index: u16,
    tabs: Vec<Tab>,
}

impl Default for TabWidget {
    fn default() -> Self {
        TabWidget {
            state: Arc::new(RwLock::new(TabWidgetState {
                selected_index: 0,
                tabs: vec![Tab {
                    title: String::from("My Issues"),
                    custom_view: None,
                }],
            })),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Tab {
    title: String,
    custom_view: Option<custom_views_query::ViewFragment>,
}

impl TabWidget {
    async fn fetch(self) {
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = custom_views_query::Variables {};
        match client.query(CustomViewsQuery, variables).await {
            Ok(data) => {
                let mut state = self.state.write().unwrap();
                for custom_view in data.custom_views.nodes.iter() {
                    state.tabs.push(Tab {
                        title: custom_view.name.clone(),
                        custom_view: Some(custom_view.clone()),
                    });
                }
                //self.state.write().unwrap().issues = data.issues.nodes.iter().map(|issue| issue.to_owned().into()).collect();
                // println!("Yes {:#?}", data);
            }
            Err(e) => {
                panic!("Error {:#?}", e);
                //return;
            }
        }
        //self.set_loading_state(LoadingState::Loaded);
    }
    pub fn next(&mut self) {
        let mut state = self.state.write().unwrap();
        if usize::from(state.selected_index) < state.tabs.len() - 1 {
            state.selected_index += 1;
        }
    }

    pub fn prev(&mut self) {
        let mut state = self.state.write().unwrap();
        if state.selected_index > 0 {
            state.selected_index -= 1;
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
        Tabs::new(
            self.state.read().unwrap().tabs
                .iter()
                .map(|tab| tab.title.clone())
                .collect::<Vec<String>>(),
        )
        .select(self.state.read().unwrap().selected_index as usize)
        .padding(" ", " ")
        .divider("  ")
        .render(area, buf);
    }
}
