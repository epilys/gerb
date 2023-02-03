import glyphsLib
import os
import sys


def glyphs2ufo(options):
    """Converts a Glyphs.app source file into UFO masters and a designspace file."""
    if options.output_dir is None:
        options.output_dir = os.path.dirname(options.glyphs_file) or "."

    if options.designspace_path is None:
        options.designspace_path = os.path.join(
            options.output_dir,
            os.path.basename(os.path.splitext(options.glyphs_file)[0]) + ".designspace",
        )

    # If options.instance_dir is None, instance UFO paths in the designspace
    # file will either use the value in customParameter's UFO_FILENAME_CUSTOM_PARAM or
    # be made relative to "instance_ufos/".
    masters = glyphsLib.build_masters(
        options.glyphs_file,
        options.output_dir,
        options.instance_dir,
        designspace_path=options.designspace_path,
        minimize_glyphs_diffs=options.no_preserve_glyphsapp_metadata,
        propagate_anchors=options.propagate_anchors,
        normalize_ufos=options.normalize_ufos,
        create_background_layers=options.create_background_layers,
        generate_GDEF=options.generate_GDEF,
        store_editor_state=not options.no_store_editor_state,
        write_skipexportglyphs=options.write_public_skip_export_glyphs,
        ufo_module=__import__(options.ufo_module),
        minimal=options.minimal,
        glyph_data=options.glyph_data or None,
    ).ufos
    return list(
        zip(
            masters.keys(),
            map(lambda inst: inst.path, masters.values()),
            map(lambda inst: inst.info.familyName, masters.values()),
            map(lambda inst: inst.info.styleName, masters.values()),
        )
    )
