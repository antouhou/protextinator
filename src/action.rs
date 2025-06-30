//! Text editing actions and their results.
//!
//! This module defines the various actions that can be performed on text (like inserting,
//! deleting, copying, etc.) and the results of those actions.

use smol_str::SmolStr;

/// Represents the different text editing actions that can be performed.
///
/// Actions encapsulate text editing operations like insertion, deletion, cursor movement,
/// and clipboard operations. They are processed by the text state to modify the text buffer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Paste text from the clipboard at the current cursor position.
    Paste(String),
    /// Cut selected text to the clipboard and remove it from the buffer.
    Cut,
    /// Copy selected text to the clipboard without removing it.
    CopySelectedText,
    /// Select all text in the buffer.
    SelectAll,
    /// Delete the character before the cursor (backspace).
    DeleteBackward,
    /// Insert whitespace at the cursor position.
    InsertWhitespace,
    /// Move the cursor one position to the right.
    MoveCursorRight,
    /// Move the cursor one position to the left.
    MoveCursorLeft,
    /// Move the cursor down one line.
    MoveCursorDown,
    /// Move the cursor up one line.
    MoveCursorUp,
    /// Insert a character or string at the cursor position.
    InsertChar(SmolStr),
}

/// The result of applying an action to a text state.
///
/// Action results indicate what happened when an action was processed,
/// allowing the application to respond appropriately (e.g., updating the UI,
/// accessing the clipboard, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionResult {
    /// No action was performed or no change occurred.
    None,
    /// The cursor position was updated.
    CursorUpdated,
    /// The text content was modified.
    TextChanged,
    /// Text should be inserted into the system clipboard.
    InsertToClipboard(String),
    /// Actions are disabled for this text state.
    ActionsDisabled,
}

impl ActionResult {
    /// Returns `true` if the result is `None` (no action was performed).
    ///
    /// This is useful for checking if an action had any effect.
    ///
    /// # Examples
    /// ```
    /// use protextinator::ActionResult;
    ///
    /// let result = ActionResult::None;
    /// assert!(result.is_none());
    ///
    /// let result = ActionResult::TextChanged;
    /// assert!(!result.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        matches!(self, ActionResult::None)
    }
}
