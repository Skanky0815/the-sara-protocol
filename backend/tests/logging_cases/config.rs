#[test]
fn configuration_debug_redacts_connection_credentials() {
    let config = the_sara_protocol::config::Config {
        host: "127.0.0.1".parse().unwrap(),
        port: 3000,
        database_url: "postgres://user-secret:password-secret@localhost/db".into(),
    };
    let debug = format!("{config:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("secret"));
}
