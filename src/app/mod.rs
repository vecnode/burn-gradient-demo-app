// Platform-specific app components

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(any(feature = "web", feature = "server"))]
pub mod web;

#[cfg(feature = "desktop")]
pub mod desktop_server;

#[cfg(feature = "desktop")]
pub mod web_server;
