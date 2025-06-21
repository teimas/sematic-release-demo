//! Layout Components Module
//! 
//! Layout components for organizing UI structure.

#[cfg(feature = "new-components")]
pub mod split_pane;
#[cfg(feature = "new-components")]
pub mod tabs;
#[cfg(feature = "new-components")]
pub mod sidebar;
#[cfg(feature = "new-components")]
pub mod status_bar;

#[cfg(feature = "new-components")]
pub use split_pane::{SplitPane, SplitPaneProps, SplitPaneComponentState, SplitOrientation};
#[cfg(feature = "new-components")]
pub use tabs::{Tabs, TabsProps, TabsComponentState, TabItem};
#[cfg(feature = "new-components")]
pub use sidebar::{Sidebar, SidebarProps, SidebarComponentState, SidebarItem};
#[cfg(feature = "new-components")]
pub use status_bar::{StatusBar, StatusBarProps, StatusBarComponentState}; 