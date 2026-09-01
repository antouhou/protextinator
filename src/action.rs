//! Text editing actions and their results.

use smol_str::SmolStr;

/// A text editing operation: insertion, deletion, cursor movement, or clipboard access.
///
/// The text state applies the action to its buffer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Paste text from the clipboard at the current cursor position.
    Paste(String),
    /// Cut selected text to the clipboard and remove it from the buffer.
    Cut,
    /// Cut the characters in `start..end` (character indices, clamped to the text bounds)
    /// and remove them from the buffer, placing the cursor at `start`.
    CutRange { start: usize, end: usize },
    /// Copy selected text to the clipboard without removing it.
    CopySelectedText,
    /// Select all text in the buffer.
    SelectAll,
    /// Delete the character before the cursor (backspace).
    DeleteBackward,
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

/// The result of applying an [`Action`] to a text state.
///
/// Tells the application what happened, e.g. whether the text changed or the
/// clipboard needs updating.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionResult {
    /// No action was performed or no change occurred.
    None,
    /// The cursor position was updated.
    CursorUpdated,
    /// The text content was modified.
    TextChanged,
    /// Text should be inserted into the system clipboard.
    TextCopied(String),
    /// Text should be inserted into the system clipboard, and the original text was cut.
    TextCut(String),
    /// Actions are disabled for this text state.
    ActionsDisabled,
}

impl ActionResult {
    /// Returns `true` if no action was performed.
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
