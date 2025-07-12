use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Tabs, Widget},
};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::api::LinearClient;
use crate::queries::{CustomViewsQuery, custom_views_query};
use crate::{TabChangeEvent, iconmap};

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
                    tab_type: TabType::MyIssues,
                    custom_view: None,
                }],
            })),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
enum TabType {
    #[default]
    MyIssues,
    CustomView,
    SearchResults,
}

#[derive(Debug, Clone, Default)]
struct Tab {
    title: String,
    tab_type: TabType,
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
                        tab_type: TabType::CustomView,
                        custom_view: Some(custom_view.clone()),
                    });
                }
            }
            Err(e) => {
                panic!("Error {:#?}", e);
                //return;
            }
        }
    }
    pub fn next(&self) -> usize {
        let mut state = self.state.write().unwrap();
        if state.selected_index < state.tabs.len() - 1 {
            state.selected_index += 1;
        }
        state.selected_index
    }

    pub fn prev(&self) -> usize {
        let mut state = self.state.write().unwrap();
        if state.selected_index > 0 {
            state.selected_index -= 1;
        }
        state.selected_index
    }

    pub fn show_and_select_search_tab(&self) {
        let mut state = self.state.write().unwrap();
        if state.tabs[state.tabs.len() - 1].tab_type != TabType::SearchResults {
            state.tabs.push(Tab {
                title: String::from("Search Results"),
                tab_type: TabType::SearchResults,
                custom_view: None,
            });
        }
        state.selected_index = state.tabs.len() - 1;
    }

    pub fn handle_event(&self, event: &Event) -> crate::TabChangeEvent {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Tab => {
                        let index = self.next();
                        let state = self.state.read().unwrap();
                        match &state.tabs[index].custom_view {
                            Some(custom_view) => {
                                return TabChangeEvent::FetchCustomViewIssues(custom_view.clone());
                            }
                            _ => {
                                return match state.tabs[index].tab_type {
                                    TabType::MyIssues => TabChangeEvent::FetchMyIssues,
                                    TabType::SearchResults => TabChangeEvent::SearchIssues,
                                    _ => TabChangeEvent::None,
                                };
                            }
                        }
                    }
                    KeyCode::BackTab => {
                        self.prev();
                        let state = self.state.read().unwrap();
                        match &state.tabs[state.selected_index].custom_view {
                            Some(custom_view) => {
                                return TabChangeEvent::FetchCustomViewIssues(custom_view.clone());
                            }
                            _ => {
                                return match state.tabs[state.selected_index].tab_type {
                                    TabType::MyIssues => TabChangeEvent::FetchMyIssues,
                                    TabType::SearchResults => TabChangeEvent::SearchIssues,
                                    _ => TabChangeEvent::None,
                                };
                            }
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
        use Constraint::{Length, Min};
        let horizontal = Layout::horizontal([Length(23), Min(0)]);
        let [header, main] = horizontal.areas(area);

        Line::from(vec![
            Span::from(" <tab> ").blue(),
            Span::from("to change view:  "),
        ])
        .render(header, buf);
        Tabs::new(
            self.state
                .read()
                .unwrap()
                .tabs
                .iter()
                .map(|tab| {
                    let (icon, color) = if let Some(view) = &tab.custom_view {
                        (
                            match &view.icon {
                                Some(icon) => iconmap::ico_to_nf(icon),
                                _ => "".to_string(),
                            },
                            match &view.color {
                                Some(color) => String::from(color),
                                _ => "#ffffff".to_string(),
                            },
                        )
                    } else if tab.tab_type == TabType::SearchResults {
                        (iconmap::ico_to_nf("Magnify"), String::from("#FFFFFF"))
                    } else {
                        (iconmap::ico_to_nf("Home"), String::from("#FFFFFF"))
                    };
                    let project_color = Color::from_str(&color).unwrap();
                    Span::from(format!("{} {}", icon, tab.title.clone().bold()))
                        .fg(project_color)
                        .bold()
                })
                .collect::<Vec<Span>>(),
        )
        .select(self.state.read().unwrap().selected_index)
        .padding(" ", " ")
        .divider("  ")
        .render(main, buf);
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
        widgets::{TabWidget, tab_widget::TabType},
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
        let mut terminal = Terminal::new(TestBackend::new(75, 2)).unwrap();
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
                        tab_type: TabType::MyIssues,
                        custom_view: None,
                    },
                    Tab {
                        title: String::from("Custom A"),
                        tab_type: TabType::CustomView,
                        custom_view: Some(custom_views_query::ViewFragment {
                            slug_id: Some("sluga".into()),
                            color: Some("#fa0faf".to_string()),
                            icon: Some("Education".to_string()),
                            id: "sluga".into(),
                            name: "Custom A".into(),
                        }),
                    },
                    Tab {
                        title: String::from("Custom B"),
                        tab_type: TabType::CustomView,
                        custom_view: Some(custom_views_query::ViewFragment {
                            slug_id: Some("slugb".into()),
                            id: "slugb".into(),
                            color: Some("#fa0faf".to_string()),
                            icon: Some("Education".to_string()),
                            name: "Custom B".into(),
                        }),
                    },
                ],
            })),
        };

        let mut terminal = Terminal::new(TestBackend::new(75, 2)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());

        // should go to  tab a
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(
            ev,
            TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
                name: "sluga".into(),
                slug_id: Some("sluga".into()),
                color: Some("#fa0faf".to_string()),
                icon: Some("Education".to_string()),
                id: String::from("sluga")
            })
        );

        // then to b
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(
            ev,
            TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
                name: "slugb".into(),
                slug_id: Some("slugb".into()),
                color: Some("#fa0faf".to_string()),
                icon: Some("Education".to_string()),
                id: String::from("slugb")
            })
        );

        // but not passed b
        let ev = app.handle_event(&create_key_event(KeyCode::Tab));
        assert_eq!(
            ev,
            TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
                name: "slugb".into(),
                slug_id: Some("slugb".into()),
                color: Some("#fa0faf".to_string()),
                icon: Some("Education".to_string()),
                id: String::from("slugb")
            })
        );

        // then back to a
        let ev = app.handle_event(&create_key_event(KeyCode::BackTab));
        assert_eq!(
            ev,
            TabChangeEvent::FetchCustomViewIssues(custom_views_query::ViewFragment {
                name: "sluga".into(),
                slug_id: Some("sluga".into()),
                color: Some("#fa0faf".to_string()),
                icon: Some("Education".to_string()),
                id: String::from("sluga")
            })
        );

        let ev = app.handle_event(&create_key_event(KeyCode::BackTab));
        assert_eq!(ev, TabChangeEvent::FetchMyIssues);
    }
}
