#[cfg(not(feature = "concurrent"))]
pub use std::rc::Rc as Lrc;
#[cfg(feature = "concurrent")]
pub use std::sync::Arc as Lrc;
