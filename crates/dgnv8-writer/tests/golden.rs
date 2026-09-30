//! Golden-byte tests: the encoders must reproduce objects of the GDAL V8
//! fixture (an ODA-produced file) byte for byte. This is the evidence that
//! the layouts in docs/FORMAT_NOTES.md match an independent producer.
//!
//! Text ranges depend on font metrics the writer does not have (FN-T04), so
//! for text objects the range bytes 0x38..0x68 are excluded from comparison.

use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_4, PI};
use std::io::{Cursor, Read};

use dgnv8_writer::element::{
    Element, EncodeContext, Geometry, IdAllocator, Symbology, TextData, FLAG_3D, FLAG_HOLE,
    FLAG_NEW,
};
use dgnv8_writer::page::Page;
use dgnv8_writer::{Point2, TextEncoding};

const FIXTURE: &[u8] = include_bytes!("data/gdal_test_dgnv8.dgn");
/// Last-modified stamp shared by every fixture object (FN-E03a).
const FIXTURE_TIME_MS: f64 = 1_490_737_306_000.0;
const RANGE: std::ops::Range<usize> = 0x38..0x68;

fn fixture_objects() -> HashMap<u64, Vec<u8>> {
    let mut compound = cfb::CompoundFile::open(Cursor::new(FIXTURE)).unwrap();
    let mut raw = Vec::new();
    compound
        .open_stream("/Dgn-Md/#000000/Dgn^G/$1")
        .unwrap()
        .read_to_end(&mut raw)
        .unwrap();
    Page::parse(&raw)
        .unwrap()
        .objects
        .into_iter()
        .map(|object| (object.element_id().unwrap(), object.bytes))
        .collect()
}

fn default_symbology() -> Symbology {
    Symbology {
        level: 64,
        ..Symbology::default()
    }
}

fn simple(geometry: Geometry) -> Element {
    Element {
        symbology: default_symbology(),
        geometry,
        extra_flags: 0,
    }
}

/// Some sample objects carry the "New" property bit (FN-E06).
fn new_flagged(geometry: Geometry) -> Element {
    Element {
        extra_flags: FLAG_NEW,
        ..simple(geometry)
    }
}

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

/// Encode `element` with IDs starting at `first_id`; returns all objects.
fn encode(element: &Element, first_id: u64) -> Vec<Vec<u8>> {
    let mut ids = IdAllocator::starting_at(first_id);
    let context = EncodeContext {
        modified_ms: FIXTURE_TIME_MS,
    };
    element.encode(&mut ids, &context).unwrap().objects
}

fn assert_same(ids: &[u64], encoded: &[Vec<u8>], mask_range: &[bool]) {
    let fixture = fixture_objects();
    assert_eq!(ids.len(), encoded.len(), "object count");
    for (index, id) in ids.iter().enumerate() {
        let expected = &fixture[id];
        let mut actual = encoded[index].clone();
        let mut expected = expected.clone();
        if mask_range[index] {
            actual[RANGE].fill(0);
            expected[RANGE].fill(0);
        }
        assert_eq!(
            hex(&actual),
            hex(&expected),
            "object {id} differs from the fixture"
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .chunks(16)
        .map(|row| {
            row.iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn zero_length_line_37() {
    let line = simple(Geometry::Line {
        start: p(0.0, 10000.0),
        end: p(0.0, 10000.0),
    });
    assert_same(&[37], &encode(&line, 37), &[false]);
}

#[test]
fn line_39() {
    let line = simple(Geometry::Line {
        start: p(0.0, 10000.0),
        end: p(20000.0, 30000.0),
    });
    assert_same(&[39], &encode(&line, 39), &[false]);
}

#[test]
fn line_string_47() {
    let element = new_flagged(Geometry::LineString(vec![
        p(0.0, 10000.0),
        p(30000.0, 40000.0),
        p(60000.0, 70000.0),
    ]));
    assert_same(&[47], &encode(&element, 47), &[false]);
}

#[test]
fn curve_57() {
    let element = new_flagged(Geometry::Curve(vec![
        p(0.0, 0.0),
        p(0.0, 10000.0),
        p(10000.0, 10000.0),
        p(10000.0, 0.0),
        p(0.0, 0.0),
        p(0.0, 10000.0),
        p(10000.0, 10000.0),
        p(10000.0, 0.0),
    ]));
    assert_same(&[57], &encode(&element, 57), &[false]);
}

#[test]
fn ellipse_51_and_circle_52() {
    let ellipse = simple(Geometry::Ellipse {
        center: p(0.0, 10000.0),
        primary: 10000.0,
        secondary: 20000.0,
        rotation: 0.0,
    });
    assert_same(&[51], &encode(&ellipse, 51), &[false]);
    let circle = simple(Geometry::Ellipse {
        center: p(0.0, 10000.0),
        primary: 10000.0,
        secondary: 10000.0,
        rotation: FRAC_PI_4,
    });
    assert_same(&[52], &encode(&circle, 52), &[false]);
}

#[test]
fn arcs_54_and_55_including_rounded_ranges() {
    let start = 10f64.to_radians();
    let arc = simple(Geometry::Arc {
        center: p(0.0, 10000.0),
        primary: 10000.0,
        secondary: 20000.0,
        rotation: 0.0,
        start,
        sweep: PI,
    });
    assert_same(&[54], &encode(&arc, 54), &[false]);
    let rotated = simple(Geometry::Arc {
        center: p(0.0, 10000.0),
        primary: 10000.0,
        secondary: 10000.0,
        rotation: FRAC_PI_4,
        start,
        sweep: PI,
    });
    assert_same(&[55], &encode(&rotated, 55), &[false]);
}

#[test]
fn text_40_escaped_windows_1252() {
    let text = Element {
        symbology: Symbology {
            level: 64,
            color: 256,
            ..Symbology::default()
        },
        geometry: Geometry::Text(TextData {
            text: "myTéxt".into(),
            encoding: TextEncoding::EscapedCp1252,
            origin: p(0.0, 10000.0),
            rotation: -FRAC_PI_4,
            height: 10000.0,
            width: 10000.0,
            font: 1024,
            justification: 0,
        }),
        extra_flags: 0,
    };
    assert_same(&[40], &encode(&text, 40), &[true]);
}

#[test]
fn text_node_42_with_text_43() {
    let node = simple(Geometry::TextNode {
        origin: p(0.0, 0.0),
        rotation: 0.0,
        node_number: 1,
        lines: vec![(
            default_symbology(),
            TextData {
                text: "z".into(),
                encoding: TextEncoding::Ascii,
                origin: p(0.0, 10000.0),
                rotation: 0.0,
                height: 10000.0,
                width: 10000.0,
                font: 127,
                justification: 0,
            },
        )],
    });
    // The fixture node header is 0x70..0x8f zero-filled, like ours (H-N01).
    assert_same(&[42, 43], &encode(&node, 42), &[true, true]);
}

#[test]
fn complex_chain_61() {
    // The fixture header carries the 3D bit although its lines are 2D.
    let chain = Element {
        symbology: default_symbology(),
        geometry: Geometry::ComplexChain(vec![
            simple(Geometry::Line {
                start: p(0.0, 10000.0),
                end: p(20000.0, 30000.0),
            }),
            simple(Geometry::Line {
                start: p(20000.0, 30000.0),
                end: p(40000.0, 50000.0),
            }),
        ]),
        extra_flags: FLAG_3D,
    };
    assert_same(&[61, 62, 63], &encode(&chain, 61), &[false, false, false]);
}

#[test]
fn complex_shape_82_with_arc() {
    let shape = Element {
        symbology: default_symbology(),
        geometry: Geometry::ComplexShape(vec![
            simple(Geometry::Line {
                start: p(0.0, 0.0),
                end: p(0.0, 10000.0),
            }),
            simple(Geometry::Arc {
                center: p(5000.0, 10000.0),
                primary: 5000.0,
                secondary: 5000.0,
                rotation: 0.0,
                start: PI,
                sweep: -PI,
            }),
            simple(Geometry::Line {
                start: p(10000.0, 10000.0),
                end: p(0.0, 0.0),
            }),
        ]),
        extra_flags: FLAG_3D,
    };
    assert_same(&[82, 83, 84, 85], &encode(&shape, 82), &[false; 4]);
}

#[test]
fn cell_72_with_hole() {
    let square = |a: f64, b: f64| vec![p(a, a), p(a, b), p(b, b), p(a, a)];
    let cell = Element {
        symbology: Symbology::default(), // the fixture cell header is on level 0
        geometry: Geometry::Cell {
            name: None,
            origin: p(0.0, 0.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            children: vec![
                new_flagged(Geometry::Shape(square(0.0, 10000.0))),
                Element {
                    symbology: default_symbology(),
                    geometry: Geometry::Shape(square(1000.0, 9000.0)),
                    extra_flags: FLAG_NEW | FLAG_HOLE,
                },
            ],
        },
        // Grouped hole in the sample: New, 0x4000 and hole bits on the header.
        extra_flags: FLAG_NEW | 0x4000 | FLAG_HOLE,
    };
    assert_same(&[72, 73, 74], &encode(&cell, 72), &[false; 3]);
}
