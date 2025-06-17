pub mod state;
pub mod components;
pub mod screens;
pub mod loading;
pub mod cursor;
pub mod scrollable_text;

// Re-export the main types and functions for easy access
pub use state::{UIState, InputMode, CommitField};
pub use components::{draw_title_bar, draw_status_bar};
pub use screens::{
    draw_main_screen, draw_config_screen, draw_commit_screen, 
    draw_commit_preview_screen, draw_release_notes_screen,
    draw_task_search_screen
};
pub use loading::draw_loading_overlay;
pub use cursor::{
    set_cursor_position, set_commit_preview_cursor_position, 
    set_search_cursor_position
};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::types::{AppScreen, AppState, CommitForm, MondayTask, JiraTask, AppConfig};
use crate::git::GitStatus;
use crate::ui::screens::semantic_release::draw_semantic_release_screen;

// Main drawing orchestrator function
pub fn draw(
    f: &mut Frame,
    app_screen: &AppScreen,
    app_state: &AppState,
    ui_state: &mut UIState,
    commit_form: &CommitForm,
    monday_tasks: &[MondayTask],
    jira_tasks: &[JiraTask],
    config: &AppConfig,
    message: Option<&str>,
    git_status: Option<&GitStatus>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Title bar
    draw_title_bar(f, chunks[0]);

    // Main content based on current screen
    match app_screen {
        AppScreen::Main => draw_main_screen(f, chunks[1], ui_state, git_status),
        AppScreen::Config => draw_config_screen(f, chunks[1]),
        AppScreen::Commit => draw_commit_screen(f, chunks[1], ui_state, commit_form),
        AppScreen::CommitPreview => draw_commit_preview_screen(f, chunks[1], ui_state),
        AppScreen::ReleaseNotes => draw_release_notes_screen(f, chunks[1]),
        AppScreen::SemanticRelease => draw_semantic_release_screen(f, chunks[1], ui_state, config, app_state, message),
        AppScreen::TaskSearch => draw_task_search_screen(f, chunks[1], ui_state, monday_tasks, jira_tasks, config, commit_form),
    }

    // Status bar
    draw_status_bar(f, chunks[2], app_state, message);

    // Loading overlay
    if matches!(app_state, AppState::Loading) {
        // Increment animation frame for spinner
        ui_state.animation_frame = ui_state.animation_frame.wrapping_add(1);
        draw_loading_overlay(f, f.area(), ui_state.animation_frame, message);
    }

    // Set cursor position when editing
    if ui_state.input_mode == InputMode::Editing {
        match app_screen {
            AppScreen::Commit => set_cursor_position(f, chunks[1], ui_state),
            AppScreen::CommitPreview => set_commit_preview_cursor_position(f, chunks[1], ui_state),
            AppScreen::TaskSearch => set_search_cursor_position(f, chunks[1], ui_state),
            _ => {}
        }
    }
} 