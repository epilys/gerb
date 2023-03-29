#!/bin/bash

for m in README.md CONTRIBUTING.md CODE_OF_CONDUCT.md DEVELOPMENT.md examples/README.md; do
  aspell --mode markdown --dont-suggest --ignore-case --personal <(
cat << EOF
personal_ws-1.1 en 0
config
ctrl
foss
FontForge
Glyphs
Homebrew
MFEK
OpenType
PRs
readme
toml
TrueType
UFOv
ui
xdg
amd
backtrace
br
cd
cli
dconf
designspaces
dev
gerb
gerb's
github
github's
gtk
href
img
indo
io
kbd
libgtk
librsvg
macos
png
prebuilt
proto
px
runebender
readline
repl
sexualized
socio
src
svg
themeable
thinsp
toolkit
toolkit's
ufo
undos
EOF
) list < $m
done
