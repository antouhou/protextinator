use smol_str::SmolStr;

pub enum Action {
    Paste(String),
    Cut,
    Copy,
    SelectAll,
    DeleteBackward,
    InsertWhitespace,
    MoveCursorRight,
    MoveCursorLeft,
    MoveCursorDown,
    MoveCursorUp,
    InsertChar(SmolStr),
}

pub enum ActionResult {
    None,
    InsertToClipboard(String),
}
