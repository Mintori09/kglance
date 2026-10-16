use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_supported_extensions() {
    let parser = Model3dParser;
    assert!(parser.supported_extensions().contains(&"stl"));
    assert!(parser.supported_extensions().contains(&"obj"));
}

#[test]
fn test_parse_ascii_stl() {
    let ascii_stl = r#"solid test_cube
  facet normal 0.0 0.0 1.0
    outer loop
      vertex 0.0 0.0 0.0
      vertex 1.0 0.0 0.0
      vertex 1.0 1.0 0.0
    endloop
  endfacet
  facet normal 0.0 0.0 1.0
    outer loop
      vertex 0.0 0.0 0.0
      vertex 1.0 1.0 0.0
      vertex 0.0 1.0 0.0
    endloop
  endfacet
endsolid test_cube
"#;

    let mut temp = NamedTempFile::with_suffix(".stl").unwrap();
    temp.write_all(ascii_stl.as_bytes()).unwrap();

    let parser = Model3dParser;
    let res = parser.parse(temp.path()).expect("Should parse ASCII STL");

    match res {
        ParsedContent::Model3d(mesh) => {
            assert_eq!(mesh.triangle_count, 2);
            assert_eq!(mesh.vertex_count, 6);
            assert_eq!(mesh.indices.len(), 6);
            assert_eq!(mesh.vertices.len(), 6 * 6);
            assert!(mesh.bounds.is_valid());
            assert_eq!(mesh.bounds.min[0], 0.0);
            assert_eq!(mesh.bounds.max[0], 1.0);
        }
        _ => panic!("Expected ParsedContent::Model3d"),
    }
}

#[test]
fn test_parse_binary_stl() {
    let mut data = Vec::new();
    // 80 bytes header
    data.extend_from_slice(&[0u8; 80]);
    // 1 triangle
    data.extend_from_slice(&1u32.to_le_bytes());

    // Normal (0.0, 0.0, 1.0)
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&1.0f32.to_le_bytes());

    // V1 (0, 0, 0)
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());

    // V2 (2, 0, 0)
    data.extend_from_slice(&2.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());

    // V3 (0, 2, 0)
    data.extend_from_slice(&0.0f32.to_le_bytes());
    data.extend_from_slice(&2.0f32.to_le_bytes());
    data.extend_from_slice(&0.0f32.to_le_bytes());

    // 2 bytes attribute
    data.extend_from_slice(&[0u8; 2]);

    let mut temp = NamedTempFile::with_suffix(".stl").unwrap();
    temp.write_all(&data).unwrap();

    let parser = Model3dParser;
    let res = parser.parse(temp.path()).expect("Should parse binary STL");

    match res {
        ParsedContent::Model3d(mesh) => {
            assert_eq!(mesh.triangle_count, 1);
            assert_eq!(mesh.vertex_count, 3);
            assert_eq!(mesh.indices.len(), 3);
            assert_eq!(mesh.vertices.len(), 3 * 6);
            assert!(mesh.bounds.is_valid());
            assert_eq!(mesh.bounds.max_dimension(), 2.0);
        }
        _ => panic!("Expected ParsedContent::Model3d"),
    }
}

#[test]
fn test_parse_obj() {
    let obj_content = r#"# Simple Quad OBJ
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vn 0.0 0.0 1.0
f 1//1 2//1 3//1
f 1//1 3//1 4//1
"#;

    let mut temp = NamedTempFile::with_suffix(".obj").unwrap();
    temp.write_all(obj_content.as_bytes()).unwrap();

    let parser = Model3dParser;
    let res = parser.parse(temp.path()).expect("Should parse OBJ file");

    match res {
        ParsedContent::Model3d(mesh) => {
            assert_eq!(mesh.triangle_count, 2);
            assert_eq!(mesh.indices.len(), 6);
            assert_eq!(mesh.vertex_count, 4);
            assert!(mesh.bounds.is_valid());
            assert_eq!(mesh.vertices.len(), 4 * 6);
        }
        _ => panic!("Expected ParsedContent::Model3d"),
    }
}
