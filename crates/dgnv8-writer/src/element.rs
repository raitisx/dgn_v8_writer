//! Byte encoders for the 2D element types in scope.
//!
//! Every graphical object starts with the 0x68-byte common header
//! (FN-E01..FN-E07); the type-specific primary data follows, then optional
//! linkages (FN-L01). All coordinates here are already in UOR.

use crate::error::{Result, WriteError};
use crate::geom::{arc_bounds, ellipse_bounds, Bounds, Point2, RangeI64};
use crate::text::{encode_text, TextEncoding};

pub const TYPE_CELL: u8 = 2;
pub const TYPE_LINE: u8 = 3;
pub const TYPE_LINE_STRING: u8 = 4;
pub const TYPE_SHAPE: u8 = 6;
pub const TYPE_TEXT_NODE: u8 = 7;
pub const TYPE_CURVE: u8 = 11;
pub const TYPE_COMPLEX_CHAIN: u8 = 12;
pub const TYPE_COMPLEX_SHAPE: u8 = 14;
pub const TYPE_ELLIPSE: u8 = 15;
pub const TYPE_ARC: u8 = 16;
pub const TYPE_TEXT: u8 = 17;

/// Size of the common graphical header (FN-E01).
pub const COMMON_HEADER_BYTES: usize = 0x68;
/// Properties word seen on every graphical object of the fixture (FN-E05).
pub const DEFAULT_PROPERTIES: u32 = 0x8000_0000;
/// Type-word bit on every object MicroStation itself writes (FN-E13).
pub const TYPE_WORD_MICROSTATION_BIT: u32 = 0x0080_0000;
/// Line style, weight and colour values that mean "ByLevel" (FN-E14).
pub const STYLE_BY_LEVEL: u32 = 0x7fff_ffff;
pub const WEIGHT_BY_LEVEL: u32 = 0xffff_ffff;
pub const COLOR_BY_LEVEL: u32 = 0xffff_ffff;
/// "New" property: MicroStation shows New when set (FN-E06). The GDAL
/// sample has it on some objects; this writer never sets it by default.
pub const FLAG_NEW: u32 = 0x0000_0200;
/// Flag bit for a 3D object (FN-E06). Never set by this 2D writer.
pub const FLAG_3D: u32 = 0x0000_0800;
/// Hole shape, for example inside a grouped hole (FN-E06).
pub const FLAG_HOLE: u32 = 0x0000_8000;
/// Text size multiplier: UOR = raw * 6 / 1000 (FN-T01). Written as
/// `uor * (1000 / 6)`, which reproduces the fixture's bits exactly.
pub const UOR_TO_TEXT_MULTIPLIER: f64 = 1000.0 / 6.0;
/// Property/string linkage identifier (FN-L02).
pub const STRING_LINKAGE_ID: u16 = 0x56d2;
/// Linkage MicroStation attaches to every text and text node (H-N04), as
/// written in EXP-0003 E4. Byte 8 is `0x80` on node lines that end with a
/// line break, else 0.
pub const TEXT_RUN_LINKAGE: [u8; 24] = [
    0x0b, 0x10, 0xd4, 0x80, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x20, 0x30, 0xe9, 0x04, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
/// Offset of the line-break byte in [`TEXT_RUN_LINKAGE`] (H-N04).
pub const TEXT_RUN_LINE_BREAK_AT: usize = 8;
/// Second linkage on every MicroStation text and text node, holding f64 1.0
/// (H-N04).
pub const TEXT_SCALE_LINKAGE: [u8; 24] = [
    0x0b, 0x10, 0xf3, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Position of an object inside a complex element (FN-E01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Standalone,
    ComplexHeader,
    Component,
}

impl Role {
    #[must_use]
    pub const fn bits(self) -> u32 {
        match self {
            Self::Standalone => 0x1000_0000,
            Self::ComplexHeader => 0x3000_0000,
            Self::Component => 0x5000_0000,
        }
    }
}

/// Display attributes written into the common header (FN-E04, FN-E11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Symbology {
    pub level: u32,
    pub color: u32,
    pub weight: u32,
    pub style: u32,
    pub graphic_group: u32,
}

/// Text fields for a type-17 element.
#[derive(Debug, Clone, PartialEq)]
pub struct TextData {
    pub text: String,
    pub encoding: TextEncoding,
    pub origin: Point2,
    pub rotation: f64,
    pub height: f64,
    pub width: f64,
    pub font: u32,
    /// Raw justification code, V7 table: 0 Left Top, 1 Left Center, 2 Left
    /// Bottom, 7 Center Center, 12 Right Top, 14 Right Bottom (FN-T06). `origin` is always the
    /// lower-left of the text, whatever the justification.
    pub justification: u16,
}

/// EXPERIMENT H-N04: linkages MicroStation adds to text nodes, which may
/// tell it where one line ends when the node is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeLinkages {
    /// No linkages, as in round 1 and round 2.
    #[default]
    None,
    /// The `0x80d4` linkage on each line, with the line-break byte set on
    /// every line but the last.
    LineBreaks,
    /// Both linkages on the node header and on every line, exactly as in E4.
    Full,
}

/// Node-wide text settings that MicroStation stores in the text node header
/// (FN-N02). Sizes and spacing in UOR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextNodeStyle {
    pub font: u32,
    pub justification: u16,
    /// Gap between one line's box and the next; baseline pitch is
    /// `height + line_spacing`.
    pub line_spacing: f64,
    pub width: f64,
    pub height: f64,
}

/// A 2D element in UOR with its symbology.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub symbology: Symbology,
    pub geometry: Geometry,
    /// Extra bits OR-ed into the flags word (for example [`FLAG_HOLE`]).
    pub extra_flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Line {
        start: Point2,
        end: Point2,
    },
    LineString(Vec<Point2>),
    Shape(Vec<Point2>),
    Curve(Vec<Point2>),
    Ellipse {
        center: Point2,
        primary: f64,
        secondary: f64,
        rotation: f64,
    },
    Arc {
        center: Point2,
        primary: f64,
        secondary: f64,
        rotation: f64,
        start: f64,
        sweep: f64,
    },
    Text(TextData),
    TextNode {
        /// Node origin as stored: the first line's origin for the `Oda`
        /// profile (FN-N01), the node's justification point for
        /// `MicroStation` (FN-N02).
        origin: Point2,
        rotation: f64,
        node_number: u32,
        /// Written only by the `MicroStation` profile; `Oda` writes zeros.
        style: TextNodeStyle,
        lines: Vec<(Symbology, TextData)>,
        /// EXPERIMENT H-N04.
        linkages: NodeLinkages,
    },
    ComplexChain(Vec<Element>),
    ComplexShape(Vec<Element>),
    Cell {
        /// Cell name, written as a string linkage (FN-C02); `None` writes an
        /// unnamed cell.
        name: Option<String>,
        origin: Point2,
        /// Row-major 2x2 matrix (FN-C01 only confirms the identity).
        transform: [f64; 4],
        children: Vec<Element>,
    },
}

/// Whose byte conventions to follow where writers differ.
///
/// `Oda` reproduces the GDAL sample (written by the ODA library) and the
/// round-1 test files. `MicroStation` follows what MicroStation itself writes
/// in the ADTI seed (EXP-0003): FN-E13, FN-P05, FN-T04, FN-N02, FN-C03.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Oda,
    /// The default since round 2 (EXP-0004).
    #[default]
    #[serde(rename = "microstation")]
    MicroStation,
}

/// Settings shared by every object written in one save.
#[derive(Debug, Clone, Copy)]
pub struct EncodeContext {
    /// Last-modified time, milliseconds since 1970-01-01 UTC (FN-E03a).
    pub modified_ms: f64,
    pub profile: Profile,
}

impl EncodeContext {
    /// Type-word bits and properties word for this profile (FN-E05, FN-E13).
    const fn header_style(&self) -> (u32, u32) {
        match self.profile {
            Profile::Oda => (0, DEFAULT_PROPERTIES),
            Profile::MicroStation => (TYPE_WORD_MICROSTATION_BIT, 0),
        }
    }
}

/// Hands out element IDs above everything already in the seed (FN-S02).
#[derive(Debug, Clone)]
pub struct IdAllocator {
    next: u64,
}

impl IdAllocator {
    #[must_use]
    pub const fn starting_at(next: u64) -> Self {
        Self { next }
    }

    pub fn allocate(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }

    #[must_use]
    pub const fn last_allocated(&self) -> u64 {
        self.next - 1
    }
}

/// Encoded objects of one top-level element, in page order.
#[derive(Debug, Clone)]
pub struct Encoded {
    pub objects: Vec<Vec<u8>>,
    pub range: RangeI64,
}

impl Element {
    /// Encode this element (and its components) as page objects.
    pub fn encode(&self, ids: &mut IdAllocator, context: &EncodeContext) -> Result<Encoded> {
        self.encode_with_role(Role::Standalone, ids, context, &[])
    }

    /// `extra_linkages` are appended after this object's own linkages.
    fn encode_with_role(
        &self,
        role: Role,
        ids: &mut IdAllocator,
        context: &EncodeContext,
        extra_linkages: &[u8],
    ) -> Result<Encoded> {
        let (type_bits, properties) = context.header_style();
        let simple = |element_type: u8, flags: u32, primary: Vec<u8>, bounds: Bounds| {
            let range = RangeI64::from_bounds(bounds);
            let header = HeaderFields {
                element_type,
                role,
                type_bits,
                symbology: self.symbology,
                element_id: 0,
                modified_ms: context.modified_ms,
                properties,
                flags: flags | self.extra_flags,
                range,
            };
            (header, primary, range)
        };
        let profile = context.profile;
        let (header, primary, range, linkages, components) = match &self.geometry {
            Geometry::Line { start, end } => {
                // FN-E09: start and end point.
                let mut primary = Vec::with_capacity(32);
                push_point(&mut primary, *start);
                push_point(&mut primary, *end);
                let bounds = Bounds::of_points(&[*start, *end]).unwrap();
                let (h, p, r) = simple(TYPE_LINE, 0, primary, bounds);
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::LineString(points) => {
                check_vertices("line string", points, 2)?;
                let (h, p, r) = simple(
                    TYPE_LINE_STRING,
                    0,
                    vertex_list(points),
                    Bounds::of_points(points).unwrap(),
                );
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Shape(points) => {
                check_vertices("shape", points, 4)?;
                if points.first() != points.last() {
                    return Err(WriteError::InvalidElement(
                        "shape must repeat its first vertex at the end".into(),
                    ));
                }
                let (h, p, r) = simple(
                    TYPE_SHAPE,
                    0,
                    vertex_list(points),
                    Bounds::of_points(points).unwrap(),
                );
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Curve(points) => {
                check_vertices("curve", points, 6)?;
                let (h, p, r) = simple(
                    TYPE_CURVE,
                    0,
                    vertex_list(points),
                    Bounds::of_points(points).unwrap(),
                );
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Ellipse {
                center,
                primary,
                secondary,
                rotation,
            } => {
                check_axes(*primary, *secondary)?;
                let mut data = Vec::with_capacity(40);
                push_f64(&mut data, *primary);
                push_f64(&mut data, *secondary);
                push_f64(&mut data, *rotation);
                push_point(&mut data, *center);
                let bounds = ellipse_bounds(*center, *primary, *secondary, *rotation);
                let (h, p, r) = simple(TYPE_ELLIPSE, 0, data, bounds);
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Arc {
                center,
                primary,
                secondary,
                rotation,
                start,
                sweep,
            } => {
                check_axes(*primary, *secondary)?;
                if *sweep == 0.0 || !sweep.is_finite() {
                    return Err(WriteError::InvalidElement(
                        "arc sweep must be non-zero".into(),
                    ));
                }
                let mut data = Vec::with_capacity(56);
                push_f64(&mut data, *start);
                push_f64(&mut data, *sweep);
                push_f64(&mut data, *primary);
                push_f64(&mut data, *secondary);
                push_f64(&mut data, *rotation);
                push_point(&mut data, *center);
                let bounds = arc_bounds(*center, *primary, *secondary, *rotation, *start, *sweep);
                let (h, p, r) = simple(TYPE_ARC, 0, data, bounds);
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Text(text) => {
                let (data, bounds) = text_primary(text, profile)?;
                let (h, p, r) = simple(TYPE_TEXT, 0, data, bounds);
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::TextNode {
                origin,
                rotation,
                node_number,
                style,
                lines,
                linkages,
            } => {
                if lines.is_empty() {
                    return Err(WriteError::InvalidElement(
                        "text node needs at least one line".into(),
                    ));
                }
                let children: Vec<Element> = lines
                    .iter()
                    .map(|(symbology, text)| Element {
                        symbology: *symbology,
                        geometry: Geometry::Text(text.clone()),
                        extra_flags: 0,
                    })
                    .collect::<Vec<_>>();
                let mut bounds = Bounds::of_point(*origin);
                for child in &children {
                    bounds = bounds.union(child.bounds_for(profile)?);
                }
                // FN-N01: line count, node number, rotation, origin.
                let mut data = Vec::with_capacity(0x40);
                push_u32(&mut data, lines.len() as u32);
                push_u32(&mut data, *node_number);
                match profile {
                    Profile::Oda => data.extend_from_slice(&[0u8; 0x20]), // as the sample (FN-N01)
                    Profile::MicroStation => {
                        // FN-N02: font, 255, justification, spacing, size.
                        push_u32(&mut data, style.font);
                        data.extend_from_slice(&255u16.to_le_bytes());
                        data.extend_from_slice(&style.justification.to_le_bytes());
                        push_f64(&mut data, style.line_spacing);
                        push_f64(&mut data, style.width * UOR_TO_TEXT_MULTIPLIER);
                        push_f64(&mut data, style.height * UOR_TO_TEXT_MULTIPLIER);
                    }
                }
                push_f64(&mut data, *rotation);
                push_point(&mut data, *origin);
                let (h, p, r) = simple(TYPE_TEXT_NODE, 0, data, bounds);
                let h = HeaderFields {
                    role: header_role(role),
                    ..h
                };
                // EXPERIMENT H-N04: MicroStation's text linkages.
                let last = children.len() - 1;
                let line_linkages = |index: usize| {
                    let mut run = TEXT_RUN_LINKAGE.to_vec();
                    if index != last {
                        run[TEXT_RUN_LINE_BREAK_AT] = 0x80;
                    }
                    match linkages {
                        NodeLinkages::None => Vec::new(),
                        NodeLinkages::LineBreaks => run,
                        NodeLinkages::Full => [run, TEXT_SCALE_LINKAGE.to_vec()].concat(),
                    }
                };
                let header_linkages = match linkages {
                    NodeLinkages::Full => [TEXT_RUN_LINKAGE, TEXT_SCALE_LINKAGE].concat(),
                    _ => Vec::new(),
                };
                let components = children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| (child, line_linkages(index)))
                    .collect();
                (h, p, r, header_linkages, components)
            }
            Geometry::ComplexChain(children) | Geometry::ComplexShape(children) => {
                let (element_type, what) = if matches!(self.geometry, Geometry::ComplexChain(_)) {
                    (TYPE_COMPLEX_CHAIN, "complex chain")
                } else {
                    (TYPE_COMPLEX_SHAPE, "complex shape")
                };
                check_components(what, children, false)?;
                let mut bounds: Option<Bounds> = None;
                for child in children {
                    let child_bounds = child.bounds_for(profile)?;
                    bounds = Some(bounds.map_or(child_bounds, |b| b.union(child_bounds)));
                }
                // FN-X01: component count and a zero word.
                let mut data = Vec::with_capacity(8);
                push_u32(&mut data, children.len() as u32);
                push_u32(&mut data, 0);
                let (h, p, r) = simple(element_type, 0, data, bounds.unwrap());
                let h = HeaderFields {
                    role: header_role(role),
                    ..h
                };
                (h, p, r, Vec::new(), without_linkages(children))
            }
            Geometry::Cell {
                name,
                origin,
                transform,
                children,
            } => {
                check_components("cell", children, true)?;
                let mut bounds: Option<Bounds> = None;
                for child in children {
                    let child_bounds = child.bounds_for(profile)?;
                    bounds = Some(bounds.map_or(child_bounds, |b| b.union(child_bounds)));
                }
                let bounds = bounds.unwrap();
                let mut data = Vec::with_capacity(0x58);
                // Components cannot nest yet, so this equals the number of
                // descendants that MicroStation stores (FN-C03).
                push_u32(&mut data, children.len() as u32);
                let local = match profile {
                    Profile::Oda => {
                        push_u32(&mut data, 1); // 0x6c: value 1 observed (FN-C01)
                        bounds // model coordinates, as in round 1 (FN-C01)
                    }
                    Profile::MicroStation => {
                        data.extend_from_slice(&[1, 0, 1, 0]); // 0x6c (FN-C03)
                        cell_local_bounds(bounds, *origin, transform)? // FN-C03
                    }
                };
                push_point(&mut data, local.min); // range low (FN-C01)
                push_point(&mut data, local.max); // range high
                for value in transform {
                    push_f64(&mut data, *value);
                }
                push_point(&mut data, *origin);
                let linkages = match (name, profile) {
                    (None, _) => Vec::new(),
                    (Some(name), Profile::MicroStation) if name.is_ascii() => {
                        string_linkage_bytes(1, name.as_bytes()) // FN-C03
                    }
                    (Some(name), _) => string_linkage(1, name), // FN-C02
                };
                let (h, p, r) = simple(TYPE_CELL, 0, data, bounds);
                let h = HeaderFields {
                    role: header_role(role),
                    ..h
                };
                (h, p, r, linkages, without_linkages(children))
            }
        };

        let mut header = header;
        header.element_id = ids.allocate();
        let linkages = [linkages.as_slice(), extra_linkages].concat();
        let mut objects = vec![encode_object(&header, &primary, &linkages)?];
        for (child, child_linkages) in &components {
            let encoded = child.encode_with_role(Role::Component, ids, context, child_linkages)?;
            objects.extend(encoded.objects);
        }
        Ok(Encoded { objects, range })
    }

    /// Floating bounds of this element in UOR, with the round-1 text box.
    pub fn bounds(&self) -> Result<Bounds> {
        self.bounds_for(Profile::Oda)
    }

    /// Floating bounds of this element in UOR; the text box depends on the
    /// profile (FN-T04).
    pub fn bounds_for(&self, profile: Profile) -> Result<Bounds> {
        Ok(match &self.geometry {
            Geometry::Line { start, end } => Bounds::of_points(&[*start, *end]).unwrap(),
            Geometry::LineString(points) | Geometry::Shape(points) | Geometry::Curve(points) => {
                Bounds::of_points(points)
                    .ok_or_else(|| WriteError::InvalidElement("element has no vertices".into()))?
            }
            Geometry::Ellipse {
                center,
                primary,
                secondary,
                rotation,
            } => ellipse_bounds(*center, *primary, *secondary, *rotation),
            Geometry::Arc {
                center,
                primary,
                secondary,
                rotation,
                start,
                sweep,
            } => arc_bounds(*center, *primary, *secondary, *rotation, *start, *sweep),
            Geometry::Text(text) => text_bounds(text, profile),
            Geometry::TextNode { origin, lines, .. } => {
                let mut bounds = Bounds::of_point(*origin);
                for (_, line) in lines {
                    bounds = bounds.union(text_bounds(line, profile));
                }
                bounds
            }
            Geometry::ComplexChain(children)
            | Geometry::ComplexShape(children)
            | Geometry::Cell { children, .. } => {
                let mut bounds: Option<Bounds> = None;
                for child in children {
                    let child_bounds = child.bounds_for(profile)?;
                    bounds = Some(bounds.map_or(child_bounds, |b| b.union(child_bounds)));
                }
                bounds
                    .ok_or_else(|| WriteError::InvalidElement("complex element is empty".into()))?
            }
        })
    }
}

fn without_linkages(children: &[Element]) -> Vec<(Element, Vec<u8>)> {
    children
        .iter()
        .map(|child| (child.clone(), Vec::new()))
        .collect()
}

/// Header of a complex element keeps the header role even when nested.
fn header_role(_outer: Role) -> Role {
    Role::ComplexHeader
}

/// Fields of the 0x68-byte common header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaderFields {
    pub element_type: u8,
    pub role: Role,
    /// Extra type-word bits, for example [`TYPE_WORD_MICROSTATION_BIT`].
    pub type_bits: u32,
    pub symbology: Symbology,
    pub element_id: u64,
    pub modified_ms: f64,
    pub properties: u32,
    pub flags: u32,
    pub range: RangeI64,
}

/// Assemble one object: common header, primary data and linkages.
pub fn encode_object(header: &HeaderFields, primary: &[u8], linkages: &[u8]) -> Result<Vec<u8>> {
    let primary_len = COMMON_HEADER_BYTES + primary.len();
    let total = primary_len + linkages.len();
    if primary_len % 2 != 0 || total % 2 != 0 {
        return Err(WriteError::InvalidElement(
            "object length must be a whole number of 16-bit words".into(),
        ));
    }
    let mut out = Vec::with_capacity(total);
    push_u32(
        &mut out,
        u32::from(header.element_type) | header.type_bits | header.role.bits(),
    ); // FN-E01, FN-E13
    push_u32(&mut out, (total / 2) as u32); // FN-P03
    push_u32(&mut out, (primary_len / 2) as u32); // FN-P03
    push_u32(&mut out, header.symbology.level); // FN-E02
    out.extend_from_slice(&header.element_id.to_le_bytes()); // FN-E02
    push_f64(&mut out, header.modified_ms); // FN-E03a
    push_u32(&mut out, header.symbology.graphic_group); // FN-E04
    push_u32(&mut out, header.properties); // FN-E05
    push_u32(&mut out, header.flags); // FN-E06
    push_u32(&mut out, header.symbology.style); // FN-E11
    push_u32(&mut out, header.symbology.weight); // FN-E11
    push_u32(&mut out, header.symbology.color); // FN-E04
    let extent = header.range.extent();
    for value in header.range.low.iter().chain(extent.iter()) {
        out.extend_from_slice(&value.to_le_bytes()); // FN-E03
    }
    debug_assert_eq!(out.len(), COMMON_HEADER_BYTES);
    out.extend_from_slice(primary);
    out.extend_from_slice(linkages);
    Ok(out)
}

/// Word-framed string linkage (FN-L01, FN-L02): header word, linkage ID,
/// property ID, payload length, `ff fd` + UTF-16LE, zero padding to 8 bytes.
#[must_use]
pub fn string_linkage(property_id: u32, value: &str) -> Vec<u8> {
    let mut payload = crate::text::UTF16_MARKER.to_vec();
    for unit in value.encode_utf16() {
        payload.extend_from_slice(&unit.to_le_bytes());
    }
    string_linkage_bytes(property_id, &payload)
}

/// String linkage with a ready payload, for example plain ASCII as
/// MicroStation writes cell names (FN-C03, FN-L03).
#[must_use]
pub fn string_linkage_bytes(property_id: u32, payload: &[u8]) -> Vec<u8> {
    let unpadded = 12 + payload.len();
    let total = unpadded.div_ceil(8) * 8;
    let mut out = Vec::with_capacity(total);
    let header_word = 0x1000u16 | ((total / 2 - 1) as u16 & 0x00ff);
    out.extend_from_slice(&header_word.to_le_bytes());
    out.extend_from_slice(&STRING_LINKAGE_ID.to_le_bytes());
    push_u32(&mut out, property_id);
    push_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(payload);
    out.resize(total, 0);
    out
}

/// Cell range in cell coordinates: relative to the origin and before the
/// cell matrix (FN-C03). The model box corners are mapped back, which is
/// exact without rotation and conservative with it.
fn cell_local_bounds(bounds: Bounds, origin: Point2, transform: &[f64; 4]) -> Result<Bounds> {
    let [a, b, c, d] = *transform;
    let det = a * d - b * c;
    if det == 0.0 || !det.is_finite() {
        return Err(WriteError::InvalidElement(
            "cell matrix is not invertible".into(),
        ));
    }
    let local = |x: f64, y: f64| {
        let (dx, dy) = (x - origin.x, y - origin.y);
        Point2::new((d * dx - b * dy) / det, (a * dy - c * dx) / det)
    };
    Ok(Bounds::of_points(&[
        local(bounds.min.x, bounds.min.y),
        local(bounds.max.x, bounds.min.y),
        local(bounds.min.x, bounds.max.y),
        local(bounds.max.x, bounds.max.y),
    ])
    .unwrap())
}

fn text_primary(text: &TextData, profile: Profile) -> Result<(Vec<u8>, Bounds)> {
    if !(text.height > 0.0 && text.width > 0.0) {
        return Err(WriteError::InvalidElement(
            "text size must be positive".into(),
        ));
    }
    let payload = encode_text(&text.text, text.encoding)?;
    if payload.is_empty() {
        return Err(WriteError::InvalidElement("text is empty".into()));
    }
    let length = u16::try_from(payload.len())
        .map_err(|_| WriteError::InvalidElement("text is too long".into()))?;
    let mut data = Vec::with_capacity(0x50 + payload.len());
    push_u32(&mut data, text.font); // 0x68 (FN-T03)
    data.extend_from_slice(&text.justification.to_le_bytes()); // 0x6c (FN-T03)
    data.extend_from_slice(&length.to_le_bytes()); // 0x6e (FN-T02)
    push_f64(&mut data, text.width * UOR_TO_TEXT_MULTIPLIER); // 0x70 (FN-T01)
    push_f64(&mut data, text.height * UOR_TO_TEXT_MULTIPLIER); // 0x78 (FN-T01)
    data.extend_from_slice(&[0u8; 16]); // 0x80..0x8f: zero accepted (FN-T03)
    push_f64(&mut data, text.rotation); // 0x90 (FN-T03)
    push_point(&mut data, text.origin); // 0x98 (FN-T03)
    data.extend_from_slice(&0u16.to_le_bytes()); // 0xa8 editable fields
    data.extend_from_slice(&payload); // 0xaa (FN-T02)
    if data.len() % 2 != 0 {
        data.push(0);
    }
    Ok((data, text_bounds(text, profile)))
}

/// Text box, rotated: `n * width` long; vertically from `-0.25 * height` to
/// `height` (`Oda`, round 1) or from `-0.5 * height` to `1.5 * height`, as
/// MicroStation stores it (`MicroStation`). The exact length depends on font
/// metrics the writer does not have (FN-T04).
fn text_bounds(text: &TextData, profile: Profile) -> Bounds {
    let length = text.text.chars().count().max(1) as f64 * text.width;
    let (sin_r, cos_r) = text.rotation.sin_cos();
    let corner = |a: f64, b: f64| {
        Point2::new(
            text.origin.x + a * cos_r - b * sin_r,
            text.origin.y + a * sin_r + b * cos_r,
        )
    };
    let (descent, ascent) = match profile {
        Profile::Oda => (-0.25 * text.height, text.height),
        Profile::MicroStation => (-0.5 * text.height, 1.5 * text.height),
    };
    Bounds::of_points(&[
        corner(0.0, descent),
        corner(length, descent),
        corner(0.0, ascent),
        corner(length, ascent),
    ])
    .unwrap()
}

fn vertex_list(points: &[Point2]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 + points.len() * 16);
    push_u32(&mut data, points.len() as u32); // 0x68 vertex count (FN-E08)
    push_u32(&mut data, 0); // 0x6c
    for point in points {
        push_point(&mut data, *point);
    }
    data
}

fn check_vertices(what: &str, points: &[Point2], minimum: usize) -> Result<()> {
    if points.len() < minimum {
        return Err(WriteError::InvalidElement(format!(
            "{what} needs at least {minimum} vertices, got {}",
            points.len()
        )));
    }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return Err(WriteError::InvalidElement(format!(
            "{what} has a non-finite vertex"
        )));
    }
    Ok(())
}

fn check_axes(primary: f64, secondary: f64) -> Result<()> {
    if primary > 0.0 && secondary > 0.0 && primary.is_finite() && secondary.is_finite() {
        Ok(())
    } else {
        Err(WriteError::InvalidElement(
            "ellipse axes must be positive".into(),
        ))
    }
}

fn check_components(what: &str, children: &[Element], allow_text: bool) -> Result<()> {
    if children.is_empty() {
        return Err(WriteError::InvalidElement(format!(
            "{what} needs components"
        )));
    }
    for child in children {
        let ok = match child.geometry {
            Geometry::Line { .. }
            | Geometry::LineString(_)
            | Geometry::Arc { .. }
            | Geometry::Curve(_) => true,
            Geometry::Shape(_) | Geometry::Ellipse { .. } | Geometry::Text(_) => allow_text,
            _ => false,
        };
        if !ok {
            return Err(WriteError::InvalidElement(format!(
                "{what} cannot contain this component type yet"
            )));
        }
    }
    Ok(())
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_point(out: &mut Vec<u8>, point: Point2) {
    push_f64(out, point.x);
    push_f64(out, point.y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_linkage_matches_fixture_layout() {
        // Fixture shared cell 92 carries this 48-byte linkage.
        let linkage = string_linkage(1, "Named definition");
        assert_eq!(linkage.len(), 48);
        assert_eq!(
            &linkage[..12],
            &[0x17, 0x10, 0xd2, 0x56, 1, 0, 0, 0, 0x22, 0, 0, 0]
        );
        assert_eq!(&linkage[12..16], &[0xff, 0xfd, b'N', 0]);
        assert_eq!(&linkage[46..], &[0, 0]);
        // Model header linkage "m" fills exactly 16 bytes without padding.
        assert_eq!(
            string_linkage(0x13, "m"),
            [0x07, 0x10, 0xd2, 0x56, 0x13, 0, 0, 0, 4, 0, 0, 0, 0xff, 0xfd, b'm', 0]
        );
    }

    // The tests below rebuild layouts observed in MicroStation-written
    // objects (EXP-0003, E2-E4); only the observed numbers are used here.

    const MS: EncodeContext = EncodeContext {
        modified_ms: 0.0,
        profile: Profile::MicroStation,
    };

    fn plain(geometry: Geometry) -> Element {
        Element {
            symbology: Symbology {
                level: 64,
                ..Symbology::default()
            },
            geometry,
            extra_flags: 0,
        }
    }

    fn encode_one(element: &Element) -> Vec<Vec<u8>> {
        element
            .encode(&mut IdAllocator::starting_at(1288), &MS)
            .unwrap()
            .objects
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    fn i64_at(bytes: &[u8], at: usize) -> i64 {
        i64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
    }

    fn f64_at(bytes: &[u8], at: usize) -> f64 {
        f64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
    }

    fn text(text: &str, origin: Point2, height: f64) -> TextData {
        TextData {
            text: text.into(),
            encoding: TextEncoding::Auto,
            origin,
            rotation: 0.0,
            height,
            width: height,
            font: 1025,
            justification: 0,
        }
    }

    #[test]
    fn microstation_profile_sets_type_bit_and_clears_properties() {
        // E2 line: type word 0x10800003, properties 0 (FN-E13).
        let line = plain(Geometry::Line {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(0.0, 105_000.0),
        });
        let objects = encode_one(&line);
        assert_eq!(u32_at(&objects[0], 0), 0x1080_0003);
        assert_eq!(u32_at(&objects[0], 0x24), 0);
        assert_eq!(u32_at(&objects[0], 0x28), 0);
    }

    #[test]
    fn microstation_text_range_spans_half_height_below_to_one_and_a_half_above() {
        // E3: 0.25 m text (2500 UOR) at y 51866.3 stores low y 50616 and
        // extent 5001 (FN-T04).
        let element = plain(Geometry::Text(text(
            "Gl",
            Point2::new(15314.98, 51866.3),
            2500.0,
        )));
        let object = &encode_one(&element)[0];
        assert_eq!(i64_at(object, 0x38), 15314);
        assert_eq!(i64_at(object, 0x40), 50616);
        assert_eq!(i64_at(object, 0x58), 5001);
    }

    #[test]
    fn microstation_text_node_writes_style_block() {
        // E4 node: font 1025, 255, justification 0, spacing 5000 UOR,
        // size multipliers of a 2500-UOR text (FN-N02).
        let style = TextNodeStyle {
            font: 1025,
            justification: 0,
            line_spacing: 5000.0,
            width: 2500.0,
            height: 2500.0,
        };
        let node = plain(Geometry::TextNode {
            origin: Point2::new(89580.4, 35926.5),
            rotation: 0.0,
            node_number: 1,
            style,
            lines: vec![
                (
                    Symbology::default(),
                    text("two line", Point2::new(89580.4, 33426.5), 2500.0),
                ),
                (
                    Symbology::default(),
                    text("text", Point2::new(89580.4, 25926.5), 2500.0),
                ),
            ],
            linkages: NodeLinkages::None,
        });
        let objects = encode_one(&node);
        let header = &objects[0];
        assert_eq!(u32_at(header, 0), 0x3080_0007);
        assert_eq!(
            &header[0x68..0x78],
            &[2, 0, 0, 0, 1, 0, 0, 0, 0x01, 0x04, 0, 0, 0xff, 0, 0, 0]
        );
        assert_eq!(f64_at(header, 0x78), 5000.0);
        assert_eq!(f64_at(header, 0x80), 2500.0 * UOR_TO_TEXT_MULTIPLIER);
        // E4 node range: low (89580, 24676), extent (11695, 12501) with
        // MicroStation's own text length; the vertical part is ours too.
        assert_eq!(i64_at(header, 0x40), 24676);
        assert_eq!(i64_at(header, 0x58), 12501);
        assert_eq!(u32_at(&objects[1], 0), 0x5080_0011);
    }

    #[test]
    fn node_linkage_experiment_mirrors_e4() {
        // E4: every line and the node carry 0x80d4 and 0x56f3 linkages; the
        // 0x80d4 linkage of the first line has 0x80 at byte 8 (H-N04).
        let node = |linkages| {
            let style = TextNodeStyle {
                font: 1025,
                justification: 0,
                line_spacing: 5000.0,
                width: 2500.0,
                height: 2500.0,
            };
            let line = |y| {
                (
                    Symbology::default(),
                    text("ab", Point2::new(0.0, y), 2500.0),
                )
            };
            encode_one(&plain(Geometry::TextNode {
                origin: Point2::new(0.0, 2500.0),
                rotation: 0.0,
                node_number: 1,
                style,
                lines: vec![line(0.0), line(-7500.0)],
                linkages,
            }))
        };
        let tail = |object: &Vec<u8>| object[2 * u32_at(object, 8) as usize..].to_vec();
        let mut first_line = TEXT_RUN_LINKAGE.to_vec();
        first_line[TEXT_RUN_LINE_BREAK_AT] = 0x80;

        let full = node(NodeLinkages::Full);
        assert_eq!(
            tail(&full[0]),
            [TEXT_RUN_LINKAGE, TEXT_SCALE_LINKAGE].concat()
        );
        assert_eq!(
            tail(&full[1]),
            [&first_line[..], &TEXT_SCALE_LINKAGE].concat()
        );
        assert_eq!(
            tail(&full[2]),
            [TEXT_RUN_LINKAGE, TEXT_SCALE_LINKAGE].concat()
        );
        assert_eq!(u32_at(&full[1], 4) as usize * 2, full[1].len());

        let breaks = node(NodeLinkages::LineBreaks);
        assert!(tail(&breaks[0]).is_empty());
        assert_eq!(tail(&breaks[1]), first_line);
        assert_eq!(tail(&breaks[2]), TEXT_RUN_LINKAGE);

        let none = node(NodeLinkages::None);
        assert!(none.iter().all(|object| tail(object).is_empty()));
    }

    #[test]
    fn microstation_cell_stores_local_range_and_ascii_name() {
        // E4 cell: origin (46722.0, 23993.5), local range (-4061, 0) to
        // (4061, 15000), 0x6c = 01 00 01 00, 6-letter ASCII name in a
        // 24-byte linkage (FN-C03).
        let origin = Point2::new(46722.0, 23993.5);
        let cell = Element {
            symbology: Symbology::default(),
            geometry: Geometry::Cell {
                name: Some("ABCDEF".into()),
                origin,
                transform: [1.0, 0.0, 0.0, 1.0],
                children: vec![
                    plain(Geometry::Line {
                        start: Point2::new(origin.x - 4061.0, origin.y),
                        end: Point2::new(origin.x + 4061.0, origin.y),
                    }),
                    plain(Geometry::Line {
                        start: origin,
                        end: Point2::new(origin.x, origin.y + 15000.0),
                    }),
                ],
            },
            extra_flags: 0,
        };
        let header = &encode_one(&cell)[0];
        assert_eq!(u32_at(header, 0), 0x3080_0002);
        assert_eq!(&header[0x68..0x70], &[2, 0, 0, 0, 1, 0, 1, 0]);
        assert_eq!(
            [
                f64_at(header, 0x70),
                f64_at(header, 0x78),
                f64_at(header, 0x80),
                f64_at(header, 0x88)
            ],
            [-4061.0, 0.0, 4061.0, 15000.0]
        );
        assert_eq!(i64_at(header, 0x38), 42661);
        assert_eq!(
            &header[0xc0..],
            &[
                0x0b, 0x10, 0xd2, 0x56, 1, 0, 0, 0, 6, 0, 0, 0, b'A', b'B', b'C', b'D', b'E', b'F',
                0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn cell_local_range_undoes_rotation_and_scale() {
        // 90 degrees and scale 2: model box (0..2, 0..4) around origin (0, 0)
        // is local (0..2, -1..0).
        let local = cell_local_bounds(
            Bounds::of_points(&[Point2::new(0.0, 0.0), Point2::new(2.0, 4.0)]).unwrap(),
            Point2::new(0.0, 0.0),
            &[0.0, -2.0, 2.0, 0.0],
        )
        .unwrap();
        assert_eq!(
            (local.min, local.max),
            (Point2::new(0.0, -1.0), Point2::new(2.0, 0.0))
        );
    }
}
