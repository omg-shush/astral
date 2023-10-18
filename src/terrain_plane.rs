use std::f32::consts::{PI, E};

use bevy::{prelude::*, render::{render_resource::{PrimitiveTopology, ShaderRef, AsBindGroup, TextureDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages, SamplerDescriptor, AddressMode, FilterMode, TextureViewDescriptor, TextureViewDimension, TextureAspect}, mesh::Indices, texture::ImageSampler}, reflect::TypeUuid, math::Vec3Swizzles};
use bevy_inspector_egui::{quick::AssetInspectorPlugin, InspectorOptions, prelude::ReflectInspectorOptions};

#[derive(Default)]
pub struct TerrainPlanePlugin {}

impl Plugin for TerrainPlanePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TerrainPlaneMaterial>();
        app.add_asset::<TerrainPlaneMaterial>();
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
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<TerrainPlaneMaterial>, images: &mut Assets<Image>, heightmap: impl Fn(f32, f32) -> f32) -> TerrainPlane {
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

        let perlin_size = 64;
        let mut perlin_data = vec![0; perlin_size * perlin_size * perlin_size * 4 * 4]; // 4 floats per Vec4, 4 bytes per float
        for _ in 0..perlin_size {
            let (dx, dy, dz) = rand::random();
            let vec = Vec3::new(dx, dy, dz).normalize().xyzz();
            perlin_data.append(&mut vec.to_array().iter().flat_map(|x| x.to_ne_bytes()).collect::<Vec<u8>>());
        }
        let image = Image {
            data: perlin_data,
            texture_descriptor: TextureDescriptor {
                label: "3D Perlin Noise Texture".into(),
                size: Extent3d {
                    width: perlin_size as u32,
                    height: perlin_size as u32,
                    depth_or_array_layers: perlin_size as u32
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            sampler_descriptor: ImageSampler::Descriptor(SamplerDescriptor {
                label: "3D Perlin Noise Sampler".into(),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 1.0,
                lod_max_clamp: 1.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None
            }),
            texture_view_descriptor: Some(TextureViewDescriptor {
                label: "3D Perlin Noise View".into(),
                format: Some(TextureFormat::Rgba32Float),
                dimension: Some(TextureViewDimension::D3),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }),
        };
        let img_handle = images.add(image);

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
            ambient_strength: 0.1,
            noise_3d: img_handle
        });

        TerrainPlane { mesh: meshes.add(mesh), material: mat }
    }
}

#[derive(TypeUuid, Clone, AsBindGroup, Reflect, InspectorOptions, Resource, Default, Debug)]
#[reflect(InspectorOptions, Resource)]
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
    ambient_strength: f32,

    #[texture(2, dimension = "3d")]
    #[sampler(3)]
    noise_3d: Handle<Image>
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
