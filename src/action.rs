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
    InsertChar(char),
}

pub enum ActionResult {
    None,
    Text(String),
}