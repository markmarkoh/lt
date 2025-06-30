use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::{
    layout::Rect,
    widgets::{Tabs, Widget},
};
use std::sync::{Arc, RwLock};

use crate::TabChangeEvent;
use crate::api::LinearClient;
use crate::queries::{CustomViewsQuery, custom_views_query};

#[derive(Debug, Clone)]
pub struct TabWidget {
    state: Arc<RwLock<TabWidgetState>>,
}

#[derive(Debug, Clone)]
struct TabWidgetState {
    selected_index: usize,
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
    pub fn run(&self) {
        let this = self.clone();
        tokio::spawn(this.fetch());
    }

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
            }
            Err(e) => {
                panic!("Error {:#?}", e);
                //return;
            }
        }
        //self.set_loading_state(LoadingState::Loaded);
    }
    pub fn next(&self) {
        let mut state = self.state.write().unwrap();
        if state.selected_index < state.tabs.len() - 1 {
            state.selected_index += 1;
        }
    }

    pub fn prev(&self) {
        let mut state = self.state.write().unwrap();
        if state.selected_index > 0 {
            state.selected_index -= 1;
        }
    }

    pub fn handle_event(&self, event: &Event) -> crate::TabChangeEvent {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Tab => {
                        self.next();
                        let state = self.state.read().unwrap();
                        match &state.tabs[state.selected_index].custom_view {
                            Some(custom_view) => {
                                return TabChangeEvent::FetchCustomViewIssues(custom_view.clone());
                            }
                            _ => return TabChangeEvent::FetchMyIssues,
                        }
                    }
                    KeyCode::BackTab => {
                        self.prev();
                        let state = self.state.read().unwrap();
                        match &state.tabs[state.selected_index].custom_view {
                            Some(custom_view) => {
                                return TabChangeEvent::FetchCustomViewIssues(custom_view.clone());
                            }
                            _ => return TabChangeEvent::FetchMyIssues,
                        }
                    }
                    _ => return TabChangeEvent::None,
                }
            }
        }
        TabChangeEvent::None
    }
}

impl Widget for &TabWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(
            self.state
                .read()
                .unwrap()
                .tabs
                .iter()
                .map(|tab| tab.title.clone())
                .collect::<Vec<String>>(),
        )
        .select(self.state.read().unwrap().selected_index)
        .padding(" ", " ")
        .divider("  ")
        .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::{
        TabChangeEvent,
        queries::custom_views_query,
        widgets::TabWidget,
    };

    use super::{Tab, TabWidgetState};

    fn create_key_event(code: KeyCode) -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers: KeyModifiers::empty(),
            state: KeyEventState::empty(),
        })
    }

    #[test]
    fn test_empty_state() {
        let app = TabWidget::default();
        let mut terminal = Terminal::new(TestBackend::new(50, 2)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());

        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(ev, TabChangeEvent::FetchMyIssues);

        let ev = app.handle_event(&create_key_event(KeyCode::Char('j')));
        assert_eq!(ev, TabChangeEvent::None);
    }

    #[test]
    fn test_multi_tabs() {
        let app = TabWidget {
            state: Arc::new(RwLock::new(TabWidgetState {
                selected_index: 0,
                tabs: vec![
                    Tab {
                        title: String::from("My Issues"),
                        custom_view: None,
                    },
                    Tab {
                        title: String::from("Custom A"),
                        custom_view: Some(custom_views_query::ViewFragment {
                            slug_id: Some("sluga".into()),
                            id: "sluga".into(),
                            name: "Custom A".into(),
                        }),
                    },
                    Tab {
                        title: String::from("Custom B"),
                        custom_view: Some(custom_views_query::ViewFragment {
                            slug_id: Some("slugb".into()),
                            id: "slugb".into(),
                            name: "Custom B".into(),
                        }),
                    },
                ],
            })),
        };

        let mut terminal = Terminal::new(TestBackend::new(50, 2)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());

        // should go to  tab a
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(ev, TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
            name: "sluga".into(),
            slug_id: Some("sluga".into()),
            id: String::from("sluga")
        }));

        // then to b
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(ev, TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
            name: "slugb".into(),
            slug_id: Some("slugb".into()),
            id: String::from("slugb")
        }));

        // but not passed b
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(ev, TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
            name: "slugb".into(),
            slug_id: Some("slugb".into()),
            id: String::from("slugb")
        }));

        // then back to a 
        let ev = app.handle_event(&create_key_event(KeyCode::BackTab));
        assert_eq!(ev, TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
            name: "sluga".into(),
            slug_id: Some("sluga".into()),
            id: String::from("sluga")
        }));

        let ev = app.handle_event(&create_key_event(KeyCode::BackTab));
        assert_eq!(ev, TabChangeEvent::FetchMyIssues);


    }
}
