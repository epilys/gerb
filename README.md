# gerb

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

It uses the [UFOv3](https://unifiedfontobject.org/versions/ufo3/index.html) format and can import:

- UFOv2 directories
- Glyphs files

Integrated export to `{ttf, otf}` files is planned by using [`ufo2ft`](https://github.com/googlefonts/ufo2ft).

| :information_source: Interested in contributing? Consult [`CONTRIBUTING.md`](./CONTRIBUTING.md).|
| ---                                                                                             |

## Build & Run

To build, you will need Rust's `cargo` tool.
You can get it from your distribution's packages, or directly with the official [`rustup`](https://rustup.rs) tool.
If the build fails because of missing system libraries, see the [Dependencies](#dependencies) section of the `README`.

Download or clone the git repository with your method of choice, e.g.:

```shell
git clone https://github.com/epilys/gerb.git
cd gerb
cargo build --release
```

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

## Features

- mechanism for import from other font formats
- configurable shortcuts system
- unlimited undos
- embedded python shell and API for scripting and plugins (work in progress)
- git integration (work in progress)
- themeable (work in progress)

### Future features

- [ ] work with designspaces ([tracking issue #22](https://github.com/epilys/gerb/issues/22))

## Screenshots

<sup><sub>the screenshot typeface is [Sporting Grotesque](https://www.velvetyne.fr/fonts/sporting-grotesque/).</sub></sup>

<p align="center" width="100%">
<a href="./screenshot.png?raw=true"><img width="49%" src="./screenshot.png?raw=true"></a>
<a href="./screenshot2.png?raw=true"><img width="49%" src="./screenshot2.png?raw=true"></a>
</p>

### Dependencies

Needs `gtk-3`.

On Debian and relatives:

```shell
apt install libgtk-3-dev
```

On `macOS` you can install dependencies with `Homebrew`:

```shell
brew install librsvg gtk+3 gnome-icon-theme
```
