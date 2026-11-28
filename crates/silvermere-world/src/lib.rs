//! The world editor's builder format and its compiler (design/world-editor.md).
//!
//! The editor (MMUD_Builder) writes [`model::Area`] files: rooms named, not numbered, with
//! their editor positions, and exits written once for both sides. [`compile`] turns them into
//! `content/builder.toml`, in exactly the format `silvermere-content` reads, giving each room
//! a number free in the instance it's compiled against, and keeps the numbers in a claims
//! sidecar (`content/builder.claims.toml`) so the next compile gives a room the same one.
//!
//! What's here so far: new rooms on stock maps, their plain exits, exits into them from stock
//! rooms, and their monsters (an NPC and random monsters from the stock monster groups).
//! Typed exits, stock edits, and the claims and comb at install come later.

#![forbid(unsafe_code)]

pub mod compile;
pub mod model;

pub use compile::{Compiled, Key, Problems, Used, compile};
