#!/bin/sh

export OUTPUT=$(dirname "$1")/$(basename "$1" svg)png

rsvg-convert -w 24 -a -o "$OUTPUT" "$1"
