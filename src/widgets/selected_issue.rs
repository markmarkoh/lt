use crate::IssueFragment;
use crate::LtEvent;
use crate::iconmap;

use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::text::Text;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Wrap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use chrono::DateTime;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Default)]
struct SelectedIssueWidgetState {
    selected_issue: Option<IssueFragment>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectedIssueWidget {
    state: Arc<RwLock<SelectedIssueWidgetState>>,
    scroll_state: ScrollbarState,
    scroll: usize,
}

impl SelectedIssueWidget {
    pub fn set_selected_issue(&mut self, issue: Option<IssueFragment>) {
        self.state.write().unwrap().selected_issue = issue;
        self.scroll = 0;
        self.scroll_state = ScrollbarState::default();
    }

    pub fn handle_event(&mut self, event: &Event) -> LtEvent {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Down => {
                        self.scroll = self.scroll.saturating_add(1);
                        self.scroll_state.next();
                    }
                    KeyCode::Up => {
                        self.scroll = self.scroll.saturating_sub(1);
                        self.scroll_state.prev();
                    }
                    _ => {}
                }
            }
        }
        LtEvent::None
    }
}

const DICT_HEADER: Style = Style::new();

fn header(text: &str) -> Line {
    Line::from(Span::from(text.to_owned() + ":\n")).style(DICT_HEADER)
}

impl Widget for &SelectedIssueWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.read().unwrap();

        let [main, scroll, sidebar] = Layout::horizontal([
            Constraint::Percentage(75),
            Constraint::Min(1),
            Constraint::Percentage(25),
        ])
        .areas(area);

        let (identifier, title_text, description, created_at, sidebar_lines) =
            match &state.selected_issue {
                Some(issue) => {
                    let identifier = issue.identifier.clone().blue().bold();
                    let title_text = Line::from(issue.title.clone()).centered();
                    let description = if issue.description.is_some() {
                        tui_markdown::from_str(issue.description.as_ref().unwrap())
                    } else {
                        tui_markdown::from_str("#### No description")
                    };
                    let created_at = DateTime::parse_from_rfc3339(&issue.created_at.clone())
                        .unwrap()
                        .format("%m/%d/%Y")
                        .to_string();

                    let mut sidebar_items = vec![];

                    if !issue.priority_label.is_empty() {
                        sidebar_items.push(header("Priority"));
                        let priority_icon = iconmap::p_to_nf(issue.priority);
                        sidebar_items.push(Line::from(vec![
                            priority_icon.add_modifier(Modifier::BOLD),
                            issue.priority_label.clone().into(),
                        ]));
                        sidebar_items.push(Line::from(""));
                    }
                    if !issue.state.name.is_empty() {
                        sidebar_items.push(header("Status"));
                        let state_icon = iconmap::state_to_nf(&issue.state.type_)
                            .fg(Color::from_str(&issue.state.color).unwrap());
                        sidebar_items.push(Line::from(vec![
                            state_icon.fg(Color::from_str(&issue.state.color).unwrap()),
                            " ".into(),
                            issue
                                .state
                                .name
                                .clone()
                                .fg(Color::from_str(&issue.state.color).unwrap()),
                        ]));
                        sidebar_items.push(Line::from(""));
                    }

                    if let Some(project) = &issue.project {
                        let project_color = Color::from_str(&project.color).unwrap();
                        let project_icon =
                            iconmap::ico_to_nf(project.icon.as_ref().map_or("NN", |v| v))
                                .fg(project_color);
                        sidebar_items.push(header("Project"));
                        sidebar_items.push(Line::from(vec![
                            project_icon,
                            project.name.clone().fg(project_color),
                        ]));
                        sidebar_items.push(Line::from(""));
                    }

                    if let Some(assignee) = &issue.assignee {
                        sidebar_items.push(header("Assignee"));
                        if assignee.is_me {
                            sidebar_items.push(Line::from("You"));
                        } else {
                            sidebar_items.push(Line::from(assignee.display_name.clone()));
                        }
                        sidebar_items.push(Line::from(""));
                    }

                    if let Some(creator) = &issue.creator {
                        sidebar_items.push(header("Creator"));
                        if creator.is_me {
                            sidebar_items.push(Line::from("You"));
                        } else {
                            sidebar_items.push(Line::from(creator.display_name.clone()));
                        }
                        sidebar_items.push(Line::from(""));
                    }

                    if !issue.labels.edges.is_empty() {
                        sidebar_items.push(header("Tags"));
                        sidebar_items.push(Line::from(
                            issue
                                .labels
                                .edges
                                .iter()
                                .map(|value| {
                                    Span::from(format!("• {} ", value.node.name.clone()))
                                        .fg(Color::from_str(&value.node.color).unwrap())
                                })
                                .collect::<Vec<Span>>(),
                        ));
                        sidebar_items.push(Line::from(""));
                    }
                    (
                        identifier,
                        title_text,
                        description,
                        created_at,
                        sidebar_items,
                    )
                }
                None => (
                    String::from("").into(),
                    Line::from(String::from("Select an issue to see some details")),
                    Text::from(""),
                    String::from(""),
                    vec![],
                ),
            };

        let created_at_title = Line::from(created_at).right_aligned();

        // collapse borders for nicer UI
        let collapsed_top_and_left_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..symbols::border::PLAIN
        };

        let (effective_scroll, scroll_enabled) = if description.height() > buf.area().height.into()
        {
            let effective_scroll = if self.scroll > description.height() / 2 {
                description.height() / 2
            } else {
                self.scroll
            };

            let mut scroll_state = self
                .scroll_state
                .content_length(description.height() / 2)
                .position(effective_scroll);

            let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            sb.render(scroll, buf, &mut scroll_state);
            (effective_scroll, true)
        } else {
            let placeholder_block = Block::new().borders(Borders::TOP | Borders::BOTTOM);
            placeholder_block.render(scroll, buf);
            (0, false)
        };

        let mut block = Block::new()
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_style(DICT_HEADER)
            .title_bottom(Line::from(vec![
                Span::from("──"),
                Span::from(" <y> ").blue(),
                Span::from("to yank git branch "),
            ]))
            .title_bottom(Line::from(vec![
                Span::from(" <o> ").blue(),
                Span::from("to open in Linear"),
            ]))
            .title(identifier)
            .title(title_text);

        block = if scroll_enabled {
            block.title_bottom(
                Line::from(vec![
                    Span::from(" <↑/↓> ").blue(),
                    Span::from("to scroll ──────"),
                ])
                .right_aligned(),
            )
        } else {
            block
        };

        let mut p = Paragraph::new(description.clone())
            .block(block)
            .wrap(Wrap { trim: true });

        p = p.scroll((effective_scroll as u16, 0));

        let sidebar_block = Block::bordered()
            .border_set(collapsed_top_and_left_border_set)
            .title_bottom(Line::from(vec![
                Span::from("──"),
                Span::from(" <q> ").blue(),
                Span::from("to quit "),
            ]))
            .title(created_at_title);

        let sidebar_p = Paragraph::new(sidebar_lines)
            .block(sidebar_block)
            .wrap(Wrap { trim: true });

        //Clear.render(main, buf);
        p.render(main, buf);
        sidebar_p.render(sidebar, buf);
    }
}

// Now in tests module:
#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        IssueFragment, IssueFragmentAssignee, IssueFragmentCreator, IssueFragmentProject,
        IssueFragmentState,
    };
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::widgets::SelectedIssueWidget;

    pub fn make_issue(title: &str, identifier: &str) -> IssueFragment {
        IssueFragment {
            priority: 1.0,
            priority_label: "Urgent".into(),
            branch_name: "test-1-branch-name".into(),
            identifier: String::from(identifier),
            title: String::from(title),
            created_at: String::from("2025-05-10T03:09:51.740Z"),
            project: Some(IssueFragmentProject {
                name: "Test Project".into(),
                icon: Some("Subgroup".into()),
                color: "#FA0FA0".into(),
            }),
            creator: Some(IssueFragmentCreator {
                is_me: true,
                display_name: "Creator Display Name".into(),
            }),
            assignee: Some(IssueFragmentAssignee {
                is_me: false,
                display_name: "Assignee Display Name".into(),
            }),
            state: IssueFragmentState {
                name: "Backlogged".into(),
                color: "#0FA0FA".into(),
                type_: "backlog".into(),
            },
            description: Some(String::from("### Title\n\nMulti\nLine **description**")),
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_state() {
        let app = SelectedIssueWidget::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_basic_issue() {
        let mut app = SelectedIssueWidget::default();
        let issue = make_issue("Testing Ticket", "TEST-1");

        app.set_selected_issue(Some(issue));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
