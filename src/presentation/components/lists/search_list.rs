use crate::presentation::components::core::base::{
    Component, ComponentState, ComponentProps, CommonComponentState,
};
use crate::presentation::components::core::{
    ComponentResult, ComponentId, ValidationState, VisibilityState,
};
use crate::presentation::theme::AppTheme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fuzzy matching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatcher {
    pub case_sensitive: bool,
    pub prefix_bonus: f32,
    pub exact_match_bonus: f32,
    pub sequential_bonus: f32,
    pub min_score_threshold: f32,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            prefix_bonus: 10.0,
            exact_match_bonus: 50.0,
            sequential_bonus: 5.0,
            min_score_threshold: 0.1,
        }
    }
}

impl FuzzyMatcher {
    /// Calculate fuzzy match score between query and text
    pub fn score(&self, query: &str, text: &str) -> Option<f32> {
        if query.is_empty() {
            return Some(1.0);
        }

        let query = if self.case_sensitive { query.to_string() } else { query.to_lowercase() };
        let text = if self.case_sensitive { text.to_string() } else { text.to_lowercase() };

        // Exact match
        if query == text {
            return Some(100.0 + self.exact_match_bonus);
        }

        // Prefix match
        if text.starts_with(&query) {
            return Some(80.0 + self.prefix_bonus);
        }

        // Contains match
        if text.contains(&query) {
            return Some(60.0);
        }

        // Fuzzy match
        let fuzzy_score = self.fuzzy_score(&query, &text);
        if fuzzy_score >= self.min_score_threshold {
            Some(fuzzy_score * 40.0)
        } else {
            None
        }
    }

    /// Calculate fuzzy matching score using character-by-character matching
    fn fuzzy_score(&self, query: &str, text: &str) -> f32 {
        let query_chars: Vec<char> = query.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        if query_chars.is_empty() {
            return 1.0;
        }

        let mut query_idx = 0;
        let mut matches = 0;
        let mut sequential = 0;
        let mut max_sequential = 0;

        for (_text_idx, &text_char) in text_chars.iter().enumerate() {
            if query_idx < query_chars.len() && query_chars[query_idx] == text_char {
                matches += 1;
                query_idx += 1;
                sequential += 1;
                max_sequential = max_sequential.max(sequential);
            } else {
                sequential = 0;
            }
        }

        if matches == 0 {
            return 0.0;
        }

        let match_ratio = matches as f32 / query_chars.len() as f32;
        let sequential_bonus = (max_sequential as f32 * self.sequential_bonus) / 100.0;
        
        (match_ratio + sequential_bonus).min(1.0)
    }
}

/// Trait for items that can be searched
pub trait SearchableItem {
    /// Get the primary text to search in
    fn search_text(&self) -> &str;
    
    /// Get secondary text fields to search in (optional)
    fn secondary_search_text(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// Get display text for the item
    fn display_text(&self) -> String {
        self.search_text().to_string()
    }
    
    /// Get unique identifier for the item
    fn id(&self) -> String;
}

/// Basic searchable item implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicSearchableItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl BasicSearchableItem {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl SearchableItem for BasicSearchableItem {
    fn search_text(&self) -> &str {
        &self.title
    }

    fn secondary_search_text(&self) -> Vec<&str> {
        let mut texts = Vec::new();
        if let Some(desc) = &self.description {
            texts.push(desc.as_str());
        }
        for tag in &self.tags {
            texts.push(tag.as_str());
        }
        texts
    }

    fn display_text(&self) -> String {
        if let Some(desc) = &self.description {
            format!("{} - {}", self.title, desc)
        } else {
            self.title.clone()
        }
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Search result with score and highlighting information
#[derive(Debug, Clone)]
pub struct SearchResult<T: SearchableItem> {
    pub item: T,
    pub score: f32,
    pub match_type: MatchType,
    pub highlighted_ranges: Vec<(usize, usize)>, // Character ranges to highlight
}

/// Type of match found
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

impl MatchType {
    pub fn priority(&self) -> u8 {
        match self {
            MatchType::Exact => 4,
            MatchType::Prefix => 3,
            MatchType::Contains => 2,
            MatchType::Fuzzy => 1,
        }
    }
}

/// Search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_items: usize,
    pub filtered_items: usize,
    pub exact_matches: usize,
    pub prefix_matches: usize,
    pub contains_matches: usize,
    pub fuzzy_matches: usize,
    pub search_time_ms: u64,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            total_items: 0,
            filtered_items: 0,
            exact_matches: 0,
            prefix_matches: 0,
            contains_matches: 0,
            fuzzy_matches: 0,
            search_time_ms: 0,
        }
    }
}

/// Search list component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchListProps {
    pub items: Vec<BasicSearchableItem>,
    pub title: Option<String>,
    pub show_borders: bool,
    pub placeholder_text: String,
    pub matcher: FuzzyMatcher,
    pub max_results: Option<usize>,
    pub show_stats: bool,
    pub instant_search: bool, // Search as you type vs. on Enter
    pub show_scores: bool,
    pub highlight_matches: bool,
    pub empty_message: String,
}

impl Default for SearchListProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            title: None,
            show_borders: true,
            placeholder_text: "Search...".to_string(),
            matcher: FuzzyMatcher::default(),
            max_results: Some(100),
            show_stats: false,
            instant_search: true,
            show_scores: false,
            highlight_matches: true,
            empty_message: "No items found".to_string(),
        }
    }
}

impl ComponentProps for SearchListProps {
    fn default_props() -> Self {
        Self::default()
    }
}

/// Search list component state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchListComponentState {
    pub common: CommonComponentState,
    pub query: String,
    pub current_item: usize,
    pub scroll_offset: usize,
    pub search_mode: bool,
    pub results: Vec<String>, // Store item IDs for serialization
    pub stats: SearchStats,
    pub selected_items: Vec<String>,
}

impl ComponentState for SearchListComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for SearchListComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            query: String::new(),
            current_item: 0,
            scroll_offset: 0,
            search_mode: false,
            results: Vec::new(),
            stats: SearchStats::default(),
            selected_items: Vec::new(),
        }
    }
}

/// Searchable list component with fuzzy matching
pub struct SearchList {
    id: ComponentId,
    props: SearchListProps,
    state: SearchListComponentState,
    list_state: ListState,
    search_results: Vec<SearchResult<BasicSearchableItem>>, // Runtime cache
}

impl SearchList {
    pub fn new(id: ComponentId, props: SearchListProps) -> Self {
        Self {
            id,
            props,
            state: SearchListComponentState::default(),
            list_state: ListState::default(),
            search_results: Vec::new(),
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.state.query = query;
        if self.props.instant_search {
            self.perform_search();
        }
    }

    pub fn query(&self) -> &str {
        &self.state.query
    }

    pub fn results(&self) -> &[SearchResult<BasicSearchableItem>] {
        &self.search_results
    }

    pub fn current_result(&self) -> Option<&SearchResult<BasicSearchableItem>> {
        self.search_results.get(self.state.current_item)
    }

    pub fn stats(&self) -> &SearchStats {
        &self.state.stats
    }

    pub fn set_items(&mut self, items: Vec<BasicSearchableItem>) {
        self.props.items = items;
        self.perform_search();
    }

    pub fn add_item(&mut self, item: BasicSearchableItem) {
        self.props.items.push(item);
        self.perform_search();
    }

    pub fn perform_search(&mut self) {
        let start_time = std::time::Instant::now();
        
        self.search_results.clear();
        
        for item in &self.props.items {
            if let Some(score) = self.props.matcher.score(&self.state.query, item.search_text()) {
                let match_type = self.determine_match_type(&self.state.query, item.search_text());
                let highlighted_ranges = if self.props.highlight_matches {
                    self.find_highlight_ranges(&self.state.query, item.search_text())
                } else {
                    Vec::new()
                };

                self.search_results.push(SearchResult {
                    item: item.clone(),
                    score,
                    match_type,
                    highlighted_ranges,
                });
            }
        }

        self.search_results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(max_results) = self.props.max_results {
            self.search_results.truncate(max_results);
        }

        self.state.results = self.search_results.iter().map(|r| r.item.id()).collect();
        
        if self.state.current_item >= self.search_results.len() {
            self.state.current_item = 0;
        }
        
        let search_duration = start_time.elapsed();
        self.update_stats(search_duration);
    }

    fn determine_match_type(&self, query: &str, text: &str) -> MatchType {
        let query = if self.props.matcher.case_sensitive { query } else { &query.to_lowercase() };
        let text = if self.props.matcher.case_sensitive { text.to_string() } else { text.to_lowercase() };
        
        if query == text {
            MatchType::Exact
        } else if text.starts_with(query) {
            MatchType::Prefix
        } else if text.contains(query) {
            MatchType::Contains
        } else {
            MatchType::Fuzzy
        }
    }

    fn find_highlight_ranges(&self, query: &str, text: &str) -> Vec<(usize, usize)> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut ranges = Vec::new();
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        if let Some(pos) = text_lower.find(&query_lower) {
            ranges.push((pos, pos + query.len()));
        }

        ranges
    }

    fn update_stats(&mut self, search_duration: std::time::Duration) {
        self.state.stats = SearchStats {
            total_items: self.props.items.len(),
            filtered_items: self.search_results.len(),
            exact_matches: self.search_results.iter().filter(|r| r.match_type == MatchType::Exact).count(),
            prefix_matches: self.search_results.iter().filter(|r| r.match_type == MatchType::Prefix).count(),
            contains_matches: self.search_results.iter().filter(|r| r.match_type == MatchType::Contains).count(),
            fuzzy_matches: self.search_results.iter().filter(|r| r.match_type == MatchType::Fuzzy).count(),
            search_time_ms: search_duration.as_millis() as u64,
        };
    }

    pub fn toggle_selection(&mut self) {
        if let Some(current_result) = self.current_result() {
            let item_id = current_result.item.id();
            
            if self.state.selected_items.contains(&item_id) {
                self.state.selected_items.retain(|id| id != &item_id);
            } else {
                self.state.selected_items.push(item_id);
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.state.current_item > 0 {
            self.state.current_item -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.state.current_item + 1 < self.search_results.len() {
            self.state.current_item += 1;
        }
    }

    pub fn enter_search_mode(&mut self) {
        self.state.search_mode = true;
    }

    pub fn exit_search_mode(&mut self) {
        self.state.search_mode = false;
    }

    pub fn clear_search(&mut self) {
        self.state.query.clear();
        self.perform_search();
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ComponentResult<bool> {
        if self.state.search_mode {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Ok(true)
            }
            KeyCode::Up => {
                self.move_up();
                Ok(true)
            }
            KeyCode::Down => {
                self.move_down();
                Ok(true)
            }
            KeyCode::Enter => {
                self.toggle_selection();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> ComponentResult<bool> {
        match key.code {
            KeyCode::Char(c) => {
                self.state.query.push(c);
                if self.props.instant_search {
                    self.perform_search();
                }
                Ok(true)
            }
            KeyCode::Backspace => {
                self.state.query.pop();
                if self.props.instant_search {
                    self.perform_search();
                }
                Ok(true)
            }
            KeyCode::Enter => {
                if !self.props.instant_search {
                    self.perform_search();
                }
                self.exit_search_mode();
                Ok(true)
            }
            KeyCode::Esc => {
                self.exit_search_mode();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render_search_list(&mut self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        if self.state.search_mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);
            
            self.render_search_input(frame, chunks[0], theme);
            self.render_results(frame, chunks[1], theme);
        } else {
            self.render_results(frame, area, theme);
        }
    }

    fn render_search_input(&self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        let search_text = if self.state.query.is_empty() {
            self.props.placeholder_text.as_str()
        } else {
            self.state.query.as_str()
        };

        let input = Paragraph::new(search_text)
            .style(Style::default().fg(theme.colors.foreground))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(Style::default().fg(theme.colors.focus))
            );

        frame.render_widget(input, area);
    }

    fn render_results(&mut self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        if self.search_results.is_empty() {
            self.render_empty(frame, area, theme);
            return;
        }

        let items: Vec<ListItem> = self.search_results
            .iter()
            .map(|result| self.create_result_item(result, theme))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_deref().unwrap_or("Results"))
                    .border_style(Style::default().fg(theme.colors.border))
            )
            .style(Style::default().fg(theme.colors.primary))
            .highlight_style(Style::default().fg(theme.colors.focus).bg(theme.colors.focus_bg));

        let mut list_state_clone = self.list_state.clone();
        list_state_clone.select(Some(self.state.current_item));
        frame.render_stateful_widget(list, area, &mut list_state_clone);
        self.list_state = list_state_clone;
    }

    fn create_result_item(&self, result: &SearchResult<BasicSearchableItem>, theme: &AppTheme) -> ListItem {
        let display_text = result.item.display_text();
        let mut spans = Vec::new();

        if self.props.highlight_matches && !result.highlighted_ranges.is_empty() {
            let mut last_end = 0;
            
            for &(start, end) in &result.highlighted_ranges {
                if start > last_end {
                    spans.push(Span::raw(display_text[last_end..start].to_string()));
                }
                
                spans.push(Span::styled(
                    display_text[start..end].to_string(),
                    Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD)
                ));
                
                last_end = end;
            }
            
            if last_end < display_text.len() {
                spans.push(Span::raw(display_text[last_end..].to_string()));
            }
        } else {
            spans.push(Span::raw(display_text));
        }

        if self.props.show_scores {
            spans.push(Span::styled(
                format!(" ({:.1})", result.score),
                Style::default().fg(theme.colors.secondary)
            ));
        }

        ListItem::new(Line::from(spans))
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        let empty_text = Paragraph::new(self.props.empty_message.as_str())
            .style(Style::default().fg(theme.colors.secondary))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_deref().unwrap_or("Search"))
                    .border_style(Style::default().fg(theme.colors.border))
            );

        frame.render_widget(empty_text, area);
    }
}

#[async_trait::async_trait]
impl Component for SearchList {
    type Props = SearchListProps;
    type State = SearchListComponentState;

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
        let mut search_list_clone = self.clone();
        search_list_clone.render_search_list(frame, area, &theme);
    }

    async fn update_from_state(&mut self, _state_event: &crate::state::StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        ValidationState::Valid
    }
}

impl Clone for SearchList {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            state: self.state.clone(),
            list_state: ListState::default(),
            search_results: self.search_results.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher() {
        let matcher = FuzzyMatcher::default();
        
        assert!(matcher.score("test", "test").is_some());
        assert!(matcher.score("te", "test").is_some());
        assert!(matcher.score("xyz", "test").is_none());
    }

    #[test]
    fn test_searchable_item() {
        let item = BasicSearchableItem::new("1".to_string(), "Test Item".to_string())
            .with_description("A test item".to_string());
        
        assert_eq!(item.search_text(), "Test Item");
        assert_eq!(item.id(), "1");
    }

    #[test]
    fn test_search_list() {
        let props = SearchListProps::default();
        let search_list = SearchList::new(ComponentId::new("test_search_list"), props);
        
        assert_eq!(search_list.query(), "");
        assert!(search_list.results().is_empty());
    }
} 