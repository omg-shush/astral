use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, core::Zeroable};

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
    pub fn new(mut meshes: ResMut<Assets<Mesh>>) -> TerrainPlane {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let (width, height) = (100u16, 100u16);
        let unit = 1.;

        let mut positions = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        let mut normals = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        let mut indices = Vec::with_capacity((width * height * 6) as usize);

        let idx = |x, y| x * (height + 1) + y;

        // Compute vertex positions & indices
        for xi in 0..=width {
            for yi in 0..=height {
                let (x, y) = ((xi as f32 - (width as f32/2.)) * unit, (yi as f32 - (height as f32/2.)) * unit);
                let attr_idx = idx(xi, yi) as usize;
                positions[attr_idx] = Vec3::new(x, (x*x+y).sin() / 4., y);
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

        assert!(positions.len() == ((width+1) * (height+1)) as usize);
        assert!(normals.len() == ((width+1) * (height+1)) as usize);
        assert!(indices.len() == (width * height * 6) as usize);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U16(indices)));
        let handle = meshes.add(mesh);
        TerrainPlane { mesh: handle }
    }    
}
