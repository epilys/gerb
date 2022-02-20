# `gerb`

1. _*gerb ʰ-_: [reconstructed Proto-Indo-European root](https://en.wiktionary.org/wiki/Reconstruction:Proto-Indo-European/gerb%CA%B0-), meaning _to carve_
2. `gerb`: a WIP font editor in `gtk3` and `rust`, ignore for now

## Roadmap to a minimum working prototype

- [ ] save modifications to disk
- [ ] be able to create new paths/contours in a glyph ([Tracking issue #3](https://github.com/epilys/gerb/issues/3))
- [ ] be able to add/remove glyphs
- [ ] create new projects

## Running the demo

Expects a folder "font.ufo" to be defined in the command line:

```shell
cargo run -- -u ./font.ufo
```

I got mine from the `Regular` instance at the [Source Sans repository](https://github.com/adobe-fonts/source-sans).

![./screenshot.png](./screenshot.png)

![./screenshot2.png](./screenshot2.png)
