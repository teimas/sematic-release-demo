use crate::presentation::components::core::base::{
    Component, ComponentState, ComponentProps, CommonComponentState,
};
use crate::presentation::components::core::{
    ComponentResult, ComponentId, ValidationState,
};
use crate::presentation::theme::AppTheme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Rect, Constraint},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table as RatatuiTable, TableState},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Table cell alignment options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
}

impl Default for TableAlignment {
    fn default() -> Self {
        TableAlignment::Left
    }
}

/// Sort direction for table columns
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

/// Table column definition with sorting and filtering capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub title: String,
    pub sortable: bool,
    pub filterable: bool,
    pub alignment: TableAlignment,
}

impl TableColumn {
    pub fn new(key: String, title: String) -> Self {
        Self {
            key,
            title,
            sortable: true,
            filterable: true,
            alignment: TableAlignment::Left,
        }
    }
}

/// Table row data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: String,
    pub cells: HashMap<String, String>,
    pub selectable: bool,
}

impl TableRow {
    pub fn new(id: String) -> Self {
        Self {
            id,
            cells: HashMap::new(),
            selectable: true,
        }
    }
}

/// Table pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePagination {
    pub page_size: usize,
    pub current_page: usize,
    pub total_items: usize,
    pub show_page_info: bool,
}

impl TablePagination {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            current_page: 0,
            total_items: 0,
            show_page_info: true,
        }
    }
}

/// Table component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableProps {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub title: Option<String>,
    pub show_header: bool,
    pub show_borders: bool,
    pub multi_select: bool,
    pub pagination: Option<TablePagination>,
    pub sort_column: Option<String>,
    pub sort_direction: SortDirection,
    pub loading: bool,
    pub empty_message: String,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            show_header: true,
            show_borders: true,
            multi_select: false,
            pagination: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            loading: false,
            empty_message: "No data available".to_string(),
        }
    }
}

impl ComponentProps for TableProps {
    fn default_props() -> Self {
        Self::default()
    }
}

/// Table component state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableComponentState {
    pub common: CommonComponentState,
    pub selected_rows: Vec<String>,
    pub current_row: usize,
    pub scroll_offset: usize,
    pub filtered_rows: Vec<usize>,
}

impl ComponentState for TableComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for TableComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_rows: Vec::new(),
            current_row: 0,
            scroll_offset: 0,
            filtered_rows: Vec::new(),
        }
    }
}

/// Simplified table component
pub struct Table {
    id: ComponentId,
    props: TableProps,
    state: TableComponentState,
    table_state: TableState,
}

impl Table {
    pub fn new(id: ComponentId, props: TableProps) -> Self {
        Self {
            id,
            props,
            state: TableComponentState::default(),
            table_state: TableState::default(),
        }
    }

    pub fn add_row(&mut self, row: TableRow) {
        self.props.rows.push(row);
    }

    pub fn selected_rows(&self) -> &[String] {
        &self.state.selected_rows
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ComponentResult<bool> {
        match key.code {
            KeyCode::Up => {
                if self.state.current_row > 0 {
                    self.state.current_row -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if self.state.current_row + 1 < self.props.rows.len() {
                    self.state.current_row += 1;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        if self.props.rows.is_empty() {
            return;
        }

        let rows: Vec<Row> = self.props.rows
            .iter()
            .map(|row| {
                let cells: Vec<Cell> = self.props.columns.iter()
                    .map(|col| {
                        let binding = String::new();
                        let value = row.cells.get(&col.key).unwrap_or(&binding);
                        Cell::from(value.clone())
                    })
                    .collect();
                Row::new(cells)
            })
            .collect();

        // Calculate column widths
        let widths: Vec<Constraint> = self.props.columns.iter()
            .map(|_| Constraint::Percentage((100 / self.props.columns.len().max(1)) as u16))
            .collect();

        let table = RatatuiTable::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_deref().unwrap_or("Table"))
                    .border_style(Style::default().fg(theme.colors.border))
            )
            .style(Style::default().fg(theme.colors.primary));

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}

#[async_trait::async_trait]
impl Component for Table {
    type Props = TableProps;
    type State = TableComponentState;

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
        let mut table_clone = self.clone();
        table_clone.render_table(frame, area, &theme);
    }

    async fn update_from_state(&mut self, _state_event: &crate::state::StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        ValidationState::Valid
    }
}

impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            state: self.state.clone(),
            table_state: TableState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let props = TableProps::default();
        let table = Table::new(ComponentId::new("test_table"), props);
        
        assert_eq!(table.state.current_row, 0);
        assert!(table.state.selected_rows.is_empty());
    }
} 