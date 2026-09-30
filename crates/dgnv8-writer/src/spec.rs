//! JSON job description for the `dgnv8w build` command.
//!
//! Coordinates are in master units (metres for ADTI), angles in degrees.
//! The job names a seed file; every element is appended to one of its models.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::document::{Document, WriteOptions};
use crate::element::{Element, Geometry, Symbology, TextData, FLAG_HOLE};
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
    #[serde(default)]
    pub defaults: Defaults,
    pub elements: Vec<ElementSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Defaults {
    pub level: u32,
    pub color: u32,
    pub weight: u32,
    pub style: u32,
    pub font: u32,
    pub text_height: f64,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            level: 64, // "Default" level ID in the evidence (FN-E04)
            color: 0,
            weight: 0,
            style: 0,
            font: 0,
            text_height: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElementSpec {
    #[serde(flatten)]
    pub geometry: GeometrySpec,
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub weight: Option<u32>,
    #[serde(default)]
    pub style: Option<u32>,
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
    TextNode {
        lines: Vec<String>,
        origin: [f64; 2],
        /// Distance between baselines; default 1.5 x height.
        #[serde(default)]
        line_pitch: Option<f64>,
        #[serde(default)]
        node_number: Option<u32>,
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
    for spec in &job.elements {
        let element = to_element(&document, &job.defaults, spec, Placement::IDENTITY)?;
        document.add(element);
    }
    let options = WriteOptions {
        modified_ms: job.modified_ms,
        update_file_header_counter: job.update_file_header_counter,
        allow_3d_model: job.allow_3d_model,
    };
    let output = base.join(&job.output);
    let report = document.save(&output, &options)?;
    Ok((output, report))
}

fn to_element(
    document: &Document,
    defaults: &Defaults,
    spec: &ElementSpec,
    placement: Placement,
) -> Result<Element> {
    // Cell headers default to level 0, as in both sample cells (FN-C01).
    let default_level = if matches!(spec.geometry, GeometrySpec::Cell { .. }) {
        0
    } else {
        defaults.level
    };
    let symbology = Symbology {
        level: spec.level.unwrap_or(default_level),
        color: spec.color.unwrap_or(defaults.color),
        weight: spec.weight.unwrap_or(defaults.weight),
        style: spec.style.unwrap_or(defaults.style),
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
            line_pitch,
            node_number,
            style,
        } => {
            let height = style.height.unwrap_or(defaults.text_height);
            let pitch = line_pitch.unwrap_or(1.5 * height);
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
            Geometry::TextNode {
                origin: point(*origin),
                rotation: angle(style.rotation_deg),
                node_number: node_number.unwrap_or(1),
                lines: node_lines,
            }
        }
        GeometrySpec::ComplexChain { parts } | GeometrySpec::ComplexShape { parts } => {
            // MicroStation draws complex elements with the components'
            // symbology (EXP-0002 T01), so parts inherit the parent's.
            let inherited = inherit(defaults, spec);
            let children = parts
                .iter()
                .map(|part| to_element(document, &inherited, part, placement))
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
                .map(|part| to_element(document, &inherited, part, inner))
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
        level: parent.level.unwrap_or(defaults.level),
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
            color: 3,
            ..Defaults::default()
        };
        to_element(&document, &defaults, &spec, Placement::IDENTITY).unwrap()
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
    }
}
