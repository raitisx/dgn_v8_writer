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
/// Flag bit for a 3D object (FN-E06). Never set by this 2D writer.
pub const FLAG_3D: u32 = 0x0000_0800;
/// Flag observed on 2D line strings, shapes, curves and cells (FN-E06).
pub const FLAG_2D_VERTEX_LIST: u32 = 0x0000_0200;
/// Flag observed on hole shapes inside a cell (FN-E06).
pub const FLAG_HOLE: u32 = 0x0000_8000;
/// Text size multiplier: UOR = raw * 6 / 1000 (FN-T01). Written as
/// `uor * (1000 / 6)`, which reproduces the fixture's bits exactly.
pub const UOR_TO_TEXT_MULTIPLIER: f64 = 1000.0 / 6.0;
/// Property/string linkage identifier (FN-L02).
pub const STRING_LINKAGE_ID: u16 = 0x56d2;

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

/// Display attributes written into the common header (FN-E04).
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
    /// Raw justification code (FN-T03 only knows code 0 = left baseline).
    pub justification: u16,
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
        origin: Point2,
        rotation: f64,
        node_number: u32,
        lines: Vec<(Symbology, TextData)>,
    },
    ComplexChain(Vec<Element>),
    ComplexShape(Vec<Element>),
    Cell {
        /// Experimental name linkage (H-C02); `None` writes an unnamed cell.
        name: Option<String>,
        origin: Point2,
        /// Row-major 2x2 matrix (FN-C01 only confirms the identity).
        transform: [f64; 4],
        children: Vec<Element>,
    },
}

/// Settings shared by every object written in one save.
#[derive(Debug, Clone, Copy)]
pub struct EncodeContext {
    /// Last-modified time, milliseconds since 1970-01-01 UTC (FN-E03a).
    pub modified_ms: f64,
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
        self.encode_with_role(Role::Standalone, ids, context)
    }

    fn encode_with_role(
        &self,
        role: Role,
        ids: &mut IdAllocator,
        context: &EncodeContext,
    ) -> Result<Encoded> {
        let simple = |element_type: u8, flags: u32, primary: Vec<u8>, bounds: Bounds| {
            let range = RangeI64::from_bounds(bounds);
            let header = HeaderFields {
                element_type,
                role,
                symbology: self.symbology,
                element_id: 0,
                modified_ms: context.modified_ms,
                properties: DEFAULT_PROPERTIES,
                flags: flags | self.extra_flags,
                range,
            };
            (header, primary, range)
        };
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
                    FLAG_2D_VERTEX_LIST,
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
                    FLAG_2D_VERTEX_LIST,
                    vertex_list(points),
                    Bounds::of_points(points).unwrap(),
                );
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::Curve(points) => {
                check_vertices("curve", points, 6)?;
                let (h, p, r) = simple(
                    TYPE_CURVE,
                    FLAG_2D_VERTEX_LIST,
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
                let (data, bounds) = text_primary(text)?;
                let (h, p, r) = simple(TYPE_TEXT, 0, data, bounds);
                (h, p, r, Vec::new(), Vec::new())
            }
            Geometry::TextNode {
                origin,
                rotation,
                node_number,
                lines,
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
                    .collect();
                let mut bounds = Bounds::of_point(*origin);
                for child in &children {
                    bounds = bounds.union(child.bounds()?);
                }
                // FN-N01: line count, node number, rotation, origin.
                let mut data = Vec::with_capacity(0x40);
                push_u32(&mut data, lines.len() as u32);
                push_u32(&mut data, *node_number);
                data.extend_from_slice(&[0u8; 0x20]); // 0x70..0x8f: unresolved (H-N01)
                push_f64(&mut data, *rotation);
                push_point(&mut data, *origin);
                let (h, p, r) = simple(TYPE_TEXT_NODE, 0, data, bounds);
                let h = HeaderFields {
                    role: header_role(role),
                    ..h
                };
                (h, p, r, Vec::new(), children)
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
                    let child_bounds = child.bounds()?;
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
                (h, p, r, Vec::new(), children.clone())
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
                    let child_bounds = child.bounds()?;
                    bounds = Some(bounds.map_or(child_bounds, |b| b.union(child_bounds)));
                }
                let bounds = bounds.unwrap();
                let mut data = Vec::with_capacity(0x58);
                push_u32(&mut data, children.len() as u32);
                push_u32(&mut data, 1); // 0x6c: value 1 observed (FN-C01)
                push_point(&mut data, bounds.min); // range low (FN-C01, V7 analogue)
                push_point(&mut data, bounds.max); // range high
                for value in transform {
                    push_f64(&mut data, *value);
                }
                push_point(&mut data, *origin);
                let linkages = match name {
                    Some(name) => string_linkage(1, name),
                    None => Vec::new(),
                };
                let (h, p, r) = simple(TYPE_CELL, FLAG_2D_VERTEX_LIST, data, bounds);
                let h = HeaderFields {
                    role: header_role(role),
                    ..h
                };
                (h, p, r, linkages, children.clone())
            }
        };

        let mut header = header;
        header.element_id = ids.allocate();
        let mut objects = vec![encode_object(&header, &primary, &linkages)?];
        for child in &components {
            let encoded = child.encode_with_role(Role::Component, ids, context)?;
            objects.extend(encoded.objects);
        }
        Ok(Encoded { objects, range })
    }

    /// Floating bounds of this element in UOR.
    pub fn bounds(&self) -> Result<Bounds> {
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
            Geometry::Text(text) => text_bounds(text),
            Geometry::TextNode { origin, lines, .. } => {
                let mut bounds = Bounds::of_point(*origin);
                for (_, line) in lines {
                    bounds = bounds.union(text_bounds(line));
                }
                bounds
            }
            Geometry::ComplexChain(children)
            | Geometry::ComplexShape(children)
            | Geometry::Cell { children, .. } => {
                let mut bounds: Option<Bounds> = None;
                for child in children {
                    let child_bounds = child.bounds()?;
                    bounds = Some(bounds.map_or(child_bounds, |b| b.union(child_bounds)));
                }
                bounds
                    .ok_or_else(|| WriteError::InvalidElement("complex element is empty".into()))?
            }
        })
    }
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
        u32::from(header.element_type) | header.role.bits(),
    ); // FN-E01
    push_u32(&mut out, (total / 2) as u32); // FN-P03
    push_u32(&mut out, (primary_len / 2) as u32); // FN-P03
    push_u32(&mut out, header.symbology.level); // FN-E02
    out.extend_from_slice(&header.element_id.to_le_bytes()); // FN-E02
    push_f64(&mut out, header.modified_ms); // FN-E03a
    push_u32(&mut out, header.symbology.graphic_group); // FN-E04
    push_u32(&mut out, header.properties); // FN-E05
    push_u32(&mut out, header.flags); // FN-E06
    push_u32(&mut out, header.symbology.style); // FN-E04
    push_u32(&mut out, header.symbology.weight); // FN-E04
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
    let unpadded = 12 + payload.len();
    let total = unpadded.div_ceil(8) * 8;
    let mut out = Vec::with_capacity(total);
    let header_word = 0x1000u16 | ((total / 2 - 1) as u16 & 0x00ff);
    out.extend_from_slice(&header_word.to_le_bytes());
    out.extend_from_slice(&STRING_LINKAGE_ID.to_le_bytes());
    push_u32(&mut out, property_id);
    push_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(&payload);
    out.resize(total, 0);
    out
}

fn text_primary(text: &TextData) -> Result<(Vec<u8>, Bounds)> {
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
    data.extend_from_slice(&[0u8; 16]); // 0x80..0x8f: zero in every sample
    push_f64(&mut data, text.rotation); // 0x90 (FN-T03)
    push_point(&mut data, text.origin); // 0x98 (FN-T03)
    data.extend_from_slice(&0u16.to_le_bytes()); // 0xa8 editable fields
    data.extend_from_slice(&payload); // 0xaa (FN-T02)
    if data.len() % 2 != 0 {
        data.push(0);
    }
    Ok((data, text_bounds(text)))
}

/// Conservative text box: `n * width` long, from `-0.25 * height` (descenders)
/// to `height` above the baseline origin, rotated. The exact box depends on
/// font metrics the writer does not have (FN-T04).
fn text_bounds(text: &TextData) -> Bounds {
    let length = text.text.chars().count().max(1) as f64 * text.width;
    let (sin_r, cos_r) = text.rotation.sin_cos();
    let corner = |a: f64, b: f64| {
        Point2::new(
            text.origin.x + a * cos_r - b * sin_r,
            text.origin.y + a * sin_r + b * cos_r,
        )
    };
    let descent = -0.25 * text.height;
    Bounds::of_points(&[
        corner(0.0, descent),
        corner(length, descent),
        corner(0.0, text.height),
        corner(length, text.height),
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
}
