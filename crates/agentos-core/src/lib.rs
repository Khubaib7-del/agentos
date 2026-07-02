pub mod model;
pub mod redact;
pub mod store;

pub use model::{Decision, NoteStatus, ReviewNote};
pub use redact::redact;
pub use store::Store;

#[cfg(test)]
mod tests {
    use super::Store;

    #[test]
    fn init_then_reopen_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store
            .add_decision("DB: PostgreSQL", Some("relational model fits"), true)
            .unwrap();
        store.add_note("use debounce on the search input").unwrap();

        let store = Store::open(dir.path()).unwrap();
        let decisions = store.decisions().unwrap();
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].locked);
        assert_eq!(decisions[0].why.as_deref(), Some("relational model fits"));
        assert_eq!(store.pending_notes().unwrap().len(), 1);
    }

    #[test]
    fn secrets_never_reach_disk() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store
            .add_decision(
                "use key sk-abcdefghijklmnop1234 for the API",
                Some("connect with postgres://app:supersecretpw@db/x"),
                false,
            )
            .unwrap();
        store.add_note("set API_KEY=abc123def456xyz first").unwrap();

        let raw = std::fs::read_to_string(dir.path().join(".agentos/decisions.json")).unwrap()
            + &std::fs::read_to_string(dir.path().join(".agentos/review-queue.json")).unwrap()
            + &std::fs::read_to_string(dir.path().join(".agentos/decisions.md")).unwrap();
        assert!(!raw.contains("sk-abcdefghijklmnop1234"));
        assert!(!raw.contains("supersecretpw"));
        assert!(!raw.contains("abc123def456xyz"));
        assert!(raw.contains("[redacted:api-key]"));
    }

    #[test]
    fn mark_delivered_removes_from_pending() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let a = store.add_note("first").unwrap();
        store.add_note("second").unwrap();
        store.mark_delivered(&[a.id]).unwrap();
        let pending = store.pending_notes().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].text, "second");
    }

    #[test]
    fn ids_increment() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let a = store.add_decision("first", None, false).unwrap();
        let b = store.add_decision("second", None, false).unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
    }

    #[test]
    fn init_twice_fails() {
        let dir = tempfile::tempdir().unwrap();
        Store::init(dir.path()).unwrap();
        assert!(Store::init(dir.path()).is_err());
    }

    #[test]
    fn open_without_init_fails() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Store::open(dir.path()).is_err());
    }
}
