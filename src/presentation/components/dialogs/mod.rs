//! Dialog Components Module
//! 
//! Reusable dialog components for user interactions.
//! 
//! NOTE: Dialog components are temporarily disabled due to compilation issues.
//! They will be implemented in a future phase.

// TODO: Implement dialog components like modals, confirmations, etc.
// - Modal component with animations and focus management
// - Confirmation dialog with customizable buttons
// - Alert dialog for notifications
// - Input dialog for user input collection

#[cfg(feature = "new-components")]
pub mod modal;
#[cfg(feature = "new-components")]
pub mod confirmation;

#[cfg(feature = "new-components")]
pub use modal::{Modal, ModalProps, ModalComponentState};
#[cfg(feature = "new-components")]
pub use confirmation::{Confirmation, ConfirmationProps, ConfirmationComponentState};
