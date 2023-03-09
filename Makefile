fmt: pyfmt
	cargo fmt
	cargo sort
	cargo clippy --bin gerb

check: tags
	cargo check --bin gerb

.PHONY: tags
tags:
	@which tagref > /dev/null && tagref || (printf "Warning: tagref binary not in PATH.\n" 1>&2)

.PHONY: pyfmt
pyfmt:
	@which black > /dev/null && (find src -name "*.py" | xargs black) || (printf "Warning: black binary not in PATH.\n" 1>&2)
