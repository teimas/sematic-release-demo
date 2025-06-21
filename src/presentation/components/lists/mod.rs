//! List Components
//! 
//! This module provides list-based UI components including tables, trees, and search lists.

#[cfg(feature = "new-components")]
pub mod table;
#[cfg(feature = "new-components")]
pub mod tree;
#[cfg(feature = "new-components")]
pub mod task_list;
#[cfg(feature = "new-components")]
pub mod search_list;

// Re-exports for new-components feature
#[cfg(feature = "new-components")]
pub use table::*;
#[cfg(feature = "new-components")]
pub use tree::*;
#[cfg(feature = "new-components")]
pub use task_list::*;
#[cfg(feature = "new-components")]
pub use search_list::*;

// Legacy/simplified exports for new-domains without new-components
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub mod simple_lists {
    //! Simplified list components for new-domains feature without full component framework
    
    use ratatui::{
        widgets::{List, ListItem, ListState},
        style::{Color, Style},
        text::{Line, Span},
    };
    
    /// Simple task list for new-domains compatibility
    pub struct SimpleTaskList {
        pub items: Vec<String>,
        pub state: ListState,
    }
    
    impl SimpleTaskList {
        pub fn new(items: Vec<String>) -> Self {
            Self {
                items,
                state: ListState::default(),
            }
        }
        
        pub fn render(&self) -> List {
            let items: Vec<ListItem> = self.items
                .iter()
                .map(|item| ListItem::new(Line::from(Span::styled(item.clone(), Style::default().fg(Color::White)))))
                .collect();
            
            List::new(items)
                .style(Style::default().fg(Color::White))
        }
    }
    
    /// Simple table for new-domains compatibility
    pub struct SimpleTable {
        pub headers: Vec<String>,
        pub rows: Vec<Vec<String>>,
    }
    
    impl SimpleTable {
        pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
            Self { headers, rows }
        }
    }
}

#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub use simple_lists::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::components::core::ComponentId;

    #[test]
    fn test_all_list_components_creation() {
        // Test Table creation
        let table_props = TableProps::default();
        let _table = Table::new(ComponentId::new("test_table"), table_props);

        // Test Tree creation
        let tree_props = TreeProps::default();
        let _tree = Tree::new(ComponentId::new("test_tree"), tree_props);

        // Test TaskList creation
        let task_list_props = TaskListProps::default();
        let _task_list = TaskList::new(ComponentId::new("test_task_list"), task_list_props);

        // Test SearchList creation
        let search_list_props = SearchListProps::default();
        let _search_list = SearchList::new(ComponentId::new("test_search_list"), search_list_props);
    }
}
