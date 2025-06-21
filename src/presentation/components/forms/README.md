# Form Components

This module contains form components built on our component framework, providing validation, reactive state integration, and modern UX patterns.

## TextInput Component

The `TextInput` component is a feature-rich text input widget that wraps `tui-textarea::TextArea` with our component framework. It provides:

### Features

- **Based on tui-textarea**: Leverages the mature `tui-textarea` (v0.7.0) library for robust text editing
- **Component Framework Integration**: Implements our `Component` trait with full lifecycle support
- **Validation**: Built-in validation with regex patterns, required fields, and max length
- **Single/Multi-line Support**: Configurable for single-line inputs or multi-line text areas
- **Reactive State**: Integrates with our state management system
- **Builder Pattern**: Fluent API for easy configuration
- **Focus Management**: Proper focus handling with visual indicators
- **Keyboard Navigation**: Full keyboard support including tab navigation

### Usage Examples

```rust
use crate::presentation::components::forms::{TextInput, TextInputProps};
use crate::presentation::components::core::ComponentId;

// Simple text input
let input = TextInput::new(TextInputProps {
    id: ComponentId::new("username"),
    title: "Username".to_string(),
    placeholder: "Enter your username...".to_string(),
    ..Default::default()
});

// Using builder pattern
let email_input = TextInput::new(TextInputProps::default())
    .with_title("Email Address")
    .with_placeholder("user@example.com")
    .with_validation_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
    .required(true);

// Multi-line text area
let description = TextInput::new(TextInputProps::default())
    .with_title("Description")
    .with_placeholder("Enter a detailed description...")
    .multiline(true)
    .max_length(500);
```

### Integration with Existing Code

The TextInput component can directly replace existing `tui-textarea::TextArea` usage in the codebase:

**Before** (existing code in `src/ui/state.rs`):
```rust
pub scope_textarea: TextArea<'static>,
pub title_textarea: TextArea<'static>,
// ... other TextArea fields
```

**After** (with new component):
```rust
pub scope_input: TextInput,
pub title_input: TextInput,
// ... other TextInput components
```

### Key Advantages over Direct TextArea Usage

1. **Validation**: Built-in validation with visual feedback
2. **Consistency**: Standardized appearance and behavior across the app
3. **State Management**: Automatic integration with reactive state
4. **Events**: Structured event system for value changes, validation, etc.
5. **Styling**: Consistent styling with focus states and validation indicators
6. **Reusability**: Easy to reuse across different screens and contexts

### API Reference

#### TextInputProps
- `id: ComponentId` - Unique identifier
- `placeholder: String` - Placeholder text
- `title: String` - Component title/label
- `multiline: bool` - Enable multi-line mode
- `max_length: Option<usize>` - Maximum character limit
- `required: bool` - Mark as required field
- `readonly: bool` - Make read-only
- `tab_length: u8` - Tab size for indentation
- `initial_value: String` - Initial text content
- `validation_pattern: Option<String>` - Regex validation pattern
- `help_text: Option<String>` - Additional help text

#### Key Methods
- `text() -> String` - Get current text content
- `set_text(&mut self, text: &str)` - Set text content
- `clear(&mut self)` - Clear all text
- `is_valid() -> bool` - Check validation status
- `is_empty() -> bool` - Check if empty
- `validation_message() -> Option<&str>` - Get validation error message

### Visual States

The component provides visual feedback for different states:

- **Focused**: Green border when valid, red when invalid
- **Unfocused**: Gray border when valid, light red when invalid
- **Required**: Asterisk (*) in title
- **Modified**: "(modified)" indicator in title
- **Validation**: Emoji indicators (✅ ❌ ⚠️) in title
- **Help Available**: (?) indicator when help text is provided

### Testing

The component includes comprehensive tests covering:
- Component creation and configuration
- Text manipulation operations
- Validation scenarios (required fields, max length, regex patterns)
- Keyboard event handling
- Focus state management
- Component lifecycle events

Run tests with:
```bash
cargo test --features new-components
```

### Migration Guide

To migrate existing TextArea usage to TextInput:

1. Replace `TextArea<'static>` with `TextInput`
2. Use `TextInputProps` for configuration instead of manual TextArea setup
3. Replace direct TextArea method calls with TextInput methods
4. Add validation rules using builder methods
5. Update event handling to use the component event system

This migration provides immediate benefits in validation, consistency, and maintainability while preserving all existing text editing functionality. 