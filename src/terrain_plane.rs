use std::f32::consts::{PI, E};

use bevy::{prelude::*, render::{render_resource::{PrimitiveTopology, ShaderRef, AsBindGroup}, mesh::Indices}, reflect::TypeUuid};
use bevy_inspector_egui::{quick::AssetInspectorPlugin, InspectorOptions};

#[derive(Default)]
pub struct TerrainPlanePlugin {}

impl Plugin for TerrainPlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainPlaneMaterial>::default());
        app.add_plugins(AssetInspectorPlugin::<TerrainPlaneMaterial>::default());
    }
}

#[derive(Component)]
pub struct TerrainPlane {
    pub mesh: Handle<Mesh>,
    pub material: Handle<TerrainPlaneMaterial>
}

impl TerrainPlane {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<TerrainPlaneMaterial>, heightmap: impl Fn(f32, f32) -> f32) -> TerrainPlane {
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
        // Gaussian smooth normals
        // Make kernel
        let gauss_radius = 4;
        let sigma = 3.;
        let gaussian = |x: f32, y: f32| {
            let coeff = 1. / (2. * PI * sigma * sigma);
            let exp = -(x * x + y * y) / (2. * sigma * sigma);
            coeff * E.powf(exp)
        };
        let mut gaussian_kernel = Vec::new();
        for dx in -gauss_radius..=gauss_radius {
            for dy in -gauss_radius..=gauss_radius {
                gaussian_kernel.push((dx, dy, gaussian(dx as f32, dy as f32)));
            }
        }
        // Convolution
        let mut new_normals = vec![Vec3::ZERO; ((width+1) * (height+1)) as usize];
        for xi in 0..=width as i32 {
            for yi in 0..=height as i32 {
                new_normals[idx(xi as u32, yi as u32) as usize] = gaussian_kernel.iter()
                    .fold(Vec3::ZERO, |normal_acc, &(x, y, weight)| {
                        let x = (x + xi).clamp(0, width as i32);
                        let y = (y + yi).clamp(0, height as i32);
                        normal_acc + weight * normals[idx(x as u32, y as u32) as usize]
                    }
                ).normalize();
            }
        }
        normals = new_normals;

        assert!(positions.len() == ((width+1) * (height+1)) as usize);
        assert!(normals.len() == ((width+1) * (height+1)) as usize);
        assert!(indices.len() == (width * height * 6) as usize);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(indices)));

        let mat: Handle<TerrainPlaneMaterial> = materials.add(TerrainPlaneMaterial {
            peak_color: Color::WHITE,
            flat_color: Color::rgb(0.0, 1.0, 0.0),
            steep_color: Color::rgb(0.06666667, 0.6666667, 0.18431373),
            cliff_color: Color::rgb(0.0, 0.22745098, 0.015686275),
            sea_color: Color::rgb(0.18, 0.55, 0.34),
            peak_thresh: 24.0,
            cliff_thresh: 0.92,
            steep_thresh: 0.6,
            sea_thresh: -14.5,
            steep_interp: 3.1,
            cliff_interp: 2.3,
            light_direction: Vec3::new(-5.0, -3.0, -8.0).normalize(),
            diffuse_color: Color::rgb(0.9098039, 0.77254903, 0.3137255),
            diffuse_strength: 1.0,
            ambient_color: Color::WHITE,
            ambient_strength: 0.1
        });

        TerrainPlane { mesh: meshes.add(mesh), material: mat }
    }
}

#[derive(TypeUuid, Clone, AsBindGroup, Reflect, InspectorOptions, Debug)]
#[uuid="c2ad0a24-0ccd-498e-9162-8d5854e51d8a"]
pub struct TerrainPlaneMaterial {
    #[uniform(0)]
    peak_color: Color,
    #[uniform(0)]
    flat_color: Color,
    #[uniform(0)]
    steep_color: Color,
    #[uniform(0)]
    cliff_color: Color,
    #[uniform(0)]
    sea_color: Color,
    #[uniform(0)]
    #[inspector(min = -64.0, max = 64.0)]
    peak_thresh: f32,
    #[uniform(0)]
    #[inspector(min = 0.0, max = 1.0)]
    cliff_thresh: f32,
    #[uniform(0)]
    #[inspector(min = 0.0, max = 1.0)]
    steep_thresh: f32,
    #[uniform(0)]
    #[inspector(min = -64.0, max = 64.0)]
    sea_thresh: f32,
    #[uniform(0)]
    #[inspector(min = 0.0, max = 100.0)]
    steep_interp: f32,
    #[uniform(0)]
    #[inspector(min = 0.0, max = 100.0)]
    cliff_interp: f32,

    #[uniform(1)]
    light_direction: Vec3,
    #[uniform(1)]
    diffuse_color: Color,
    #[uniform(1)]
    #[inspector(min = 0.0, max = 1.0)]
    diffuse_strength: f32,
    #[uniform(1)]
    ambient_color: Color,
    #[uniform(1)]
    #[inspector(min = 0.0, max = 1.0)]
    ambient_strength: f32
}

impl Material for TerrainPlaneMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        ShaderRef::Path("shaders/terrain_plane.vert".into())
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        ShaderRef::Path("shaders/terrain_plane.frag".into())
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        *descriptor.vertex.entry_point.to_mut() = "main".to_string();
        *descriptor.fragment.as_mut().unwrap().entry_point.to_mut() = "main".to_string();
        Ok(())
    }
}
