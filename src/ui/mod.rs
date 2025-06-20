pub mod components;
pub mod loading;
pub mod screens;
pub mod state;

// Re-export the main types and functions for easy access
pub use components::{draw_status_bar, draw_title_bar};
pub use loading::draw_loading_overlay;
pub use screens::{
    draw_commit_preview_screen, draw_commit_screen, draw_config_screen, draw_main_screen,
    draw_release_notes_screen, draw_task_search_screen,
};
pub use state::{CommitField, InputMode, UIState};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::git::GitStatus;
use crate::types::{
    AppConfig, AppScreen, AppState, CommitForm, JiraTask, MondayTask, SemanticReleaseState,
};
use crate::ui::screens::semantic_release::draw_semantic_release_screen;

// Main drawing orchestrator function
#[allow(clippy::too_many_arguments)]
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
    semantic_release_state: Option<&SemanticReleaseState>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title bar
    draw_title_bar(f, chunks[0]);

    // Update textarea styles before rendering
    ui_state.update_textarea_styles();

    // Main content based on current screen
    match app_screen {
        AppScreen::Main => draw_main_screen(f, chunks[1], ui_state, git_status),
        AppScreen::Config => draw_config_screen(f, chunks[1]),
        AppScreen::Commit => draw_commit_screen(f, chunks[1], ui_state, commit_form),
        AppScreen::CommitPreview => draw_commit_preview_screen(f, chunks[1], ui_state),
        AppScreen::ReleaseNotes => draw_release_notes_screen(f, chunks[1]),
        AppScreen::SemanticRelease => draw_semantic_release_screen(
            f,
            chunks[1],
            ui_state,
            config,
            app_state,
            message,
            semantic_release_state,
        ),
        AppScreen::TaskSearch => draw_task_search_screen(
            f,
            chunks[1],
            ui_state,
            monday_tasks,
            jira_tasks,
            config,
            commit_form,
        ),
    }

    // Status bar
    draw_status_bar(f, chunks[2], app_state, message);

    // Loading overlay
    if matches!(app_state, AppState::Loading) {
        // Increment animation frame for spinner
        ui_state.animation_frame = ui_state.animation_frame.wrapping_add(1);
        draw_loading_overlay(f, f.area(), ui_state.animation_frame, message);
    }

    // Note: Cursor positioning is now handled by tui-textarea internally
    // No need for manual cursor positioning for text fields
}
