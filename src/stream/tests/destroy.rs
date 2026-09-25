//! [`LogEntry::is_destroy`] over each shape a channel's teardown is logged in.

use super::*;

fn only_entry(message: &str) -> LogEntry {
    let mut entries: Vec<_> =
        LogStream::new(vec![full_line(UUID1, TS1, message)].into_iter()).collect();
    assert_eq!(entries.len(), 1);
    entries.remove(0)
}

#[test]
fn a_channel_close_is_a_destroy() {
    let entry = only_entry("Close Channel sofia/internal/1263@192.0.2.1 [CS_DESTROY]");
    assert!(entry.is_destroy());
}

#[test]
fn a_channel_close_before_its_thread_ran_is_a_destroy() {
    let entry = only_entry("Close Channel sofia/internal/1263@192.0.2.1 [CS_NEW]");
    assert!(entry.is_destroy());
}

#[test]
fn a_state_change_into_cs_destroy_is_a_destroy() {
    let entry =
        only_entry("(sofia/internal/1263@192.0.2.1) State Change CS_REPORTING -> CS_DESTROY");
    assert!(matches!(
        entry.message_kind,
        MessageKind::StateChange { .. }
    ));
    assert!(entry.is_destroy());
}

#[test]
fn a_channel_name_holding_an_arrow_does_not_shift_the_target() {
    let entry = only_entry("(loopback/x -> y-a) State Change CS_REPORTING -> CS_DESTROY");
    assert!(entry.is_destroy());
}

#[test]
fn a_state_change_into_cs_execute_is_not_a_destroy() {
    let entry = only_entry("(sofia/internal/1263@192.0.2.1) State Change CS_ROUTING -> CS_EXECUTE");
    assert!(!entry.is_destroy());
}

#[test]
fn a_callstate_change_to_hangup_is_not_a_destroy() {
    let entry = only_entry("(sofia/internal/1263@192.0.2.1) Callstate Change ACTIVE -> HANGUP");
    assert!(!entry.is_destroy());
}

#[test]
fn the_running_state_change_is_not_matched() {
    let entry = only_entry(
        "(sofia/internal/1263@192.0.2.1) Running State Change CS_DESTROY (Cur 4 Tot 12)",
    );
    assert!(!entry.is_destroy());
}

#[test]
fn a_script_binding_releasing_its_session_is_not_a_destroy() {
    let entry = only_entry("sofia/internal/1263@192.0.2.1 destroy/unlink session from object");
    assert!(!entry.is_destroy());
}
