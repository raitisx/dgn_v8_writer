//! Seed-based DGN V8 writer for 2D ADTI drawings.
//!
//! The writer never builds a DGN V8 file from nothing. It takes a seed file
//! (for ADTI: the official LKS-2020 seed, supplied by the user at runtime),
//! keeps every stream it does not need to touch byte-for-byte, and appends
//! new 2D graphical elements to one model.
//!
//! Every byte-level rule used here is recorded with its evidence in
//! `docs/FORMAT_NOTES.md`; code comments cite the rule IDs (`FN-...`).
//! Rules marked as hypotheses (`H-...`) are only used behind explicit
//! experiment switches until MicroStation-made files confirm them.

#![forbid(unsafe_code)]

pub mod document;
pub mod element;
pub mod error;
pub mod geom;
pub mod page;
pub mod seed;
pub mod spec;
pub mod text;

pub use document::{Document, SaveReport, WriteOptions};
pub use element::{Element, Geometry, Symbology, TextData};
pub use error::{Result, WriteError};
pub use geom::Point2;
pub use seed::{analyse, Dimension, SeedInfo};
pub use text::TextEncoding;
