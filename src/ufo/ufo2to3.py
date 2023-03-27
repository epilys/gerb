from fontTools.ufoLib import UFOReader, UFOWriter
from fontTools.ufoLib.converters import convertUFO1OrUFO2KerningToUFO3Kerning
from fontTools.pens.pointPen import SegmentToPointPen
import os
from copy import deepcopy
import tempfile
import shutil


def ufo2to3(options):
    """Converts a UFOv2 directory to a UFOv3 one."""
    ufo2_path = options.input_dir
    ufo3_path = options.output_dir
    if os.path.exists(ufo3_path) and not os.listdir(ufo3_path):
        os.rmdir(ufo3_path)
    with tempfile.TemporaryDirectory() as tmpdirname:
        shutil.copytree(ufo2_path, tmpdirname, dirs_exist_ok=True)
        reader = UFOReader(tmpdirname)
        writer = UFOWriter(
            ufo3_path, formatVersion=(3, 0), fileCreator="io.github.epilys.gerb"
        )
        writer.writeKerning(deepcopy(reader.readKerning()))
        writer.writeGroups(deepcopy(reader.readGroups()))
        in_glyph_set = reader.getGlyphSet()
        out_glyph_set = writer.getGlyphSet(expectContentsFile=False)
        for glyph_name in in_glyph_set.keys():
            glyph = in_glyph_set[glyph_name]
            in_glyph_set.readGlyph(glyph_name, glyph)

            def draw_points_func(pen):
                pen = SegmentToPointPen(pen)
                glyph.draw(pen)

            out_glyph_set.writeGlyph(glyph_name, glyph, draw_points_func)

        out_glyph_set.writeContents()

        class Info:
            pass

        infoobj = Info()
        info = reader.readInfo(infoobj)
        writer.writeInfo(infoobj)
        lib = reader.readLib()
        writer.writeLib(lib)
        writer.writeFeatures(deepcopy(reader.readFeatures()))
        writer.writeLayerContents(deepcopy(reader.getLayerNames()), validate=False)
        for fname in reader.getDataDirectoryListing():
            writer.copyFromReader(reader, fname, "data" / fname)

        for fname in reader.getImageDirectoryListing():
            writer.copyFromReader(reader, fname, "images" / fname)

        writer.setModificationTime()
        writer.close()
        return (
            ufo3_path,
            os.path.abspath(ufo3_path),
            infoobj.familyName,
            infoobj.styleName,
        )
