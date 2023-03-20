.POSIX:
.SUFFIXES:
CARGOBIN	= cargo
TAGREFBIN = tagref
BLACKBIN = black
SASSCBIN = sassc
SASSCOPTS = -a -M -t compact
PRINTF = /usr/bin/printf

fmt: pyfmt
	$(CARGOBIN) fmt
	$(CARGOBIN) sort
	$(CARGOBIN) clippy --bin gerb

.PHONY: check
check: tags
	$(CARGOBIN) check --bin gerb

.PHONY: tags
tags:
	@which $(TAGREFBIN) > /dev/null && $(TAGREFBIN) || ($(PRINTF) "Warning: tagref binary not in PATH.\n" 1>&2)

.PHONY: pyfmt
pyfmt:
	@which $(BLACKBIN) > /dev/null && (find src -name "*.py" | xargs $(BLACKBIN)) || ($(PRINTF) "Warning: black binary not in PATH.\n" 1>&2)

.PHONY: feature-check
feature-check: check
	# No features
	@sh -c '$(CARGOBIN) check --bin gerb --no-default-features || (export EXIT="$$?"; $(PRINTF) "--no-default-features fails cargo check.\n" && exit $$EXIT)'
	@$(CARGOBIN) clippy --bin gerb --no-default-features
	# `git`
	@sh -c '$(CARGOBIN) check --bin gerb --no-default-features --features git || (export EXIT="$$?"; $(PRINTF) "--features git fails cargo check.\n" && exit $$EXIT)'
	@$(CARGOBIN) clippy --bin gerb --no-default-features --features git
	# `python`
	@sh -c '$(CARGOBIN) check --bin gerb --no-default-features --features python || (export EXIT="$$?"; $(PRINTF) "--features python fails cargo check.\n" && exit $$EXIT)'
	@$(CARGOBIN) clippy --bin gerb --no-default-features --features python
	# all features
	@sh -c '$(CARGOBIN) check --bin gerb --all-features || (export EXIT="$$?"; $(PRINTF) "--all-features fails cargo check.\n" && exit $$EXIT)'
	@$(CARGOBIN) clippy --bin gerb --all-features

src/themes/paperwhite/gtk.css: src/themes/paperwhite/*.scss
	$(SASSCBIN) $(SASSCOPTS) "src/themes/paperwhite/gtk.scss" "src/themes/paperwhite/gtk.css"

.PHONY:
themes: src/themes/paperwhite/gtk.css
