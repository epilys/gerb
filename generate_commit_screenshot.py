#!/usr/bin/env python3
import argparse, pathlib
from ctypes import c_void_p, c_char_p

from wand.image import Image
from wand.api import library


def generate_image_url(blob: bytes, output_width: int, lossless: bool) -> str:
    # Tell python about the MagickSetOption method
    library.MagickSetOption.argtypes = [
        c_void_p,  # MagickWand * wand
        c_char_p,  # const char * option
        c_char_p,
    ]  # const char * value
    with Image(blob=blob) as i:
        if lossless:
            # -define webp:lossless=true
            library.MagickSetOption(i.wand, b"webp:lossless", b"true")
        with i.convert("webp") as page:
            page.alpha_channel = False
            width = page.width
            height = page.height
            ratio = output_width / (width * 1.0)
            new_height = int(ratio * height)
            page.thumbnail(width=output_width, height=new_height)
            return page.data_url().replace("x-webp", "webp")


def main(image_path: pathlib.Path, width: int, lossless: bool) -> str:
    with open(image_path, "rb") as f:
        return generate_image_url(f.read(), width, lossless)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="sum the integers at the command line")
    parser.add_argument("file", type=pathlib.Path, help="an integer to be summed")
    parser.add_argument("-w", "--width", default=100, type=int)
    parser.add_argument("--lossless", default=False, action="store_true")
    args = parser.parse_args()
    print(main(args.file, args.width, args.lossless))
