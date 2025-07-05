mod api;
mod iconmap;
mod queries;
mod widgets;
use serde::{Deserialize, Serialize};
use widgets::{MyIssuesWidget, SelectedIssueWidget, TabWidget};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use std::{
    fmt::{self},
    time::Duration,
};

use color_eyre::eyre::Result;

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

/* Events for widget communication */
#[derive(Debug, PartialEq)]
pub enum LtEvent {
    None,
    SelectIssue,
}

#[derive(Debug, PartialEq)]
pub enum TabChangeEvent {
    None,
    FetchCustomViewIssues(custom_views_query::ViewFragment),
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
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    KeyCode::Tab | KeyCode::BackTab => {
                        self.issue_list_widget
                            .run(self.tab_widget.handle_event(event));
                    }
                    _ => {
                        self.selected_issue_widget.handle_event(event);
                        if let LtEvent::SelectIssue = self.issue_list_widget.handle_event(event) {
                            let issue_list_widget_state =
                                self.issue_list_widget.state.write().unwrap();
                            let selected_issue: Option<IssueFragment> = issue_list_widget_state
                                .list_state
                                .selected()
                                .map(|index| {
                                    issue_list_widget_state.issue_map[&self.issue_list_widget.selected_view_id][index].clone()
                                });
                            self.selected_issue_widget
                                .set_selected_issue(selected_issue);
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

impl From<my_issues_query::IssueFragment> for IssueFragment {
    fn from(item: my_issues_query::IssueFragment) -> Self {
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

impl From<custom_view_query::IssueFragment> for IssueFragment {
    fn from(item: custom_view_query::IssueFragment) -> Self {
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

impl From<my_issues_query::IssueFragmentState> for IssueFragmentState {
    fn from(item: my_issues_query::IssueFragmentState) -> Self {
        Self {
            name: item.name,
            color: item.color,
            type_: item.type_,
        }
    }
}

impl From<custom_view_query::IssueFragmentState> for IssueFragmentState {
    fn from(item: custom_view_query::IssueFragmentState) -> Self {
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

impl From<my_issues_query::IssueFragmentAssignee> for IssueFragmentAssignee {
    fn from(item: my_issues_query::IssueFragmentAssignee) -> Self {
        Self {
            is_me: item.is_me,
            display_name: item.display_name,
        }
    }
}

impl From<custom_view_query::IssueFragmentAssignee> for IssueFragmentAssignee {
    fn from(item: custom_view_query::IssueFragmentAssignee) -> Self {
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

impl From<my_issues_query::IssueFragmentCreator> for IssueFragmentCreator {
    fn from(item: my_issues_query::IssueFragmentCreator) -> Self {
        Self {
            is_me: item.is_me,
            display_name: item.display_name,
        }
    }
}

impl From<custom_view_query::IssueFragmentCreator> for IssueFragmentCreator {
    fn from(item: custom_view_query::IssueFragmentCreator) -> Self {
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

impl From<my_issues_query::IssueFragmentProject> for IssueFragmentProject {
    fn from(item: my_issues_query::IssueFragmentProject) -> Self {
        Self {
            name: item.name,
            icon: item.icon,
            color: item.color,
        }
    }
}

impl From<custom_view_query::IssueFragmentProject> for IssueFragmentProject {
    fn from(item: custom_view_query::IssueFragmentProject) -> Self {
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

impl From<my_issues_query::IssueFragmentLabels> for IssueFragmentLabels {
    fn from(item: my_issues_query::IssueFragmentLabels) -> Self {
        Self {
            edges: item
                .edges
                .iter()
                .map(|edge| edge.to_owned().into())
                .collect(),
        }
    }
}

impl From<custom_view_query::IssueFragmentLabels> for IssueFragmentLabels {
    fn from(item: custom_view_query::IssueFragmentLabels) -> Self {
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

impl From<my_issues_query::IssueFragmentLabelsEdges> for IssueFragmentLabelsEdges {
    fn from(item: my_issues_query::IssueFragmentLabelsEdges) -> Self {
        Self {
            node: item.node.into(),
        }
    }
}

impl From<custom_view_query::IssueFragmentLabelsEdges> for IssueFragmentLabelsEdges {
    fn from(item: custom_view_query::IssueFragmentLabelsEdges) -> Self {
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

impl From<my_issues_query::IssueFragmentLabelsEdgesNode> for IssueFragmentLabelsEdgesNode {
    fn from(item: my_issues_query::IssueFragmentLabelsEdgesNode) -> Self {
        Self {
            color: item.color,
            name: item.name,
        }
    }
}

impl From<custom_view_query::IssueFragmentLabelsEdgesNode> for IssueFragmentLabelsEdgesNode {
    fn from(item: custom_view_query::IssueFragmentLabelsEdgesNode) -> Self {
        Self {
            color: item.color,
            name: item.name,
        }
    }
}
