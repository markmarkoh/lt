mod api;
mod iconmap;
mod queries;
mod widgets;
use crossterm::event::EventStream;
use duplicate::duplicate_item;
use serde::{Deserialize, Serialize};
use widgets::{MyIssuesWidget, SelectedIssueWidget, TabWidget};

use std::{
    fmt::{self},
    time::Duration,
};

use color_eyre::eyre::Result;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use queries::*;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("LINEAR_API_TOKEN").is_err() {
        println!("Hey! Set a LINEAR_API_TOKEN environment variable to get this show started.");
        std::process::exit(1);
    }
    color_eyre::install()?;

    let terminal = ratatui::init();
    let app_result = App::new().run(terminal).await;
    ratatui::restore();
    app_result
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

/* Events for widget communication */
#[derive(Debug, PartialEq)]
pub enum LtEvent<'a> {
    None,
    SelectIssue,
    SearchIssues(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum TabChangeEvent {
    None,
    FetchCustomViewIssues(custom_views_query::ViewFragment),
    SearchIssues,
    FetchMyIssues,
}

impl PartialEq for custom_views_query::ViewFragment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for TabChangeEvent {
    fn default() -> Self {
        Self::FetchMyIssues
    }
}

//#[derive(Debug)]
struct App {
    should_quit: bool,
    issue_list_widget: MyIssuesWidget,
    selected_issue_widget: SelectedIssueWidget,
    tab_widget: TabWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 30.0;

    pub fn new() -> Self {
        Self {
            should_quit: false,
            issue_list_widget: MyIssuesWidget::default(),
            selected_issue_widget: SelectedIssueWidget::default(),
            tab_widget: TabWidget::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.tab_widget.run();
        self.issue_list_widget.run(TabChangeEvent::default());
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => {
                    self.handle_event(&event)
                },
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Length(1), Min(0)]);
        let [tab_area, body_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Percentage(25), Percentage(75)]);
        let [list_area, body_area] = horizontal.areas(body_area);
        frame.render_widget(&self.issue_list_widget, list_area);
        frame.render_widget(&self.selected_issue_widget, body_area);
        frame.render_widget(&self.tab_widget, tab_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match (key.code, self.issue_list_widget.input_mode.clone()) {
                    (KeyCode::Char('q') | KeyCode::Esc, InputMode::Normal) => {
                        self.should_quit = true;
                    }

                    (KeyCode::Tab | KeyCode::BackTab, _) => {
                        self.issue_list_widget
                            .run(self.tab_widget.handle_event(event));
                    }
                    (KeyCode::Char('/'), InputMode::Normal) => {
                        self.issue_list_widget.toggle_search_mode();
                    }
                    (KeyCode::Esc, InputMode::Editing) => {
                        self.issue_list_widget.toggle_search_mode();
                    }
                    _ => {
                        self.selected_issue_widget.handle_event(event);
                        match self.issue_list_widget.handle_event(event) {
                            LtEvent::SelectIssue => {
                                let issue_list_widget_state =
                                    self.issue_list_widget.state.write().unwrap();
                                let selected_issue: Option<IssueFragment> =
                                    issue_list_widget_state.list_state.selected().map(|index| {
                                        issue_list_widget_state.issue_map
                                            [&issue_list_widget_state.selected_view_id][index]
                                            .clone()
                                    });
                                self.selected_issue_widget
                                    .set_selected_issue(selected_issue);
                            }
                            LtEvent::SearchIssues(term) => {
                                self.tab_widget.show_and_select_search_tab();
                            }
                            _ => (),
                        }
                    }
                };
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error(String),
}

impl fmt::Display for LoadingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct IssueFragment {
    pub title: String,
    pub identifier: String,
    pub state: IssueFragmentState,
    pub url: String,
    pub assignee: Option<IssueFragmentAssignee>,
    pub creator: Option<IssueFragmentCreator>,
    pub estimate: Option<f64>,
    pub project: Option<IssueFragmentProject>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "priorityLabel")]
    pub priority_label: String,
    pub priority: f64,
    pub labels: IssueFragmentLabels,
    #[serde(rename = "branchName")]
    pub branch_name: String,
    pub description: Option<String>,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragment] [ IssueFragment ];
    [ my_issues_query::IssueFragment] [ IssueFragment ];
    [ search_query::IssueFragment] [ IssueFragment ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            title: item.title,
            identifier: item.identifier,
            url: item.url,
            estimate: item.estimate,
            state: item.state.into(),
            created_at: item.created_at,
            priority: item.priority,
            priority_label: item.priority_label,
            branch_name: item.branch_name,
            description: item.description,
            labels: item.labels.into(),
            assignee: item.assignee.map(|assignee| assignee.into()),
            creator: item.creator.map(|creator| creator.into()),
            project: item.project.map(|project| project.into()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct IssueFragmentState {
    pub name: String,
    pub color: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentState ] [ IssueFragmentState ];
    [ my_issues_query::IssueFragmentState ] [ IssueFragmentState ];
    [ search_query::IssueFragmentState ] [ IssueFragmentState ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            name: item.name,
            color: item.color,
            type_: item.type_,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct IssueFragmentAssignee {
    #[serde(rename = "isMe")]
    pub is_me: bool,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentAssignee ] [ IssueFragmentAssignee ];
    [ my_issues_query::IssueFragmentAssignee ] [ IssueFragmentAssignee ];
    [ search_query::IssueFragmentAssignee ] [ IssueFragmentAssignee ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            is_me: item.is_me,
            display_name: item.display_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IssueFragmentCreator {
    #[serde(rename = "isMe")]
    pub is_me: bool,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentCreator ] [ IssueFragmentCreator ];
    [ my_issues_query::IssueFragmentCreator ] [ IssueFragmentCreator ];
    [ search_query::IssueFragmentCreator ] [ IssueFragmentCreator ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            is_me: item.is_me,
            display_name: item.display_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IssueFragmentProject {
    pub name: String,
    pub icon: Option<String>,
    pub color: String,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentProject ] [ IssueFragmentProject ];
    [ my_issues_query::IssueFragmentProject ] [ IssueFragmentProject ];
    [ search_query::IssueFragmentProject ] [ IssueFragmentProject ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            name: item.name,
            icon: item.icon,
            color: item.color,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct IssueFragmentLabels {
    pub edges: Vec<IssueFragmentLabelsEdges>,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentLabels ] [ IssueFragmentLabels ];
    [ my_issues_query::IssueFragmentLabels ] [ IssueFragmentLabels ];
    [ search_query::IssueFragmentLabels ] [ IssueFragmentLabels ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            edges: item
                .edges
                .iter()
                .map(|edge| edge.to_owned().into())
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IssueFragmentLabelsEdges {
    pub node: IssueFragmentLabelsEdgesNode,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentLabelsEdges ] [ IssueFragmentLabelsEdges ];
    [ my_issues_query::IssueFragmentLabelsEdges ] [ IssueFragmentLabelsEdges ];
    [ search_query::IssueFragmentLabelsEdges ] [ IssueFragmentLabelsEdges ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            node: item.node.into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IssueFragmentLabelsEdgesNode {
    pub color: String,
    pub name: String,
}

#[duplicate_item(
    from_type   to_type;
    [ custom_view_query::IssueFragmentLabelsEdgesNode ] [ IssueFragmentLabelsEdgesNode ];
    [ my_issues_query::IssueFragmentLabelsEdgesNode ] [ IssueFragmentLabelsEdgesNode ];
    [ search_query::IssueFragmentLabelsEdgesNode ] [ IssueFragmentLabelsEdgesNode ];
)]
impl From<from_type> for to_type {
    fn from(item: from_type) -> Self {
        Self {
            color: item.color,
            name: item.name,
        }
    }
}
