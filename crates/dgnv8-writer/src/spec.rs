//! JSON job description for the `dgnv8w build` command.
//!
//! Coordinates are in master units (metres for ADTI), angles in degrees.
//! The job names a seed file; every element is appended to one of its models.
//! Levels are given by ID or by name from the seed's level table; levels
//! listed under `levels` are added to the seed when it lacks them. Colour,
//! weight and line style are given by number or as `"by_level"`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::document::{Document, WriteOptions};
use crate::element::{
    Element, Geometry, NodeLinkages, Profile, Symbology, TextData, TextNodeStyle, COLOR_BY_LEVEL,
    FLAG_HOLE, STYLE_BY_LEVEL, WEIGHT_BY_LEVEL,
};
use crate::error::{Result, WriteError};
use crate::geom::Point2;
use crate::text::TextEncoding;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    /// Seed DGN, relative to the job file.
    pub seed: PathBuf,
    /// Output DGN, relative to the job file.
    pub output: PathBuf,
    #[serde(default)]
    pub model: Option<usize>,
    /// Fixed last-modified stamp (ms since 1970) for reproducible output.
    #[serde(default)]
    pub modified_ms: Option<f64>,
    /// EXPERIMENT H-H02: write the highest new element ID into `Dgn~H`.
    #[serde(default)]
    pub update_file_header_counter: bool,
    /// Test fixtures only: allow 2D elements in a 3D model.
    #[serde(default)]
    pub allow_3d_model: bool,
    /// `"microstation"` (default) or `"oda"` (round-1 conventions).
    #[serde(default)]
    pub profile: Profile,
    /// Experiments only: allow level IDs that the seed's level table lacks
    /// (FN-V02).
    #[serde(default)]
    pub allow_unknown_levels: bool,
    /// Levels to add to the seed when it lacks them (FN-V03).
    #[serde(default)]
    pub levels: Vec<LevelDecl>,
    #[serde(default)]
    pub defaults: Defaults,
    pub elements: Vec<ElementSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Defaults {
    pub level: LevelSpec,
    pub color: AttrSpec,
    pub weight: AttrSpec,
    pub style: AttrSpec,
    pub font: u32,
    pub text_height: f64,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            level: LevelSpec::Id(64), // "Default" level ID in the evidence (FN-E02)
            color: AttrSpec::Value(0),
            weight: AttrSpec::Value(0),
            style: AttrSpec::Value(0),
            font: 0,
            text_height: 1.0,
        }
    }
}

/// A level the job wants in the file (FN-V03).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelDecl {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// A level by ID, or by exact name from the seed's level table (FN-V01).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum LevelSpec {
    Id(u32),
    Name(String),
}

/// A colour, weight or line style: a number or `"by_level"` (FN-E14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum AttrSpec {
    Value(u32),
    Keyword(AttrKeyword),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttrKeyword {
    ByLevel,
}

impl AttrSpec {
    const fn resolve(self, by_level: u32) -> u32 {
        match self {
            Self::Value(value) => value,
            Self::Keyword(AttrKeyword::ByLevel) => by_level,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElementSpec {
    #[serde(flatten)]
    pub geometry: GeometrySpec,
    #[serde(default)]
    pub level: Option<LevelSpec>,
    #[serde(default)]
    pub color: Option<AttrSpec>,
    #[serde(default)]
    pub weight: Option<AttrSpec>,
    #[serde(default)]
    pub style: Option<AttrSpec>,
    #[serde(default)]
    pub graphic_group: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeometrySpec {
    Line {
        points: [[f64; 2]; 2],
    },
    LineString {
        points: Vec<[f64; 2]>,
    },
    /// Closed automatically when the last point differs from the first.
    Shape {
        points: Vec<[f64; 2]>,
        #[serde(default)]
        hole: bool,
    },
    Curve {
        points: Vec<[f64; 2]>,
    },
    Circle {
        center: [f64; 2],
        radius: f64,
    },
    Ellipse {
        center: [f64; 2],
        primary: f64,
        secondary: f64,
        #[serde(default)]
        rotation_deg: f64,
    },
    Arc {
        center: [f64; 2],
        primary: f64,
        #[serde(default)]
        secondary: Option<f64>,
        #[serde(default)]
        rotation_deg: f64,
        start_deg: f64,
        sweep_deg: f64,
    },
    Text {
        text: String,
        origin: [f64; 2],
        #[serde(flatten)]
        style: TextStyleSpec,
    },
    /// `origin` is the first line's lower-left, as for a text.
    TextNode {
        lines: Vec<String>,
        origin: [f64; 2],
        /// Gap between lines (MicroStation's line spacing); default 0.5 x
        /// height, so baselines are 1.5 x height apart.
        #[serde(default)]
        line_spacing: Option<f64>,
        /// Distance between baselines; alternative to `line_spacing`.
        #[serde(default)]
        line_pitch: Option<f64>,
        #[serde(default)]
        node_number: Option<u32>,
        /// `"none"`, `"line_breaks"` or `"full"`; default by profile (FN-N03).
        #[serde(default)]
        linkages: Option<NodeLinkages>,
        #[serde(flatten)]
        style: TextStyleSpec,
    },
    ComplexChain {
        parts: Vec<ElementSpec>,
    },
    ComplexShape {
        parts: Vec<ElementSpec>,
    },
    /// Parts are given in cell-local coordinates relative to `origin`.
    Cell {
        #[serde(default)]
        name: Option<String>,
        origin: [f64; 2],
        #[serde(default = "one")]
        scale: f64,
        #[serde(default)]
        rotation_deg: f64,
        parts: Vec<ElementSpec>,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TextStyleSpec {
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default)]
    pub font: Option<u32>,
    #[serde(default)]
    pub justification: Option<u16>,
    #[serde(default)]
    pub encoding: TextEncoding,
}

fn one() -> f64 {
    1.0
}

/// Similarity transform used to place cell parts: model = origin + R*s*local.
#[derive(Debug, Clone, Copy)]
struct Placement {
    origin: [f64; 2],
    scale: f64,
    rotation: f64,
}

impl Placement {
    const IDENTITY: Self = Self {
        origin: [0.0, 0.0],
        scale: 1.0,
        rotation: 0.0,
    };

    fn apply(&self, point: [f64; 2]) -> [f64; 2] {
        let (sin_r, cos_r) = self.rotation.sin_cos();
        let (x, y) = (point[0] * self.scale, point[1] * self.scale);
        [
            self.origin[0] + x * cos_r - y * sin_r,
            self.origin[1] + x * sin_r + y * cos_r,
        ]
    }
}

/// Load a job file, build the document and save it. Returns the output path.
pub fn run_job(job_path: &Path) -> Result<(PathBuf, crate::document::SaveReport)> {
    let text = std::fs::read_to_string(job_path)?;
    let job: JobSpec = serde_json::from_str(&text)
        .map_err(|error| WriteError::InvalidInput(format!("{}: {error}", job_path.display())))?;
    let base = job_path.parent().unwrap_or_else(|| Path::new("."));
    let mut document = Document::from_seed_path(base.join(&job.seed), job.model)?;
    for level in &job.levels {
        document.ensure_level(&level.name, level.description.as_deref())?;
    }
    let mut elements = Vec::with_capacity(job.elements.len());
    let target = Target {
        document: &document,
        profile: job.profile,
        allow_unknown_levels: job.allow_unknown_levels,
    };
    for spec in &job.elements {
        elements.push(to_element(
            &target,
            &job.defaults,
            spec,
            Placement::IDENTITY,
        )?);
    }
    for element in elements {
        document.add(element);
    }
    let options = WriteOptions {
        modified_ms: job.modified_ms,
        update_file_header_counter: job.update_file_header_counter,
        allow_3d_model: job.allow_3d_model,
        profile: job.profile,
    };
    let output = base.join(&job.output);
    let report = document.save(&output, &options)?;
    Ok((output, report))
}

/// The document being built and the job-wide switches.
struct Target<'a> {
    document: &'a Document,
    profile: Profile,
    allow_unknown_levels: bool,
}

impl Target<'_> {
    /// Level ID for `level`, looking names up in the seed (FN-V01). An ID
    /// must be in the seed's level table (FN-V02), except 0, the cell
    /// header level (FN-C01).
    fn resolve_level(&self, level: &LevelSpec) -> Result<u32> {
        let document = self.document;
        let known = || {
            let names: Vec<String> = document
                .levels()
                .map(|level| format!("{} = {}", level.name, level.id))
                .collect();
            names.join(", ")
        };
        match level {
            LevelSpec::Id(id) => {
                if *id == 0
                    || self.allow_unknown_levels
                    || document.levels().any(|level| level.id == *id)
                {
                    Ok(*id)
                } else {
                    Err(WriteError::InvalidInput(format!(
                        "level ID {id} is not in the seed's level table (it has: {})",
                        known()
                    )))
                }
            }
            LevelSpec::Name(name) => document.level_id(name).ok_or_else(|| {
                WriteError::InvalidInput(format!(
                    "level {name:?} is not in the seed's level table (it has: {}); \
                     list it under \"levels\" to add it",
                    known()
                ))
            }),
        }
    }
}

fn to_element(
    target: &Target,
    defaults: &Defaults,
    spec: &ElementSpec,
    placement: Placement,
) -> Result<Element> {
    let (document, profile) = (target.document, target.profile);
    // A cell header is on level 0, as in all three sample cells and in
    // MicroStation's own cell (FN-C01); a cell's "level" goes to its parts.
    let level = if matches!(spec.geometry, GeometrySpec::Cell { .. }) {
        0
    } else {
        target.resolve_level(spec.level.as_ref().unwrap_or(&defaults.level))?
    };
    let symbology = Symbology {
        level,
        color: spec.color.unwrap_or(defaults.color).resolve(COLOR_BY_LEVEL),
        weight: spec
            .weight
            .unwrap_or(defaults.weight)
            .resolve(WEIGHT_BY_LEVEL),
        style: spec.style.unwrap_or(defaults.style).resolve(STYLE_BY_LEVEL),
        graphic_group: spec.graphic_group.unwrap_or(0),
    };
    let point = |p: [f64; 2]| {
        let placed = placement.apply(p);
        document.master_to_uor(placed[0], placed[1])
    };
    let points = |list: &[[f64; 2]]| list.iter().map(|p| point(*p)).collect::<Vec<Point2>>();
    let distance = |d: f64| document.distance_to_uor(d * placement.scale);
    let angle = |degrees: f64| degrees.to_radians() + placement.rotation;
    let mut extra_flags = 0;

    let geometry = match &spec.geometry {
        GeometrySpec::Line { points: [a, b] } => Geometry::Line {
            start: point(*a),
            end: point(*b),
        },
        GeometrySpec::LineString { points: list } => Geometry::LineString(points(list)),
        GeometrySpec::Shape { points: list, hole } => {
            let mut closed = points(list);
            if closed.first() != closed.last() {
                closed.push(closed[0]);
            }
            if *hole {
                extra_flags |= FLAG_HOLE;
            }
            Geometry::Shape(closed)
        }
        GeometrySpec::Curve { points: list } => Geometry::Curve(points(list)),
        GeometrySpec::Circle { center, radius } => Geometry::Ellipse {
            center: point(*center),
            primary: distance(*radius),
            secondary: distance(*radius),
            rotation: placement.rotation,
        },
        GeometrySpec::Ellipse {
            center,
            primary,
            secondary,
            rotation_deg,
        } => Geometry::Ellipse {
            center: point(*center),
            primary: distance(*primary),
            secondary: distance(*secondary),
            rotation: angle(*rotation_deg),
        },
        GeometrySpec::Arc {
            center,
            primary,
            secondary,
            rotation_deg,
            start_deg,
            sweep_deg,
        } => Geometry::Arc {
            center: point(*center),
            primary: distance(*primary),
            secondary: distance(secondary.unwrap_or(*primary)),
            rotation: angle(*rotation_deg),
            start: start_deg.to_radians(),
            sweep: sweep_deg.to_radians(),
        },
        GeometrySpec::Text {
            text,
            origin,
            style,
        } => Geometry::Text(text_data(
            defaults,
            style,
            text,
            point(*origin),
            &distance,
            &angle,
        )),
        GeometrySpec::TextNode {
            lines,
            origin,
            line_spacing,
            line_pitch,
            node_number,
            linkages,
            style,
        } => {
            let height = style.height.unwrap_or(defaults.text_height);
            let spacing = match (line_spacing, line_pitch) {
                (Some(_), Some(_)) => {
                    return Err(WriteError::InvalidInput(
                        "give line_spacing or line_pitch, not both".into(),
                    ))
                }
                (Some(spacing), None) => *spacing,
                (None, Some(pitch)) => pitch - height,
                (None, None) => 0.5 * height,
            };
            let pitch = height + spacing;
            let rotation = style.rotation_deg.to_radians();
            let (sin_r, cos_r) = rotation.sin_cos();
            let node_lines = lines
                .iter()
                .enumerate()
                .map(|(index, line)| {
                    let down = pitch * index as f64;
                    let local = [origin[0] + down * sin_r, origin[1] - down * cos_r];
                    Ok((
                        symbology,
                        text_data(defaults, style, line, point(local), &distance, &angle),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            let justification = style.justification.unwrap_or(0);
            let node_origin = match profile {
                Profile::Oda => *origin, // first line's origin (FN-N01)
                Profile::MicroStation => {
                    // Justification point of the node box (FN-N02, FN-N04).
                    let block = lines.len() as f64 * pitch - spacing;
                    let up = match justification {
                        0 => height,                      // left top
                        1 => height - 0.5 * block,        // left centre
                        2 => height - block,              // left bottom
                        _ => {
                            return Err(WriteError::InvalidInput(format!(
                                "text node justification {justification} is not supported yet; use 0, 1 or 2"
                            )))
                        }
                    };
                    [origin[0] - up * sin_r, origin[1] + up * cos_r]
                }
            };
            let width = style.width.unwrap_or(height);
            Geometry::TextNode {
                origin: point(node_origin),
                rotation: angle(style.rotation_deg),
                node_number: node_number.unwrap_or(1),
                style: TextNodeStyle {
                    font: style.font.unwrap_or(defaults.font),
                    justification,
                    line_spacing: distance(spacing),
                    width: distance(width),
                    height: distance(height),
                },
                lines: node_lines,
                linkages: linkages.unwrap_or(NodeLinkages::default_for(profile)),
            }
        }
        GeometrySpec::ComplexChain { parts } | GeometrySpec::ComplexShape { parts } => {
            // MicroStation draws complex elements with the components'
            // symbology (EXP-0002 T01), so parts inherit the parent's.
            let inherited = inherit(defaults, spec);
            let children = parts
                .iter()
                .map(|part| to_element(target, &inherited, part, placement))
                .collect::<Result<Vec<_>>>()?;
            if matches!(spec.geometry, GeometrySpec::ComplexChain { .. }) {
                Geometry::ComplexChain(children)
            } else {
                Geometry::ComplexShape(children)
            }
        }
        GeometrySpec::Cell {
            name,
            origin,
            scale,
            rotation_deg,
            parts,
        } => {
            if placement.rotation != 0.0 || placement.scale != 1.0 || placement.origin != [0.0, 0.0]
            {
                return Err(WriteError::InvalidInput(
                    "nested cells are not supported".into(),
                ));
            }
            let inner = Placement {
                origin: *origin,
                scale: *scale,
                rotation: rotation_deg.to_radians(),
            };
            let inherited = inherit(defaults, spec);
            let children = parts
                .iter()
                .map(|part| to_element(target, &inherited, part, inner))
                .collect::<Result<Vec<_>>>()?;
            let (sin_r, cos_r) = inner.rotation.sin_cos();
            Geometry::Cell {
                name: name.clone(),
                origin: point(*origin),
                transform: [scale * cos_r, -scale * sin_r, scale * sin_r, scale * cos_r],
                children,
            }
        }
    };
    Ok(Element {
        symbology,
        geometry,
        extra_flags,
    })
}

/// Defaults for the parts of a complex element or cell: the parent's
/// explicit symbology, else the job defaults. A cell's own default level 0
/// is not passed on.
fn inherit(defaults: &Defaults, parent: &ElementSpec) -> Defaults {
    Defaults {
        level: parent
            .level
            .clone()
            .unwrap_or_else(|| defaults.level.clone()),
        color: parent.color.unwrap_or(defaults.color),
        weight: parent.weight.unwrap_or(defaults.weight),
        style: parent.style.unwrap_or(defaults.style),
        ..defaults.clone()
    }
}

fn text_data(
    defaults: &Defaults,
    style: &TextStyleSpec,
    text: &str,
    origin: Point2,
    distance: &dyn Fn(f64) -> f64,
    angle: &dyn Fn(f64) -> f64,
) -> TextData {
    let height = style.height.unwrap_or(defaults.text_height);
    let width = style.width.unwrap_or(height);
    TextData {
        text: text.to_owned(),
        encoding: style.encoding,
        origin,
        rotation: angle(style.rotation_deg),
        height: distance(height),
        width: distance(width),
        font: style.font.unwrap_or(defaults.font),
        justification: style.justification.unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &[u8] = include_bytes!("../tests/data/gdal_test_dgnv8.dgn");

    fn build(json: &str) -> Element {
        let document = Document::from_seed_bytes(SEED.to_vec(), None).unwrap();
        let spec: ElementSpec = serde_json::from_str(json).unwrap();
        let defaults = Defaults {
            color: AttrSpec::Value(3),
            ..Defaults::default()
        };
        let target = Target {
            document: &document,
            profile: Profile::Oda,
            allow_unknown_levels: false,
        };
        to_element(&target, &defaults, &spec, Placement::IDENTITY).unwrap()
    }

    fn part_colors(element: &Element) -> Vec<u32> {
        match &element.geometry {
            Geometry::ComplexShape(parts)
            | Geometry::Cell {
                children: parts, ..
            } => parts.iter().map(|part| part.symbology.color).collect(),
            other => panic!("unexpected {other:?}"),
        }
    }

    fn build_with(profile: Profile, json: &str) -> Result<Element> {
        let document = Document::from_seed_bytes(SEED.to_vec(), None).unwrap();
        let spec: ElementSpec = serde_json::from_str(json).unwrap();
        let target = Target {
            document: &document,
            profile,
            allow_unknown_levels: false,
        };
        to_element(&target, &Defaults::default(), &spec, Placement::IDENTITY)
    }

    #[test]
    fn level_names_and_by_level_resolve() {
        let line = build(
            r#"{ "type": "line", "points": [[0, 0], [1, 0]], "level": "Default",
                 "color": "by_level", "weight": "by_level", "style": "by_level" }"#,
        );
        assert_eq!(
            line.symbology,
            Symbology {
                level: 64,
                color: COLOR_BY_LEVEL,
                weight: WEIGHT_BY_LEVEL,
                style: STYLE_BY_LEVEL,
                graphic_group: 0,
            }
        );
        let missing = build_with(
            Profile::Oda,
            r#"{ "type": "line", "points": [[0, 0], [1, 0]], "level": "NO_SUCH_LEVEL" }"#,
        );
        let message = missing.unwrap_err().to_string();
        assert!(message.contains("Default = 64") && message.contains("\"levels\""));
        // R04b: an ID the level table lacks is refused (FN-V02).
        let unknown = build_with(
            Profile::Oda,
            r#"{ "type": "line", "points": [[0, 0], [1, 0]], "level": 1 }"#,
        );
        assert!(unknown.is_err());
    }

    #[test]
    fn microstation_text_node_origin_is_its_justification_point() {
        // E4: a left-top node stores its origin one text height above the
        // first baseline; lines are height + spacing apart (FN-N02).
        let json = r#"{ "type": "text_node", "lines": ["a", "b"], "origin": [10, 20],
                        "height": 0.25, "line_spacing": 0.5, "font": 1025 }"#;
        let node = build_with(Profile::MicroStation, json).unwrap();
        let Geometry::TextNode {
            origin,
            style,
            lines,
            linkages,
            ..
        } = &node.geometry
        else {
            panic!("not a text node");
        };
        // FN-N03: line-break linkages unless the job says otherwise.
        assert_eq!(*linkages, NodeLinkages::LineBreaks);
        assert_eq!(*origin, Point2::new(100_000.0, 202_500.0));
        assert_eq!(lines[1].1.origin, Point2::new(100_000.0, 192_500.0));
        assert_eq!(
            (style.font, style.line_spacing, style.height),
            (1025, 5000.0, 2500.0)
        );
        let left_bottom = json.replace("\"font\"", "\"justification\": 2, \"font\"");
        let node = build_with(Profile::MicroStation, &left_bottom).unwrap();
        let Geometry::TextNode { origin, .. } = node.geometry else {
            panic!("not a text node");
        };
        assert_eq!(origin, Point2::new(100_000.0, 192_500.0));
        let centred = json.replace("\"font\"", "\"justification\": 7, \"font\"");
        assert!(build_with(Profile::MicroStation, &centred).is_err());
        // The Oda profile keeps the first line's origin (FN-N01) and writes
        // no linkages.
        let node = build_with(Profile::Oda, json).unwrap();
        let Geometry::TextNode {
            origin, linkages, ..
        } = node.geometry
        else {
            panic!("not a text node");
        };
        assert_eq!(origin, Point2::new(100_000.0, 200_000.0));
        assert_eq!(linkages, NodeLinkages::None);
    }

    #[test]
    fn every_job_file_in_the_repository_parses() {
        // Keeps the documented examples and test jobs in step with the format.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut count = 0;
        for dir in ["testfiles/jobs", "testfiles/adti_jobs", "examples"] {
            for entry in std::fs::read_dir(root.join(dir)).unwrap() {
                let path = entry.unwrap().path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    let text = std::fs::read_to_string(&path).unwrap();
                    if let Err(error) = serde_json::from_str::<JobSpec>(&text) {
                        panic!("{}: {error}", path.display());
                    }
                    count += 1;
                }
            }
        }
        assert!(count >= 27, "found only {count} job files");
    }

    #[test]
    fn complex_parts_inherit_parent_symbology() {
        let shape = build(
            r#"{ "type": "complex_shape", "color": 4, "weight": 2, "parts": [
                { "type": "line", "points": [[0, 0], [1, 0]] },
                { "type": "line", "points": [[1, 0], [0, 0]], "color": 6 } ] }"#,
        );
        assert_eq!(shape.symbology.color, 4);
        assert_eq!(part_colors(&shape), [4, 6]);
    }

    #[test]
    fn cell_header_level_zero_is_not_inherited() {
        let cell = build(
            r#"{ "type": "cell", "origin": [5, 5], "color": 2, "parts": [
                { "type": "line", "points": [[0, 0], [1, 0]] } ] }"#,
        );
        assert_eq!(cell.symbology.level, 0);
        assert_eq!(part_colors(&cell), [2]);
        match &cell.geometry {
            Geometry::Cell { children, .. } => assert_eq!(children[0].symbology.level, 64),
            other => panic!("unexpected {other:?}"),
        }
        // A cell's level goes to its parts; the header stays on 0 (FN-C01).
        let cell = build(
            r#"{ "type": "cell", "origin": [5, 5], "level": "Default", "parts": [
                { "type": "line", "points": [[0, 0], [1, 0]] } ] }"#,
        );
        assert_eq!(cell.symbology.level, 0);
        match &cell.geometry {
            Geometry::Cell { children, .. } => assert_eq!(children[0].symbology.level, 64),
            other => panic!("unexpected {other:?}"),
        }
    }
}
