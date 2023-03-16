fmt: pyfmt
	cargo fmt
	cargo sort
	cargo clippy --bin gerb

.PHONY: check
check: tags
	cargo check --bin gerb

.PHONY: tags
tags:
	@which tagref > /dev/null && tagref || (printf "Warning: tagref binary not in PATH.\n" 1>&2)

.PHONY: pyfmt
pyfmt:
	@which black > /dev/null && (find src -name "*.py" | xargs black) || (printf "Warning: black binary not in PATH.\n" 1>&2)

.PHONY: feature-check
feature-check: check
	# No features
	@sh -c 'cargo check --bin gerb --no-default-features || (export EXIT="$$?"; /usr/bin/printf "--no-default-features fails cargo check.\n" && exit $$EXIT)'
	@cargo clippy --bin gerb --no-default-features
	# `git`
	@sh -c 'cargo check --bin gerb --no-default-features --features git || (export EXIT="$$?"; /usr/bin/printf "--features git fails cargo check.\n" && exit $$EXIT)'
	@cargo clippy --bin gerb --no-default-features --features git
	# `python`
	@sh -c 'cargo check --bin gerb --no-default-features --features python || (export EXIT="$$?"; /usr/bin/printf "--features python fails cargo check.\n" && exit $$EXIT)'
	@cargo clippy --bin gerb --no-default-features --features python
	# all features
	@sh -c 'cargo check --bin gerb --all-features || (export EXIT="$$?"; /usr/bin/printf "--all-features fails cargo check.\n" && exit $$EXIT)'
	@cargo clippy --bin gerb --all-features
