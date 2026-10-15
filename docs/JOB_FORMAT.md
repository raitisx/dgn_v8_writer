# Job format

A job is a JSON file that tells `dgnv8w build` which seed to start from,
where to write the result, and which elements to add. Units are the seed's
master units (metres for ADTI) and degrees. Paths are relative to the job
file.

```json
{
  "seed": "adti_seed.dgn",
  "output": "out.dgn",
  "levels": [ { "name": "MY_LEVEL", "description": "Mans līmenis" } ],
  "defaults": { "level": "MY_LEVEL", "color": "by_level", "font": 1025, "text_height": 0.25 },
  "elements": [
    { "type": "line", "points": [[506000, 312000], [506010, 312005]] },
    { "type": "text", "text": "Rīga", "origin": [506000, 311998] }
  ]
}
```

Run it with `dgnv8w build job.json`. The command prints `wrote <path>` and a
report: the page written, the number of objects, the first and last new
element IDs, the new model extents, and any levels it added.

Ready-made examples: [`examples/`](../examples/README.md). Jobs used for the
MicroStation tests: [`testfiles/jobs/`](../testfiles/jobs/) (GDAL sample)
and [`testfiles/adti_jobs/`](../testfiles/adti_jobs/) (ADTI seed).

## Top level

| Key | Default | Meaning |
| --- | --- | --- |
| `seed` | required | The seed DGN. It is read, never changed. |
| `output` | required | The DGN to write. Overwritten if it exists. |
| `elements` | required | The elements to add, in order (may be empty). |
| `defaults` | see below | Values used when an element does not give its own. |
| `levels` | none | Levels to add to the seed when it lacks them (FN-V03). Each is `{ "name": ..., "description": ... }`; the description is optional. A level the seed already has is reused. |
| `profile` | `"microstation"` | `"microstation"` writes bytes the way MicroStation does (EXP-0003); `"oda"` keeps the round-1 conventions of GDAL's ODA-written sample. Use `"microstation"` for ADTI work. |
| `model` | `0` | Which model to write into, by position in the seed's list of models (`dgnv8w inspect` lists them). |
| `modified_ms` | now | Last-modified time stamped on new elements, in milliseconds since 1970-01-01 UTC. Fix it to make the output byte-for-byte reproducible. |
| `allow_3d_model` | `false` | Allow 2D elements in a 3D model. Only for GDAL's 3D test sample. |
| `allow_unknown_levels` | `false` | Allow level IDs the seed's level table lacks. Only for experiments: MicroStation shows such elements with no level (FN-V02). |
| `update_file_header_counter` | `false` | Experiment H-H02, not needed by MicroStation. |

Unknown top-level keys are an error, so a misspelled key is caught. Inside
an element, unknown keys are **ignored**: check spelling there (`color`,
not `colour`).

## Defaults

| Key | Default | Meaning |
| --- | --- | --- |
| `level` | `64` | Level for elements without their own, by ID or by name. 64 is "Default". |
| `color` | `0` | Colour index, or `"by_level"`. |
| `weight` | `0` | Line weight, or `"by_level"`. |
| `style` | `0` | Line style number, or `"by_level"`. |
| `font` | `0` | Font number for texts. In the ADTI seed, 1025 is Arial. |
| `text_height` | `1.0` | Text height in master units. |

## Symbology on any element

Every element accepts these keys; each overrides the default.

| Key | Meaning |
| --- | --- |
| `level` | A level ID (number) or level name (text). Names are matched exactly against the seed's levels and the job's `levels`. An unknown name or ID is an error that lists the known levels. |
| `color` | Colour index in the seed's colour table (0–255 tested), or `"by_level"` |
| `weight` | `0`–`31`, or `"by_level"` |
| `style` | Line style number, or `"by_level"` |
| `graphic_group` | Graphic group number (default 0) |

`"by_level"` stores MicroStation's ByLevel values (FN-E14): the element then
takes the colour, weight or style of its level.

Parts of complex chains, complex shapes and cells inherit their parent's
`level`, `color`, `weight` and `style` unless they give their own.
MicroStation draws complex elements with their parts' symbology (FN-E07),
so set symbology on the parent or on the parts, not only on a header.

## Element types

Points are `[x, y]` in master units: x is easting, y northing, as
MicroStation shows them (EXP-0004 R05).

### line

| Key | Meaning |
| --- | --- |
| `points` | Exactly two points |

### line_string

| Key | Meaning |
| --- | --- |
| `points` | Two or more points |

### shape

| Key | Meaning |
| --- | --- |
| `points` | Three or more corners. The first point is repeated at the end if it is not already there. |
| `hole` | `true` marks the shape as a hole (default `false`) |

### curve

| Key | Meaning |
| --- | --- |
| `points` | Six or more points. MicroStation draws the curve through the inner points; the first two and last two control its ends (FN-E08). |

### circle

| Key | Meaning |
| --- | --- |
| `center` | Centre point |
| `radius` | Radius |

### ellipse

| Key | Meaning |
| --- | --- |
| `center` | Centre point |
| `primary`, `secondary` | Semi-axes |
| `rotation_deg` | Angle of the primary axis (default 0) |

### arc

| Key | Meaning |
| --- | --- |
| `center` | Centre point |
| `primary` | Radius, or primary semi-axis |
| `secondary` | Secondary semi-axis (default: `primary`, a circular arc) |
| `rotation_deg` | Angle of the primary axis (default 0) |
| `start_deg` | Start angle, measured from the primary axis |
| `sweep_deg` | Sweep; negative runs clockwise. Not zero. |

### text

| Key | Default | Meaning |
| --- | --- | --- |
| `text` | required | The text. Latvian letters are fine. |
| `origin` | required | Lower-left of the text, whatever the justification (FN-T03) |
| `height` | `defaults.text_height` | Text height |
| `width` | `height` | Character width |
| `rotation_deg` | 0 | Angle |
| `font` | `defaults.font` | Font number from the seed's font table |
| `justification` | 0 | 0 left top, 1 left middle, 2 left bottom (baseline), 6 centre top, 7 centre centre, 8 centre bottom, 12 right top, 13 right centre, 14 right bottom (FN-T06). MicroStation computes the justification point from the font; the text is drawn at `origin`. |
| `encoding` | `"auto"` | How the text is stored; see below |

Encodings: `"auto"` stores ASCII as is and anything else as MicroStation
does (`ff fe` + UTF-16, FN-T07). The others are for tests: `"ascii"`,
`"unicode"` (always `ff fe` + UTF-16), `"utf16"` (`ff fd` + UTF-16, FN-T05),
`"escaped-cp1252"` (FN-T02), and `"escaped-cp1257"` (a rejected experiment,
H-T03).

### text_node

A text of several lines that MicroStation edits as one element.

| Key | Default | Meaning |
| --- | --- | --- |
| `lines` | required | The lines, top first |
| `origin` | required | Lower-left of the first line |
| `line_spacing` | 0.5 × height | Gap between lines, as in MicroStation; baselines are height + spacing apart |
| `line_pitch` | | Distance between baselines, instead of `line_spacing` (not both) |
| `node_number` | 1 | Node number |
| `linkages` | by profile | `"line_breaks"` (default for `"microstation"`) keeps the lines apart when the node is edited in MicroStation (FN-N03); `"none"` (default for `"oda"`) and `"full"` are for tests |

It also takes `height`, `width`, `rotation_deg`, `font`, `justification`
and `encoding`, as a text does; they apply to every line. With the
`"microstation"` profile, `justification` must be 0, 1 or 2: the writer
places the node's origin from them (FN-N04), and other justifications
would need line lengths that only MicroStation knows.

### complex_chain and complex_shape

| Key | Meaning |
| --- | --- |
| `parts` | Elements of type `line`, `line_string`, `arc` or `curve`, each starting where the previous one ends. A complex shape's last part should end at the first part's start. The writer does not check that the parts connect. |

### cell

A normal (not shared) cell, which is how ADTI symbols are placed.

| Key | Default | Meaning |
| --- | --- | --- |
| `name` | none | Cell name (optional) |
| `origin` | required | Insertion point |
| `scale` | 1 | Scale |
| `rotation_deg` | 0 | Rotation |
| `parts` | required | Elements in the cell's own coordinates around `origin`: `line`, `line_string`, `shape`, `curve`, `circle`, `ellipse`, `arc` or `text` |

The cell header is always on level 0, as in MicroStation's own cells; a
`level` given on the cell goes to its parts (FN-C01). Cells cannot be
nested yet, and cannot contain complex elements or text nodes.

## Levels

Use level names, not numbers. A name is looked up in the seed's level
table first (`dgnv8w inspect seed.dgn` lists it), then in the job's
`levels`. To add a level, list it:

```json
"levels": [
  { "name": "ZZ_TEST_A", "description": "Pārbaudes līmenis A" },
  { "name": "ZZ_TEST_B" }
]
```

A new level gets the lowest free level ID from 1 upwards and the same
settings as the seed's "Default" level, including Default's ByLevel colour,
weight and style (FN-V03). Names are case-sensitive, but a new name that
differs from an existing one only in letter case is refused, to avoid
near-duplicates. Latvian letters are fine (FN-V04).

## What the writer does not do

It does not change or delete anything already in the seed, apart from the
model extents, the level table when the job adds levels, and the graphic
page it appends to. It does not write 3D elements, B-splines, dimensions,
shared cells, tags or other non-graphic data. See [SCOPE.md](SCOPE.md).
