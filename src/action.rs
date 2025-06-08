use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Paste(String),
    Cut,
    CopySelectedText,
    SelectAll,
    DeleteBackward,
    InsertWhitespace,
    MoveCursorRight,
    MoveCursorLeft,
    MoveCursorDown,
    MoveCursorUp,
    InsertChar(SmolStr),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionResult {
    None,
    CursorUpdated,
    TextChanged,
    InsertToClipboard(String),
}

impl ActionResult {
    pub fn is_none(&self) -> bool {
        matches!(self, ActionResult::None)
    }
}
