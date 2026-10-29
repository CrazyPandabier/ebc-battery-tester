#[cfg(target_arch = "wasm32")]
#[path = "usb_wasm.rs"]
mod usb;

#[cfg(not(target_arch = "wasm32"))]
#[path = "usb_native.rs"]
mod usb;

#[cfg(not(target_arch = "wasm32"))]
mod update_check;

#[cfg(not(target_arch = "wasm32"))]
mod api;

mod app;
mod device;
mod export;
mod session;
mod timer_sync;
mod ui;
pub use app::MainApp;
