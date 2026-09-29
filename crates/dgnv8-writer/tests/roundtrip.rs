//! Write elements onto the GDAL fixture seed, then read the result back with
//! the independent ezdgn reader. The fixture model is 3D, so these tests use
//! `allow_3d_model` (the fixture itself stores 2D objects in that model).

use std::f64::consts::FRAC_PI_2;

use dgnv8_writer::element::{Geometry, Symbology, TextData};
use dgnv8_writer::{Document, Element, Point2, TextEncoding, WriteOptions};
use ezdgn_core::{read_v8, V8ElementData, V8ScanOptions};

const FIXTURE: &[u8] = include_bytes!("data/gdal_test_dgnv8.dgn");
const SEED_OBJECTS: usize = 58;

fn options() -> WriteOptions {
    WriteOptions {
        modified_ms: Some(1_790_000_000_000.0),
        update_file_header_counter: false,
        allow_3d_model: true,
    }
}

fn sym(level: u32, color: u32) -> Symbology {
    Symbology {
        level,
        color,
        weight: 2,
        style: 0,
        graphic_group: 0,
    }
}

fn element(geometry: Geometry) -> Element {
    Element {
        symbology: sym(64, 3),
        geometry,
        extra_flags: 0,
    }
}

fn document() -> Document {
    Document::from_seed_bytes(FIXTURE.to_vec(), None).unwrap()
}

fn text(value: &str, origin: Point2) -> TextData {
    TextData {
        text: value.into(),
        encoding: TextEncoding::Ascii,
        origin,
        rotation: 0.0,
        height: 5000.0,
        width: 5000.0,
        font: 127,
        justification: 0,
    }
}

#[test]
fn appended_elements_read_back_with_ezdgn() {
    let mut doc = document();
    let p = |x: f64, y: f64| doc.master_to_uor(x, y);
    let elements = vec![
        element(Geometry::Line {
            start: p(100.0, 100.0),
            end: p(110.0, 105.0),
        }),
        element(Geometry::LineString(vec![
            p(100.0, 110.0),
            p(105.0, 115.0),
            p(110.0, 110.0),
        ])),
        element(Geometry::Shape(vec![
            p(120.0, 100.0),
            p(130.0, 100.0),
            p(130.0, 110.0),
            p(120.0, 110.0),
            p(120.0, 100.0),
        ])),
        element(Geometry::Ellipse {
            center: p(140.0, 105.0),
            primary: 50000.0,
            secondary: 50000.0,
            rotation: 0.0,
        }),
        element(Geometry::Arc {
            center: p(160.0, 105.0),
            primary: 50000.0,
            secondary: 30000.0,
            rotation: 0.0,
            start: 0.0,
            sweep: FRAC_PI_2,
        }),
        element(Geometry::Text(text("ADTI 1", p(100.0, 90.0)))),
        element(Geometry::TextNode {
            origin: p(120.0, 90.0),
            rotation: 0.0,
            node_number: 7,
            lines: vec![
                (sym(64, 3), text("line one", p(120.0, 90.0))),
                (sym(64, 3), text("line two", p(120.0, 88.5))),
            ],
        }),
        element(Geometry::Cell {
            name: None,
            origin: p(150.0, 90.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            children: vec![
                element(Geometry::Line {
                    start: p(149.0, 90.0),
                    end: p(151.0, 90.0),
                }),
                element(Geometry::Line {
                    start: p(150.0, 89.0),
                    end: p(150.0, 91.0),
                }),
            ],
        }),
    ];
    for e in elements {
        doc.add(e);
    }
    let (bytes, report) = doc.to_bytes(&options()).unwrap();
    // 8 top-level elements + 2 text-node lines + 2 cell components.
    assert_eq!(report.objects_written, 12);
    assert_eq!(report.first_element_id, 96);
    assert_eq!(report.last_element_id, 107);

    let read = read_v8(&bytes, V8ScanOptions::default()).unwrap();
    let model = &read.models[0];
    assert_eq!(model.elements.len(), SEED_OBJECTS + 12);
    let new: Vec<_> = model.elements[SEED_OBJECTS..].iter().collect();

    let kinds: Vec<&str> = new.iter().map(|e| e.kind()).collect();
    assert_eq!(
        kinds,
        [
            "LINE",
            "LINE_STRING",
            "SHAPE",
            "ELLIPSE",
            "ARC",
            "TEXT",
            "TEXT_NODE",
            "TEXT",
            "TEXT",
            "CELL",
            "LINE",
            "LINE"
        ]
    );
    for (offset, e) in new.iter().enumerate() {
        assert_eq!(e.common.element_id, 96 + offset as u64);
        assert_eq!(e.common.level, 64);
    }
    match &new[0].data {
        V8ElementData::Line { start, end } => {
            assert_eq!((start.master.x, start.master.y), (100.0, 100.0));
            assert_eq!((end.master.x, end.master.y), (110.0, 105.0));
        }
        other => panic!("unexpected {other:?}"),
    }
    match &new[5].data {
        V8ElementData::Text {
            text,
            height_master,
            ..
        } => {
            assert_eq!(text, "ADTI 1");
            assert!((height_master - 0.5).abs() < 1e-9);
        }
        other => panic!("unexpected {other:?}"),
    }
    // Hierarchy is restored by the independent reader.
    assert_eq!(new[6].child_indices.len(), 2);
    assert_eq!(new[9].child_indices.len(), 2);
    assert_eq!(new[10].parent_index, Some(SEED_OBJECTS + 9));
}

#[test]
fn seed_objects_survive_unchanged() {
    let mut doc = document();
    let start = doc.master_to_uor(0.0, 0.0);
    let end = doc.master_to_uor(1.0, 1.0);
    doc.add(element(Geometry::Line { start, end }));
    let (bytes, _) = doc.to_bytes(&options()).unwrap();
    let before = read_v8(FIXTURE, V8ScanOptions::default()).unwrap();
    let after = read_v8(&bytes, V8ScanOptions::default()).unwrap();
    for (old, new) in before.models[0]
        .elements
        .iter()
        .zip(after.models[0].elements.iter())
    {
        assert_eq!(old.raw.as_bytes(), new.raw.as_bytes());
    }
    // Control objects and other streams are untouched.
    assert_eq!(
        before.raw.models.len(),
        after.raw.models.len(),
        "model count"
    );
}

#[test]
fn extents_grow_to_cover_new_elements() {
    let mut doc = document();
    let start = doc.master_to_uor(-50.0, -60.0);
    let end = doc.master_to_uor(70.0, 80.0);
    doc.add(element(Geometry::Line { start, end }));
    let (bytes, report) = doc.to_bytes(&options()).unwrap();
    assert_eq!(report.extents_low_uor, [-500_000, -600_000, 0]);
    assert_eq!(report.extents_high_uor, [700_000, 800_000, 80_000]);
    let info = dgnv8_writer::analyse(&bytes, None).unwrap();
    assert_eq!(info.extents_low_uor, report.extents_low_uor);
    assert_eq!(info.extents_high_uor, report.extents_high_uor);
    assert_eq!(info.max_element_id, 96);
}

#[test]
fn refuses_2d_elements_in_3d_model_by_default() {
    let doc = document();
    assert!(doc.to_bytes(&WriteOptions::default()).is_err());
}

#[test]
fn optional_file_header_counter_update() {
    let mut doc = document();
    let start = doc.master_to_uor(0.0, 0.0);
    let end = doc.master_to_uor(1.0, 0.0);
    doc.add(element(Geometry::Line { start, end }));
    let mut opts = options();
    opts.update_file_header_counter = true;
    let (bytes, report) = doc.to_bytes(&opts).unwrap();
    assert_eq!(report.file_header_counter_written, Some(96));
    let info = dgnv8_writer::analyse(&bytes, None).unwrap();
    assert_eq!(info.file_header_id_counter, Some(96));
}
