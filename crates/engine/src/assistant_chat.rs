//! Save chat with file: the desktop app's assistant conversation about a project, written into its
//! `.fcproj` file when the person turns it on.
//!
//! The conversation is one more top-level key of the file's envelope, [`KEY`], next to `project`
//! and `view`; readers skip envelope keys they don't know, so every version still opens the file.
//! The desktop app sets a [`ChatHook`]; without one (automation, the web app) a project is written
//! as it is. A save writes the envelope afresh, so a project saved with the option off has no
//! conversation in it.

use std::sync::OnceLock;

use serde_json::Value;

/// The envelope key that holds the conversation.
pub const KEY: &str = "artcraftAssistant";

/// The conversation to save in the project file at a path (None: none, or the option off).
pub type ChatHook = Box<dyn Fn(&str) -> Option<Value> + Send + Sync>;

static HOOK: OnceLock<ChatHook> = OnceLock::new();

/// Sets the [`ChatHook`] (once per run).
pub fn set_hook(hook: ChatHook) {
    let _ = HOOK.set(hook);
}

/// The project file `bytes` (an encoded envelope) about to be written to `path`, with the
/// conversation the hook gives for it.
pub(crate) fn hooked(path: &str, bytes: Vec<u8>) -> Vec<u8> {
    match HOOK.get().and_then(|hook| hook(path)) {
        Some(chat) => with_chat(bytes, &chat),
        None => bytes,
    }
}

/// `envelope` (a JSON object, as `filmcraft_format` writes it) with `chat` under [`KEY`] as its
/// last key. Anything else is returned as it is.
pub fn with_chat(mut envelope: Vec<u8>, chat: &Value) -> Vec<u8> {
    let Some(close) = envelope.iter().rposition(|b| !b.is_ascii_whitespace()) else { return envelope };
    // An object with members: `{`, then something other than `}` before its closing `}`.
    let empty = envelope.get(..close).and_then(|b| b.iter().rposition(|b| !b.is_ascii_whitespace())) == Some(0);
    if envelope.first() != Some(&b'{') || envelope.get(close) != Some(&b'}') || close == 0 || empty {
        return envelope;
    }
    let tail = envelope.split_off(close);
    envelope.extend_from_slice(format!(",\"{KEY}\":{chat}").as_bytes());
    envelope.extend_from_slice(&tail);
    envelope
}

/// The conversation saved in a project file, if any.
pub fn chat_in(bytes: &[u8]) -> Option<Value> {
    let mut envelope: serde_json::Map<String, Value> = serde_json::from_slice(bytes).ok()?;
    envelope.remove(KEY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_chat_rides_in_the_envelope_and_the_project_still_opens() {
        let project = filmcraft_project::Project::default();
        let chat = json!({"version": 1, "messages": [{"role": "user", "text": "Cắt đoạn đầu"}]});
        let bytes = with_chat(filmcraft_format::encode(&project, false), &chat);
        assert_eq!(chat_in(&bytes), Some(chat.clone()));
        assert!(filmcraft_format::decode(&bytes).is_ok(), "the project opens with the conversation in it");
        let pretty = with_chat(filmcraft_format::encode(&project, true), &chat);
        assert_eq!(chat_in(&pretty), Some(chat.clone()));
        assert_eq!(chat_in(&filmcraft_format::encode(&project, false)), None);
        assert_eq!(with_chat(b"[1]".to_vec(), &chat), b"[1]", "not an envelope: as it is");
        assert_eq!(with_chat(b"{ }".to_vec(), &chat), b"{ }");
    }
}
