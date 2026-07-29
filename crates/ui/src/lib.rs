pub mod dialogs;
pub mod docking;
pub mod layout;
pub mod notifications;
pub mod panels;
pub mod shortcuts;
pub mod themes;
pub mod timeline;
pub mod viewport;
pub mod widgets;

pub use layout::Layout;
pub use manager::UiManager;
pub use panels::PanelManager;
pub use renderer::UiRenderer;
pub use theme::Theme;
pub use widgets::WidgetManager;

mod manager;
mod renderer;
mod theme;
