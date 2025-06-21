//! TextInput Component Demo
//! 
//! This demonstrates how to use the TextInput component which wraps
//! tui-textarea::TextArea with our component framework.

#[cfg(feature = "new-components")]
use crate::presentation::components::forms::text_input::{TextInput, TextInputProps};
#[cfg(feature = "new-components")]
use crate::presentation::components::core::{Component, ComponentId, FocusState};

#[cfg(feature = "new-components")]
pub fn create_demo_text_inputs() -> Vec<TextInput> {
    vec![
        // Simple single-line input
        TextInput::new(TextInputProps {
            id: ComponentId::new("simple_input"),
            title: "Simple Input".to_string(),
            placeholder: "Enter some text...".to_string(),
            ..Default::default()
        }),

        // Required field with validation
        TextInput::new(TextInputProps::default())
            .with_title("Required Field")
            .with_placeholder("This field is required")
            .required(true),

        // Multi-line text area
        TextInput::new(TextInputProps::default())
            .with_title("Description")
            .with_placeholder("Enter a detailed description...")
            .multiline(true)
            .max_length(500),

        // Email input with validation pattern
        TextInput::new(TextInputProps::default())
            .with_title("Email Address")
            .with_placeholder("user@example.com")
            .with_validation_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .required(true),

        // Read-only display field
        TextInput::new(TextInputProps::default())
            .with_title("Read-Only Field")
            .with_initial_value("This value cannot be changed")
            .readonly(true),

        // Password-style input (would need additional styling)
        TextInput::new(TextInputProps::default())
            .with_title("Secure Input")
            .with_placeholder("Enter sensitive data...")
            .max_length(50)
            .required(true)
            .with_help_text("This field has special security requirements"),
    ]
}

#[cfg(feature = "new-components")]
pub fn demo_component_operations() {
    println!("=== TextInput Component Demo ===");
    
    // Create a sample text input
    let mut input = TextInput::new(TextInputProps::default())
        .with_title("Demo Input")
        .with_placeholder("Type something...")
        .required(true);

    println!("Initial state:");
    println!("  Text: '{}'", input.text());
    println!("  Is empty: {}", input.is_empty());
    println!("  Is valid: {}", input.is_valid());
    
    // Simulate text input
    input.set_text("Hello, World!");
    println!("\nAfter setting text:");
    println!("  Text: '{}'", input.text());
    println!("  Character count: {}", input.char_count());
    println!("  Is empty: {}", input.is_empty());
    println!("  Is valid: {}", input.is_valid());
    
    // Simulate focus state changes
    input.set_focus_state(FocusState::Focused);
    println!("\nAfter focusing:");
    println!("  Focus state: {:?}", input.focus_state());
    println!("  Can focus: {}", input.can_focus());
    
    // Clear the text
    input.clear();
    println!("\nAfter clearing:");
    println!("  Text: '{}'", input.text());
    println!("  Is empty: {}", input.is_empty());
    println!("  Is valid: {}", input.is_valid());
    
    if let Some(msg) = input.validation_message() {
        println!("  Validation message: {}", msg);
    }
}

#[cfg(feature = "new-components")]
pub fn demo_integration_with_existing_textarea() {
    println!("\n=== Integration with Existing tui-textarea Usage ===");
    
    // Show how the new component can replace existing TextArea usage
    let mut input = TextInput::new(TextInputProps::default())
        .with_title("Commit Message")
        .with_placeholder("Enter commit message...")
        .multiline(true);
    
    // Simulate the kind of operations done in the existing codebase
    input.set_text("feat: add new text input component\n\nThis component wraps tui-textarea with validation and component framework integration.");
    
    println!("Commit message preview:");
    for (i, line) in input.lines().iter().enumerate() {
        println!("  {}: {}", i + 1, line);
    }
    
    println!("\nText statistics:");
    println!("  Lines: {}", input.line_count());
    println!("  Characters: {}", input.char_count());
    
    // Demonstrate validation
    if input.is_valid() {
        println!("  ✅ Commit message is valid");
    } else {
        println!("  ❌ Commit message validation failed");
    }
}

#[cfg(not(feature = "new-components"))]
pub fn demo_not_available() {
    println!("TextInput component demo requires the 'new-components' feature");
    println!("Run with: cargo run --features new-components");
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "new-components")]
    use super::*;

    #[cfg(feature = "new-components")]
    #[test]
    fn test_demo_components_creation() {
        let inputs = create_demo_text_inputs();
        assert_eq!(inputs.len(), 6);
        
        // Test that each component has expected properties
        assert_eq!(inputs[0].props().title, "Simple Input");
        assert!(inputs[1].props().required);
        assert!(inputs[2].props().multiline);
        assert!(inputs[3].props().validation_pattern.is_some());
        assert!(inputs[4].props().readonly);
        assert!(inputs[5].props().help_text.is_some());
    }

    #[cfg(feature = "new-components")]
    #[test]
    fn test_component_operations() {
        let mut input = TextInput::new(TextInputProps::default())
            .with_title("Test")
            .required(true);
        
        // Debug: Print initial state
        println!("DEBUG: Initial state");
        println!("  Text: '{}'", input.text());
        println!("  Is empty: {}", input.is_empty());
        println!("  Is valid: {}", input.is_valid());
        println!("  Required: {}", input.props().required);
        println!("  Validation state: {:?}", input.state().common.validation_state);
        
        // Initial state
        assert!(input.is_empty());
        assert!(!input.is_valid()); // Required field is empty
        
        // Add text
        input.set_text("Test content");
        
        // Debug: Print state after setting text
        println!("DEBUG: After setting text");
        println!("  Text: '{}'", input.text());
        println!("  Is empty: {}", input.is_empty());
        println!("  Is valid: {}", input.is_valid());
        println!("  Validation state: {:?}", input.state().common.validation_state);
        
        assert!(!input.is_empty());
        assert!(input.is_valid());
        
        // Clear text
        input.clear();
        
        // Debug: Print state after clearing
        println!("DEBUG: After clearing");
        println!("  Text: '{}'", input.text());
        println!("  Is empty: {}", input.is_empty());
        println!("  Is valid: {}", input.is_valid());
        println!("  Validation state: {:?}", input.state().common.validation_state);
        
        assert!(input.is_empty());
        assert!(!input.is_valid()); // Required field is empty again
    }
} 