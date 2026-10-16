use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::model3d::math::{cross, normalize, sub};
use crate::features::model3d::types::{BoundingBox, Model3dMesh};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::Arc;

pub const MAX_TRIANGLES: usize = 2_500_000;

pub struct Model3dParser;

impl PreviewParser for Model3dParser {
    fn supported_extensions(&self) -> &[&str] {
        &["stl", "obj"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "stl" => parse_stl(path),
            "obj" => parse_obj(path),
            _ => Err(ParseError::UnsupportedFormat),
        }
    }
}

pub fn parse_stl(path: &Path) -> Result<ParsedContent, ParseError> {
    let mut file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let metadata = file
        .metadata()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let file_size = metadata.len();

    // Check if it's binary STL (80 bytes header + 4 bytes triangle count)
    if file_size >= 84 {
        let mut header = [0u8; 84];
        if file.read_exact(&mut header).is_ok() {
            let num_triangles = u32::from_le_bytes(header[80..84].try_into().unwrap()) as u64;
            let expected_size = 84 + num_triangles * 50;
            if file_size == expected_size {
                return parse_binary_stl(&mut file, num_triangles as usize);
            }
        }
    }

    // Fall back to ASCII STL
    parse_ascii_stl(path)
}

fn parse_binary_stl<R: Read>(
    reader: &mut R,
    num_triangles: usize,
) -> Result<ParsedContent, ParseError> {
    if num_triangles > MAX_TRIANGLES {
        return Err(ParseError::ParseFailed(format!(
            "Model exceeds {MAX_TRIANGLES} triangles limit ({num_triangles} found)"
        )));
    }

    // Each triangle: 12 floats (50 bytes total with 2 byte attribute)
    // 3 floats normal, 9 floats vertices, 2 bytes attr
    // We construct vertices buffer: [pos_x, pos_y, pos_z, norm_x, norm_y, norm_z]
    let mut vertices = Vec::with_capacity(num_triangles * 3 * 6);
    let mut indices = Vec::with_capacity(num_triangles * 3);
    let mut bounds = BoundingBox::new();

    let mut buf = vec![0u8; num_triangles * 50];
    reader
        .read_exact(&mut buf)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let mut idx = 0u32;
    for chunk in buf.as_chunks::<50>().0 {
        let n = [
            f32::from_le_bytes(chunk[0..4].try_into().unwrap()),
            f32::from_le_bytes(chunk[4..8].try_into().unwrap()),
            f32::from_le_bytes(chunk[8..12].try_into().unwrap()),
        ];

        let v1 = [
            f32::from_le_bytes(chunk[12..16].try_into().unwrap()),
            f32::from_le_bytes(chunk[16..20].try_into().unwrap()),
            f32::from_le_bytes(chunk[20..24].try_into().unwrap()),
        ];
        let v2 = [
            f32::from_le_bytes(chunk[24..28].try_into().unwrap()),
            f32::from_le_bytes(chunk[28..32].try_into().unwrap()),
            f32::from_le_bytes(chunk[32..36].try_into().unwrap()),
        ];
        let v3 = [
            f32::from_le_bytes(chunk[36..40].try_into().unwrap()),
            f32::from_le_bytes(chunk[40..44].try_into().unwrap()),
            f32::from_le_bytes(chunk[44..48].try_into().unwrap()),
        ];

        bounds.add_point(v1);
        bounds.add_point(v2);
        bounds.add_point(v3);

        // Compute normal if not present or zero
        let normal = if n[0] == 0.0 && n[1] == 0.0 && n[2] == 0.0 {
            normalize(cross(sub(v2, v1), sub(v3, v1)))
        } else {
            normalize(n)
        };

        for v in [v1, v2, v3] {
            vertices.extend_from_slice(&[v[0], v[1], v[2], normal[0], normal[1], normal[2]]);
            indices.push(idx);
            idx += 1;
        }
    }

    center_vertices(&mut vertices, &bounds);

    let triangle_count = num_triangles;
    let vertex_count = vertices.len() / 6;

    Ok(ParsedContent::Model3d(Model3dMesh {
        vertices: Arc::from(vertices),
        indices: Arc::from(indices),
        bounds,
        triangle_count,
        vertex_count,
    }))
}

fn parse_ascii_stl(path: &Path) -> Result<ParsedContent, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut bounds = BoundingBox::new();

    let mut current_normal = [0.0f32; 3];
    let mut current_tri_vertices = Vec::with_capacity(3);
    let mut idx = 0u32;
    let mut triangle_count = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.starts_with("facet normal") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                current_normal = [
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                    parts[4].parse().unwrap_or(0.0),
                ];
            }
            current_tri_vertices.clear();
        } else if trimmed.starts_with("vertex") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 4 {
                let v = [
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                ];
                bounds.add_point(v);
                current_tri_vertices.push(v);
            }
        } else if trimmed.starts_with("endfacet") && current_tri_vertices.len() == 3 {
            let v1 = current_tri_vertices[0];
            let v2 = current_tri_vertices[1];
            let v3 = current_tri_vertices[2];

            let normal =
                if current_normal[0] == 0.0 && current_normal[1] == 0.0 && current_normal[2] == 0.0
                {
                    normalize(cross(sub(v2, v1), sub(v3, v1)))
                } else {
                    normalize(current_normal)
                };

            for v in [v1, v2, v3] {
                vertices.extend_from_slice(&[v[0], v[1], v[2], normal[0], normal[1], normal[2]]);
                indices.push(idx);
                idx += 1;
            }
            triangle_count += 1;
            current_tri_vertices.clear();
        }
    }

    center_vertices(&mut vertices, &bounds);

    let vertex_count = vertices.len() / 6;

    if triangle_count > MAX_TRIANGLES {
        return Err(ParseError::ParseFailed(format!(
            "Model exceeds {MAX_TRIANGLES} triangles limit ({triangle_count} found)"
        )));
    }

    Ok(ParsedContent::Model3d(Model3dMesh {
        vertices: Arc::from(vertices),
        indices: Arc::from(indices),
        bounds,
        triangle_count,
        vertex_count,
    }))
}

pub fn parse_obj(path: &Path) -> Result<ParsedContent, ParseError> {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _) = tobj::load_obj(path, &load_options)
        .map_err(|e| ParseError::ParseFailed(format!("Failed to parse OBJ: {e}")))?;

    if models.is_empty() {
        return Err(ParseError::ParseFailed(
            "No models found in OBJ file".into(),
        ));
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut bounds = BoundingBox::new();
    let mut vertex_offset = 0u32;
    let mut total_triangles = 0;

    for model in &models {
        let mesh = &model.mesh;
        let has_normals = !mesh.normals.is_empty();

        let num_verts = mesh.positions.len() / 3;
        for i in 0..num_verts {
            let pos = [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ];
            bounds.add_point(pos);

            let normal = if has_normals && (i * 3 + 2) < mesh.normals.len() {
                normalize([
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ])
            } else {
                [0.0, 0.0, 0.0]
            };

            vertices.extend_from_slice(&[pos[0], pos[1], pos[2], normal[0], normal[1], normal[2]]);
        }

        // If mesh had no normals, calculate smooth/face normals across triangles
        if !has_normals {
            for chunk in mesh.indices.as_chunks::<3>().0 {
                let (i0, i1, i2) = (
                    (chunk[0] + vertex_offset) as usize,
                    (chunk[1] + vertex_offset) as usize,
                    (chunk[2] + vertex_offset) as usize,
                );

                let p0 = [vertices[i0 * 6], vertices[i0 * 6 + 1], vertices[i0 * 6 + 2]];
                let p1 = [vertices[i1 * 6], vertices[i1 * 6 + 1], vertices[i1 * 6 + 2]];
                let p2 = [vertices[i2 * 6], vertices[i2 * 6 + 1], vertices[i2 * 6 + 2]];

                let norm = normalize(cross(sub(p1, p0), sub(p2, p0)));
                for idx in [i0, i1, i2] {
                    vertices[idx * 6 + 3] += norm[0];
                    vertices[idx * 6 + 4] += norm[1];
                    vertices[idx * 6 + 5] += norm[2];
                }
            }

            // Normalize accumulated vertex normals
            for i in (vertex_offset as usize)..(vertex_offset as usize + num_verts) {
                let norm = normalize([
                    vertices[i * 6 + 3],
                    vertices[i * 6 + 4],
                    vertices[i * 6 + 5],
                ]);
                vertices[i * 6 + 3] = norm[0];
                vertices[i * 6 + 4] = norm[1];
                vertices[i * 6 + 5] = norm[2];
            }
        }

        for idx in &mesh.indices {
            indices.push(vertex_offset + idx);
        }

        vertex_offset += num_verts as u32;
        total_triangles += mesh.indices.len() / 3;
    }

    center_vertices(&mut vertices, &bounds);

    let vertex_count = vertices.len() / 6;

    if total_triangles > MAX_TRIANGLES {
        return Err(ParseError::ParseFailed(format!(
            "Model exceeds {MAX_TRIANGLES} triangles limit ({total_triangles} found)"
        )));
    }

    Ok(ParsedContent::Model3d(Model3dMesh {
        vertices: Arc::from(vertices),
        indices: Arc::from(indices),
        bounds,
        triangle_count: total_triangles,
        vertex_count,
    }))
}

fn center_vertices(vertices: &mut [f32], bounds: &BoundingBox) {
    if !bounds.is_valid() {
        return;
    }
    let c = bounds.center();
    for chunk in vertices.chunks_mut(6) {
        chunk[0] -= c[0];
        chunk[1] -= c[1];
        chunk[2] -= c[2];
    }
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
