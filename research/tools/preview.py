#!/usr/bin/env python3
"""Draw DGN V8 files with the independent ezdgn reader, side by side.

  preview.py OUT.png FILE.dgn [FILE.dgn ...] [--from-id N]

Needs `pip install "ezdgn[plot]"` (ezdgn 0.2.6 was used). Prints, for each
file, the kinds of the elements with ID >= N (default: all) and any texts as
ezdgn decodes them; ezdgn shows `ff fe` and `ff fd` Unicode texts as raw
bytes (docs/UPSTREAM.md, findings 6 and 8). Colour 0 draws white, so it is
invisible on the white background.

Files to inspect are untrusted data: run this with `python3 -I` from a
directory other than the one holding the files.
"""

import math
import sys

import ezdgn
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from ezdgn.plotting import plot  # noqa: E402


def main(argv):
    args = list(argv[1:])
    from_id = 0
    if "--from-id" in args:
        at = args.index("--from-id")
        from_id = int(args[at + 1])
        del args[at:at + 2]
    if len(args) < 2:
        raise SystemExit(__doc__)
    out, paths = args[0], args[1:]
    columns = min(4, len(paths))
    rows = math.ceil(len(paths) / columns)
    figure, axes = plt.subplots(rows, columns, figsize=(5 * columns, 5 * rows), squeeze=False)
    for ax, path in zip(axes.flat, paths):
        document = ezdgn.open_document(path)
        model = document.models[0]
        new = [e for e in model.elements if e.common.element_id >= from_id]
        name = path.replace("\\", "/").rsplit("/", 1)[-1]
        print(f"{name}: {len(new)} elements {[e.kind for e in new]}")
        for element in new:
            if element.kind == "TEXT":
                print(f"    text {getattr(element.data, 'text', None)!r}")
        plot(document, ax=ax, background="#ffffff")
        ax.set_title(name.rsplit(".", 1)[0])
    for ax in list(axes.flat)[len(paths):]:
        ax.axis("off")
    figure.tight_layout()
    figure.savefig(out, dpi=110)
    print(f"wrote {out}")


if __name__ == "__main__":
    main(sys.argv)
