lint:
	cargo clippy --tests --all-features -- -Wclippy::pedantic -Aclippy::missing_errors_doc -Aclippy::must_use_candidate
