pub mod layout;
pub mod panels;
pub mod timeline;
pub mod viewport;
pub mod widgets;
pub mod dialogs;
pub mod notifications;
pub mod themes;
pub mod docking;
pub mod shortcuts;

pub use manager::UiManager;
pub use renderer::UiRenderer;
pub use theme::Theme;
pub use layout::Layout;
pub use panels::PanelManager;
pub use widgets::WidgetManager;

mod manager;
mod renderer;
mod theme;
