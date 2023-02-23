use std::io::{BufRead, BufReader};

use log::{error, warn};
use mint::{Point2, Point3};
use reerror::{conversions::invalid_argument, throw, Result, StatusCode};

use crate::{formats::Format, Path};

use super::{Mesh, Scene};

/// File format definition for Wavefront obj files
#[derive(Clone, Copy)]
pub struct ObjMesh;

impl Format for ObjMesh {
    type Output = Mesh<f32>;

    fn parse(&self, path: &Path) -> Result<Self::Output> {
        let mut scene = (ObjScene).parse(path)?;
        Ok(scene.nodes.pop().unwrap().0)
    }
}

/// File format definition for Wavefront obj files
pub struct ObjScene;

impl Format for ObjScene {
    type Output = Scene<f32>;
    fn parse(&self, path: &Path) -> Result<Self::Output> {
        let reader = BufReader::new(path.reader()?);

        let mut verts: Vec<Point3<f32>> = vec![];
        let mut normals: Vec<Point3<f32>> = vec![];
        let mut uvs: Vec<Point2<f32>> = vec![];
        let mut indices: Vec<(usize, Option<usize>, Option<usize>)> = vec![];
        let mut scene: Vec<(Mesh<f32>, String)> = vec![];
        let mut cur_obj: Option<String> = None;

        for (n, line) in reader.lines().enumerate() {
            let n = n + 1; // files usually aren't 0 indexed
            let line = throw!(line, "Failed to parse line {n}");

            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens[..] {
                ["#", ..] => { /* do nothing this is a comment */ }
                ["v", x, y, z] => verts.push(Point3 {
                    x: throw!(x.parse(), "parsing x coord of vertex on line {n}: '{x}'"),
                    y: throw!(y.parse(), "parsing y coord of vertex on line {n}: '{y}'"),
                    z: throw!(z.parse(), "parsing z coord of vertex on line {n}: '{z}'"),
                }),
                ["vt", u, v] => uvs.push(Point2 {
                    x: throw!(u.parse(), "parsing u coord of vertex on line {n}: '{u}'"),
                    y: throw!(v.parse(), "parsing v coord of vertex on line {n}: '{v}'"),
                }),
                ["vn", x, y, z] => normals.push(Point3 {
                    x: throw!(x.parse(), "parsing x coord of normal on line {n}: '{x}'"),
                    y: throw!(y.parse(), "parsing y coord of normal on line {n}: '{y}'"),
                    z: throw!(z.parse(), "parsing z coord of normal on line {n}: '{z}'"),
                }),
                ["o", name] => {
                    if let Some(name) = cur_obj {
                        // do some validation of the parsed data
                        if indices.len() % 3 != 0 {
                            warn!("object does not have a valid number of indices ({}), expected a multiple of 3", indices.len());
                        }
                        if verts.len() > normals.len() {
                            warn!("found {} vertices and {} normals, some vertices will be missing normals", verts.len(), normals.len());
                        }
                        if verts.len() > uvs.len() {
                            warn!("found {} vertices and {} uv coordinates, some vertices will be missing uv coords", verts.len(), uvs.len());
                        }

                        // build a mesh from parsed data
                        let mut mesh = Mesh::default();
                        for (v, uv, norm) in &indices {
                            mesh.verts.push(*throw!(verts.get(*v - 1),
                                if none StatusCode::OutOfRange,
                                "on line {n} vertex index  '{v}' max value is {}",
                                verts.len()
                            ));
                            if let Some(norm) = norm {
                                mesh.normals.push(*throw!(normals.get(*norm - 1),
                                    if none StatusCode::OutOfRange,
                                    "on line {n} normal index '{norm}' max value is {}",
                                    normals.len()
                                ));
                            }
                            if let Some(uv) = uv {
                                mesh.uvs.push(*throw!(uvs.get(*uv - 1),
                                    if none StatusCode::OutOfRange,
                                    "on line {n} uv index '{uv}' max value is {}",
                                    uvs.len()
                                ));
                            }
                        }

                        // add mesh to scene
                        scene.push((mesh, name.to_string()));

                        // clear the current info
                        indices.clear();
                        verts.clear();
                        normals.clear();
                        uvs.clear();
                    }
                    // record the name of the last object
                    cur_obj = Some(name.to_string());
                }
                ["f", a, b, c] => {
                    let mut parse_index = |index: &str| {
                        let tokens: Vec<_> = index
                            .split('/')
                            .map(|s| {
                                if s.is_empty() {
                                    Ok(None)
                                } else {
                                    Ok(Some(throw!(
                                        s.parse::<usize>(),
                                        "parsing face index on line {n}: {index}"
                                    )))
                                }
                            })
                            .map(|r| r.transpose())
                            .collect();
                        match &tokens[..] {
                            [Some(v)] => indices.push((v.clone()?, None, None)),
                            [Some(v), uv] => {
                                indices.push((v.clone()?, uv.clone().transpose()?, None))
                            }
                            [Some(v), uv, norm] => indices.push((
                                v.clone()?,
                                uv.clone().transpose()?,
                                norm.clone().transpose()?,
                            )),
                            [None, ..] => {
                                return Err(invalid_argument(
                                    "Missing geometry index while parsing .obj file",
                                ));
                            }
                            _ => {
                                return Err(invalid_argument(
                                    "Unrecognized index format: '{index}'",
                                ))
                            }
                        };
                        Ok(())
                    };
                    (parse_index)(a)?;
                    (parse_index)(b)?;
                    (parse_index)(c)?;
                }
                _ => error!("Unrecognized .obj command '{line}'"),
            }
        }

        // record the last mesh since it won't have an `o` tag
        // build a mesh from parsed data
        let mut mesh = Mesh::default();
        for (v, uv, norm) in &indices {
            mesh.verts.push(verts[*v - 1]);
            if let Some(norm) = norm {
                mesh.normals.push(normals[*norm - 1]);
            }
            if let Some(uv) = uv {
                mesh.uvs.push(uvs[*uv - 1]);
            }
        }

        // add mesh to scene
        scene.push((mesh, cur_obj.unwrap_or_else(|| String::from("<anonymous>"))));

        let mut out_scene = Scene::default();
        for elem in scene {
            out_scene.nodes.push(elem);
        }

        Ok(out_scene)
    }
}
