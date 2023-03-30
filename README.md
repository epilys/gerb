# gerb [![License]][gpl3]&nbsp;[![Build Status]][actions]&nbsp;[![Latest Version]][crates.io] [![netbsd]][pkgsrc.se]&nbsp;[![aur]][aur-url]

[gpl3]: https://github.com/epilys/gerb/blob/main/LICENSE
[Build Status]: https://img.shields.io/github/actions/workflow/status/epilys/gerb/builds.yaml?branch=main
[actions]: https://github.com/epilys/gerb/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/gerb.svg?color=white
[crates.io]: https://crates.io/crates/gerb
[Top Language]: https://img.shields.io/github/languages/top/epilys/gerb?color=white&logo=rust&logoColor=black
[License]: https://img.shields.io/github/license/epilys/gerb?color=white
[pkgsrc.se]: https://pkgsrc.se/fonts/gerb
[netbsd]: https://img.shields.io/badge/netbsd-pkgsrc%2Ffonts%2Fgerb-%23777777?labelColor=%23ea6410
[aur-url]: https://aur.archlinux.org/packages/gerb-git
[aur]: https://img.shields.io/badge/aur-gerb--git-%23555555?labelColor=%23ecf2f5

1. _*gerb ʰ-_: [reconstructed Proto-Indo-European root](https://en.wiktionary.org/wiki/Reconstruction:Proto-Indo-European/gerb%CA%B0-), meaning _to carve_
2. `gerb`: a font editor in gtk3 and Rust

<p align="center">
<a href="./screenshot-small.png?raw=true">
<img alt="Editing a glyph inside gerb." src="./screenshot-small.png?raw=true" width="450" height="429" style="object-fit: scale-down; height: auto; max-width: 450px;">
</a><br />
<kbd><strong>Editing a glyph.</strong></kbd>
</p>

<sup><sub>&#x261B; <em>See the <strong><a href="#screenshots">Screenshots</a></strong> section.</em></sub></sup>

`gerb` is a GUI font editor and IDE.
It is not production ready, but all the basics are implemented.

It uses the [_UFOv3_](https://unifiedfontobject.org/versions/ufo3/index.html) format and can import:[^0]

- _UFOv2_ directories
- _Glyphs_ files

and export:[^1]

- OpenType files (`.otf`)
- TrueType files (`.ttf`)

[^0]: Import is performed with [`fontTools`](https://github.com/fonttools/fonttools) and [`glyphsLib`](https://github.com/googlefonts/glyphsLib).
[^1]: Export is performed with [`ufo2ft`](https://github.com/googlefonts/ufo2ft).

| ℹ️  Interested in contributing? Consult [`CONTRIBUTING.md`](./CONTRIBUTING.md). |
| ---                                                                            |

## Features

- import from other font source formats
- export to `.otf` and `.ttf`
- configurable shortcuts system
- unlimited undos
- embedded python shell and API for scripting and plugins (work in progress)
- git integration (work in progress)
- themeable (work in progress)

### Future features

- [ ] work with designspaces ([tracking issue #22](https://github.com/epilys/gerb/issues/22))

## Screenshots [^2]

[^2]: The screenshot typeface is [Sporting Grotesque](https://www.velvetyne.fr/fonts/sporting-grotesque/).

<p align="center" width="100%">
<a href="./screenshot.png?raw=true"><img width="49%" src="./screenshot.png?raw=true"></a>
<a href="./screenshot2.png?raw=true"><img width="49%" src="./screenshot2.png?raw=true"></a>
</p>

## Alternative software

- [FontForge](https://fontforge.org) is the indisputable FOSS font editor.
  Realistically this is the only choice one has for making a professional quality typeface with free software.

Some other projects in development are:

- [runebender](https://github.com/linebender/runebender): development seems to have slowed down (as of Wed 15 Mar 2023).
  Unfortunately, the project —while excellent— looks like it is encumbered by its UI toolkit's development.
  In contrast, **gerb uses the standard FOSS UI toolkit, `gtk`**.
- [MFEK](https://github.com/MFEK): it's focused on splitting every functionality into micro-libraries.
  **gerb's technical goal is shipping a font editor**.

## Install

### Packages

Packages are available:

- NetBSD <https://pkgsrc.se/fonts/gerb>
- Debian / Ubuntu as `.deb` files included with each [release](https://github.com/epilys/gerb/releases)
- Arch Linux <https://aur.archlinux.org/packages/gerb-git>

It is also installable from [crates.io](https://crates.io/crates/gerb): ```cargo install gerb```

### Prebuilt GNU/Linux & macos amd64 binaries

See ['Releases'](https://github.com/epilys/gerb/releases) for binaries of tagged releases built in the CI.

## Build

To build, you will need Rust's `cargo` tool.
You can get it from your distribution's packages, or directly with the official [`rustup`](https://rustup.rs) tool.
If the build fails because of missing system libraries, see the [Dependencies](#dependencies) section of the `README`.

Download or clone the git repository with your method of choice, e.g.:

```shell
git clone https://github.com/epilys/gerb.git
cd gerb
cargo build --release
```

### Dependencies

Needs `gtk-3`.
For the `python` feature you'll need `libpython3.9` or greater.

On Debian and relatives:

```shell
apt install libgtk-3-dev
```

On `macOS` you can install dependencies with `Homebrew`:

```shell
brew install librsvg gtk+3 gnome-icon-theme
```

## Run & Configuration

You can open a UFOv3 project from the GUI (&thinsp;*File->Open* or <kbd>Ctrl+O</kbd>&thinsp;) or directly in the command line with the `-u` flag.
Assuming the project directory is "/path/to/font.ufo":

```shell
# Directly calling the binary:
gerb -u /path/to/font.ufo
# Running through cargo
cargo run --release -- -u /path/to/font.ufo
```

Configuration of various settings is stored at the `$XDG_CONFIG_HOME/gerb` directory in a TOML file.
The usual location would be `$HOME/.config/gerb/config.toml`.
[**dconf**](https://en.wikipedia.org/wiki/Dconf) is not used but PRs that add dconf support are welcome.
