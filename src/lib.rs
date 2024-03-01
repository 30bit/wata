#![cfg_attr(doc, feature(doc_cfg))]

mod common;
#[cfg(feature = "make")]
mod make;
#[cfg(feature = "read")]
mod read;
#[cfg(feature = "write")]
mod write;

#[cfg(any(feature = "read", feature = "write"))]
#[cfg_attr(doc, doc(cfg(any(feature = "read", feature = "write"))))]
pub use self::common::ReadConfig;
#[cfg(feature = "write")]
#[cfg_attr(doc, doc(cfg(feature = "write")))]
pub use self::common::WriteConfig;
#[cfg(feature = "make")]
#[cfg_attr(doc, doc(cfg(feature = "make")))]
pub use self::make::make;
#[cfg(feature = "read")]
#[cfg_attr(doc, doc(cfg(feature = "read")))]
pub use self::read::read;
#[cfg(feature = "write")]
#[cfg_attr(doc, doc(cfg(feature = "write")))]
pub use self::write::write;
