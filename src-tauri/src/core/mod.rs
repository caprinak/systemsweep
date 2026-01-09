// Core module - exports all core functionality

pub mod scanner;
pub mod duplicates;
pub mod cleaner;
pub mod rules;
pub mod undo;
pub mod state;

pub use scanner::FileScanner;
pub use duplicates::DuplicateFinder;
pub use cleaner::Cleaner;
pub use rules::{CleanupRule, RuleEngine};
pub use undo::UndoManager;
