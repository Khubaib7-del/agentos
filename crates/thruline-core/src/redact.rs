//! Secret redaction, applied on every write into .thruline state
//! (security finding 3). Known credential formats are replaced with
//! `[redacted:<kind>]` tags before anything touches disk, so a pasted key
//! can't leak through a committed decision log or snapshot.

use regex::Regex;
use std::sync::LazyLock;

struct Rule {
    re: Regex,
    replacement: &'static str,
}

static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    let rule = |pattern: &str, replacement: &'static str| Rule {
        re: Regex::new(pattern).expect("static redaction pattern must compile"),
        replacement,
    };
    vec![
        // Private key blocks (PEM)
        rule(
            r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
            "[redacted:private-key]",
        ),
        // AWS access key id
        rule(r"\bAKIA[0-9A-Z]{16}\b", "[redacted:aws-key]"),
        // OpenAI/Anthropic-style keys
        rule(r"\bsk-[A-Za-z0-9_-]{16,}\b", "[redacted:api-key]"),
        // GitHub tokens
        rule(
            r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b",
            "[redacted:github-token]",
        ),
        rule(
            r"\bgithub_pat_[A-Za-z0-9_]{22,}\b",
            "[redacted:github-token]",
        ),
        // Slack tokens
        rule(
            r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b",
            "[redacted:slack-token]",
        ),
        // Google API keys
        rule(r"\bAIza[0-9A-Za-z_-]{35}\b", "[redacted:google-key]"),
        // JWTs
        rule(
            r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{5,}\b",
            "[redacted:jwt]",
        ),
        // Credentials embedded in URLs: keep scheme and user, drop password
        rule(
            r"\b([a-z][a-z0-9+.-]*://[^/\s:@]+:)[^@\s]+@",
            "${1}[redacted:password]@",
        ),
        // Bearer tokens in headers
        rule(
            r"(?i)\bbearer\s+[A-Za-z0-9_\-.=]{16,}",
            "Bearer [redacted:token]",
        ),
        // Generic assignments: api_key=..., password: "...", TOKEN = '...'
        rule(
            r#"(?i)\b(api[_-]?key|apikey|secret|token|password|passwd|pwd)(\s*[:=]\s*)["']?[A-Za-z0-9_\-./+]{8,}["']?"#,
            "${1}${2}[redacted:value]",
        ),
    ]
});

/// Replace anything that looks like a credential with a `[redacted:*]` tag.
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for rule in RULES.iter() {
        if let std::borrow::Cow::Owned(replaced) = rule.re.replace_all(&out, rule.replacement) {
            out = replaced;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn known_key_formats_are_redacted() {
        let cases = [
            ("aws AKIAIOSFODNN7EXAMPLE done", "aws-key"),
            ("key sk-abcdefghijklmnop1234 end", "api-key"),
            (
                "gh ghp_abcdefghijklmnopqrstuvwxyz0123456789 x",
                "github-token",
            ),
            ("slack xoxb-12345678901-abcdef end", "slack-token"),
            ("g AIzaSyA1234567890abcdefghijklmnopqrstuv x", "google-key"),
            (
                "jwt eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123def x",
                "jwt",
            ),
        ];
        for (input, kind) in cases {
            let out = redact(input);
            assert!(
                out.contains(&format!("[redacted:{kind}]")),
                "expected {kind} in {out:?}"
            );
        }
    }

    #[test]
    fn url_password_redacted_but_structure_kept() {
        let out = redact("db is postgres://admin:hunter2secret@db.example.com:5432/app");
        assert!(out.contains("postgres://admin:[redacted:password]@db.example.com"));
        assert!(!out.contains("hunter2secret"));
    }

    #[test]
    fn assignments_redact_value_keep_key() {
        let out = redact("set API_KEY=abc123def456xyz and continue");
        assert!(out.contains("API_KEY"));
        assert!(!out.contains("abc123def456xyz"));
    }

    #[test]
    fn private_key_block_redacted() {
        let out = redact(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEow\nsecret\n-----END RSA PRIVATE KEY-----",
        );
        assert_eq!(out, "[redacted:private-key]");
    }

    #[test]
    fn normal_engineering_text_is_untouched() {
        let cases = [
            "DB: PostgreSQL because the relational model fits",
            "Auth: Clerk — the team knows it",
            "use debounce on the search input",
            "password hashing should use bcrypt",
            "the token bucket algorithm fits rate limiting",
        ];
        for input in cases {
            assert_eq!(redact(input), input, "false positive on {input:?}");
        }
    }
}
