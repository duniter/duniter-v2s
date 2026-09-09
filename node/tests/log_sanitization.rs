//! RUSTSEC-2025-0055 regression through Duniter's actual SDK logger.

#[test]
fn untrusted_log_messages_escape_terminal_controls() {
    const CHILD_ENV: &str = "DUNITER_LOG_SANITIZATION_CHILD";
    if std::env::var_os(CHILD_ENV).is_some() {
        sc_cli::LoggerBuilder::new("info").init().unwrap();
        for input in [
            "before\x1b[2Jafter",
            "before\x07after",
            "before\u{009b}2Jafter",
        ] {
            log::info!("untrusted-log-input {input}");
        }
        return;
    }

    // Logger initialization is global, so isolate it from other tests and log filters.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "untrusted_log_messages_escape_terminal_controls",
            "--nocapture",
        ])
        .env(CHILD_ENV, "1")
        .env_remove("RUST_LOG")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines: Vec<_> = stderr
        .lines()
        .filter(|line| line.contains("untrusted-log-input"))
        .collect();
    assert_eq!(lines.len(), 3, "{stderr}");
    for line in lines {
        assert!(line.contains("before"), "{line}");
        assert!(line.contains("after"), "{line}");
        assert!(
            !line
                .chars()
                .any(|c| c == '\x1b' || c == '\x07' || c == '\u{009b}'),
            "terminal control was not escaped: {line:?}"
        );
    }
}
