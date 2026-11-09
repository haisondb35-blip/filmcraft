//! Save chat with file: the assistant's conversation about a project, written into its `.fcproj`
//! file when the person turns it on (off by default), so it travels with the file and comes back
//! when the file is opened on another computer.
//!
//! The conversation (portable form, without this computer's conversation id, see
//! `Memory::to_json`) is a key of the file's envelope (`filmcraft_engine::assistant_chat`).

use serde_json::Value;

/// The largest file read whole to find its conversation (opening a file).
const MAX_FILE: u64 = 256 << 20;

/// Files the conversation can be saved in.
pub fn can_embed(path: &str) -> bool {
    std::path::Path::new(path).extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("fcproj"))
}

/// Has saves put the conversation `a` keeps for each project file into it, and `a` read it back.
pub fn install(a: &mut super::Assistant) {
    let shelf = a.shelf.clone();
    filmcraft_engine::assistant_chat::set_hook(Box::new(move |path| {
        if !can_embed(path) {
            return None;
        }
        shelf.lock().ok().and_then(|s| s.chat_for(path))
    }));
    a.embedded_reader = Some(read);
}

/// The conversation saved in the file at `path`, if it has one.
pub fn read(path: &str) -> Option<Value> {
    if !can_embed(path) || std::fs::metadata(path).ok()?.len() > MAX_FILE {
        return None;
    }
    filmcraft_engine::assistant_chat::chat_in(&std::fs::read(path).ok()?)
}
