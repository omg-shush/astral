use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};

#[derive(Default)]
pub struct TerrainPlanePlugin {}

impl Plugin for TerrainPlanePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
pub struct TerrainPlane {
    pub mesh: Handle<Mesh>
}

impl TerrainPlane {
    pub fn new(meshes: &mut ResMut<Assets<Mesh>>, heightmap: impl Fn(f32, f32) -> f32) -> TerrainPlane {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let (width, height) = (1000u32, 1000u32);
        let unit = 1.;

        let idx = |x, y| x * (height + 1) + y;

        let mut positions = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        let mut normals = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        let mut indices = Vec::with_capacity((width * height * 6) as usize);

        // Compute vertex positions & indices
        for xi in 0..=width {
            for yi in 0..=height {
                let (x, y) = ((xi as f32 - (width as f32/2.)) * unit, (yi as f32 - (height as f32/2.)) * unit);
                let attr_idx = idx(xi, yi) as usize;
                positions[attr_idx] = Vec3::new(x, heightmap(x, y), y);
                normals[attr_idx] = Vec3::Y; // TODO recompute
                if xi > 0 && yi > 0 {
                    // Draw quad between (xi-1, yi-1) and (xi, yi)
                    let xi_1_yi_1 = idx(xi-1, yi-1);
                    let xi_1_yi = idx(xi-1, yi);
                    let xi_yi_1 = idx(xi, yi-1);
                    let xi_yi = idx(xi, yi);
                    indices.append(&mut vec![xi_1_yi_1, xi_1_yi, xi_yi, xi_yi, xi_yi_1, xi_1_yi_1]);
                }
            }
        }

        // Compute normals
        for i in 0..indices.len()/3 {
            let (ai, bi, ci) = (indices[i*3] as usize, indices[i*3 + 1] as usize, indices[i*3 + 2] as usize);
            let (a, b, c) = (positions[ai], positions[bi], positions[ci]);
            let normal = (c - a).cross(a - b).normalize();
            normals[ai] += normal;
            normals[bi] += normal;
            normals[ci] += normal;
        }
        // Renormalize
        for i in 0..normals.len() {
            normals[i] = normals[i].normalize();
        }
        // Gaussian smooth
        let mut new_normals = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        for xi in 0..=width as i32 {
            for yi in 0..=height as i32 {
                let gaussian_kernel = [
                    (xi,   yi,   4.),
                    (xi,   yi-1, 2.),
                    (xi,   yi+1, 2.),
                    (xi-1, yi,   2.),
                    (xi-1, yi-1, 1.),
                    (xi-1, yi+1, 1.),
                    (xi+1, yi,   2.),
                    (xi+1, yi-1, 1.),
                    (xi+1, yi+1, 1.),
                ];
                new_normals[idx(xi as u32, yi as u32) as usize] = gaussian_kernel.into_iter()
                    .fold(Vec3::ZERO, |normal_acc, (x, y, weight)| {
                        let x = x.clamp(0, width as i32);
                        let y = y.clamp(0, height as i32);
                        normal_acc + weight * normals[idx(x as u32, y as u32) as usize]
                    }
                ).normalize();
            }
        }
        normals = new_normals;

        assert!(positions.len() == ((width+1) * (height+1)) as usize);
        assert!(normals.len() == ((width+1) * (height+1)) as usize);
        assert!(indices.len() == (width * height * 6) as usize);

        // Color overrides
        let mut colors = vec![Color::WHITE.as_rgba_f32(); ((width+1) * (height+1)) as usize];
        for i in 0..colors.len() {
            if positions[i].y < -16.5 {
                colors[i] = Color::SEA_GREEN.as_rgba_f32();
            } else if normals[i].y < 0.82 {
                colors[i] = Color::GRAY.as_rgba_f32();
            } else if positions[i].y < 24. {
                if normals[i].y > 0.98 {
                    colors[i] = Color::LIME_GREEN.as_rgba_f32();
                } else {
                    colors[i] = Color::GREEN.as_rgba_f32();
                }
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));
        let handle = meshes.add(mesh);
        TerrainPlane { mesh: handle }
    }
}
