use crate::presentation::components::core::base::{
    Component, ComponentState, ComponentProps, CommonComponentState,
};
use crate::presentation::components::core::{
    ComponentResult, ComponentId, ValidationState,
};
use crate::presentation::theme::AppTheme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Lowest,
    Low,
    Medium,
    High,
    Highest,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Todo
    }
}

/// Task source system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    Jira,
    Monday,
    GitHub,
    Local,
    Custom,
}

impl Default for TaskSource {
    fn default() -> Self {
        TaskSource::Local
    }
}

/// Task item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub source: TaskSource,
    pub assignee: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl TaskItem {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            source: TaskSource::Local,
            assignee: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Task list component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListProps {
    pub tasks: Vec<TaskItem>,
    pub title: Option<String>,
    pub show_borders: bool,
    pub show_status: bool,
    pub show_priority: bool,
    pub show_assignee: bool,
    pub multi_select: bool,
    pub empty_message: String,
}

impl Default for TaskListProps {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            title: None,
            show_borders: true,
            show_status: true,
            show_priority: true,
            show_assignee: true,
            multi_select: false,
            empty_message: "No tasks".to_string(),
        }
    }
}

impl ComponentProps for TaskListProps {
    fn default_props() -> Self {
        Self::default()
    }
}

/// Task list component state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListComponentState {
    pub common: CommonComponentState,
    pub selected_tasks: Vec<String>,
    pub current_task: usize,
    pub scroll_offset: usize,
    pub filter_status: Option<TaskStatus>,
    pub filter_priority: Option<TaskPriority>,
    pub search_query: String,
}

impl ComponentState for TaskListComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for TaskListComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_tasks: Vec::new(),
            current_task: 0,
            scroll_offset: 0,
            filter_status: None,
            filter_priority: None,
            search_query: String::new(),
        }
    }
}

/// Task list component
pub struct TaskList {
    id: ComponentId,
    props: TaskListProps,
    state: TaskListComponentState,
    list_state: ListState,
}

impl TaskList {
    pub fn new(id: ComponentId, props: TaskListProps) -> Self {
        Self {
            id,
            props,
            state: TaskListComponentState::default(),
            list_state: ListState::default(),
        }
    }

    pub fn add_task(&mut self, task: TaskItem) {
        self.props.tasks.push(task);
    }

    pub fn selected_tasks(&self) -> &[String] {
        &self.state.selected_tasks
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ComponentResult<bool> {
        match key.code {
            KeyCode::Up => {
                if self.state.current_task > 0 {
                    self.state.current_task -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if self.state.current_task + 1 < self.props.tasks.len() {
                    self.state.current_task += 1;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render_task_list(&mut self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        let items: Vec<ListItem> = self.props.tasks
            .iter()
            .map(|task| {
                ListItem::new(Line::from(Span::raw(task.title.clone())))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_deref().unwrap_or("Tasks"))
                    .border_style(Style::default().fg(theme.colors.border))
            )
            .style(Style::default().fg(theme.colors.primary));

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

#[async_trait::async_trait]
impl Component for TaskList {
    type Props = TaskListProps;
    type State = TaskListComponentState;

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn props(&self) -> &Self::Props {
        &self.props
    }

    async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentResult<Vec<crate::presentation::components::core::ComponentEvent>> {
        let handled = self.handle_key_event(key)?;
        if handled {
            self.state.common.mark_dirty();
        }
        Ok(vec![])
    }

    async fn handle_event(&mut self, _event: crate::presentation::components::core::ComponentEvent) -> ComponentResult<Vec<crate::presentation::components::core::ComponentEvent>> {
        Ok(vec![])
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        let mut task_list_clone = self.clone();
        task_list_clone.render_task_list(frame, area, &theme);
    }

    async fn update_from_state(&mut self, _state_event: &crate::state::StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        ValidationState::Valid
    }
}

impl Clone for TaskList {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            state: self.state.clone(),
            list_state: ListState::default(),
        }
    }
} 
