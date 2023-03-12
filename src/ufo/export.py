import tempfile
import shutil
from defcon import Font
from ufo2ft import compileOTF, compileTTF


def make_output_path(ufo, options):
    ext = None

    if options.output_path and len(options.output_path) != 0:
        return options.output_path

    if options.format == OutputFormat.Otf:
        ext = "otf"
    elif options.format == OutputFormat.Ttf:
        ext = "ttf"
    else:
        raise ValueError(f"Got unrecognized output format option: {options.format}")

    if not options.filename_stem:
        return f"{options.output_dir}/{ufo.info.familyName}-{ufo.info.styleName}.{ext}"

    return f"{options.output_dir}/{options.filename_stem}.{ext}"


def export(options) -> str:
    """Compiles a UFO project to OTF/TTF."""
    with tempfile.TemporaryDirectory() as tmpdirname:
        shutil.copytree(options.input_dir, tmpdirname, dirs_exist_ok=True)
        ufo = Font(tmpdirname)
        result = make_output_path(ufo, options)
        if options.format == OutputFormat.Otf:
            otf = compileOTF(ufo)
            otf.save(result)
        elif options.format == OutputFormat.Ttf:
            ttf = compileTTF(ufo)
            ttf.save(result)
        else:
            raise ValueError(f"Got unrecognized output format option: {options.format}")
        return result
