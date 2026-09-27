#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../src/dto/mod.rs"]
mod dto;
#[allow(dead_code)]
#[path = "../src/error.rs"]
mod error;

#[test]
fn absent_error_details_are_omitted_to_match_the_typescript_contract() {
    let value = serde_json::to_value(error::AppError::generation_busy()).unwrap();
    assert_eq!(value, serde_json::json!({ "code": "GENERATION_BUSY" }));
    let value =
        serde_json::to_value(error::AppError::from(dtt_application::Error::Cancelled)).unwrap();
    assert_eq!(value, serde_json::json!({ "code": "GENERATION_CANCELLED" }));
}

#[test]
fn path_failures_retain_machine_readable_context_without_a_message_field() {
    let value = serde_json::to_value(error::AppError::from(dtt_application::Error::Settings(
        dtt_application::SettingsIssue::InvalidPath {
            field: "launcher_db",
            path: "synthetic.db".into(),
        },
    )))
    .unwrap();
    assert_eq!(value["code"], "INVALID_PATH");
    assert_eq!(value["context"]["path"], "synthetic.db");
    assert!(value.get("message").is_none());
    assert!(value["technicalDetail"].is_string());
}
