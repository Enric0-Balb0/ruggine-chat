// Integration test: ensure the show_invites_modal empty state text is the expected one
#[cfg(test)]
mod invite_empty_message_test {
    use std::fs;

    #[test]
    fn empty_invites_shows_received_text() {
        // read the component source and check the empty-state string
        let path = "src/components/modals/show_invites_modal.rs";
        let content = fs::read_to_string(path).expect("failed to read show_invites_modal.rs");
        // the UI should always show 'Nessun invito ricevuto' when there are no invites
        assert!(content.contains("Nessun invito ricevuto"), "Expected empty-state text not found in {}", path);
    }
}
