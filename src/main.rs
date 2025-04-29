mod api;
mod iconmap;
mod queries;
mod widgets;

use widgets::{MyIssuesWidget, SelectedIssueWidget};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use std::time::Duration;

use color_eyre::eyre::Result;

use api::*;
use queries::*;
use ratatui::{
    layout::{Constraint, Layout}, widgets::Clear, DefaultTerminal, Frame
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting up");
    color_eyre::install()?;
    println!("Installed eyre");
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal).await;
    println!("App started");
    ratatui::restore();
    app_result
}

/*
* Basic high level API for each widget
*/
pub trait LTWidget {
    fn run(&self);
    fn handle_event(&self, event: &Event) -> LtEvent;
}

pub enum LtEvent {
    None,
    SelectIssue,
}

//#[derive(Debug)]
struct App {
    should_quit: bool,
    api: LinearClient,
    my_issues_widget: MyIssuesWidget,
    selected_issue_widget: SelectedIssueWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 20.0;

    pub fn new() -> Self {
        Self {
            should_quit: false,
            api: LinearClient::default(),
            my_issues_widget: MyIssuesWidget::default(),
            selected_issue_widget: SelectedIssueWidget::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.my_issues_widget.run();
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => {
                    //terminal.draw(|frame| self.clear(frame))?;
                    //terminal.clear();
                    self.handle_event(&event)
                },
            }
        }
        Ok(())
    }

    fn clear(&mut self, frame: &mut Frame) {
        let vertical = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)]);
        let [list_area, body_area] = vertical.areas(frame.area());
        frame.render_widget(&self.my_issues_widget, list_area);
        frame.render_widget(Clear, body_area);
    }


    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)]);
        let [list_area, body_area] = vertical.areas(frame.area());
        frame.render_widget(&self.my_issues_widget, list_area);
        frame.render_widget(&self.selected_issue_widget, body_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    _ => {
                        self.selected_issue_widget.handle_event(event);
                        if let LtEvent::SelectIssue = self.my_issues_widget.handle_event(event) {
                            let my_issues_widget_state =
                                self.my_issues_widget.state.write().unwrap();
                            let selected_issue: Option<my_issues_query::MyIssuesQueryIssuesNodes> =
                                my_issues_widget_state
                                    .list_state
                                    .selected()
                                    .map(|index| my_issues_widget_state.issues.nodes[index].clone());
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
