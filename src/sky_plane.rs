use bevy::{prelude::*, render::{render_resource::{ShaderRef, AsBindGroup, TextureDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages, SamplerDescriptor, AddressMode, FilterMode, TextureViewDescriptor, TextureViewDimension, TextureAspect}, texture::ImageSampler}, reflect::TypeUuid};
use bevy_inspector_egui::{quick::AssetInspectorPlugin, InspectorOptions, prelude::ReflectInspectorOptions};

use crate::perlin_3d;

#[derive(Default)]
pub struct SkyPlanePlugin {}

impl Plugin for SkyPlanePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkyPlaneMaterial>();
        app.add_asset::<SkyPlaneMaterial>();
        app.add_plugins(MaterialPlugin::<SkyPlaneMaterial>::default());
        app.add_plugins(AssetInspectorPlugin::<SkyPlaneMaterial>::default());
        app.add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct SkyPlane {
    pub mesh: Handle<Mesh>,
    pub material: Handle<SkyPlaneMaterial>
}

impl SkyPlane {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<SkyPlaneMaterial>, images: &mut Assets<Image>) -> SkyPlane {
        let width = 2048.0;
        let corner = Vec3::new(width/2.0, 200.0, width/2.0);
        let mesh = shape::Box::from_corners(corner, -corner).into();

        let perlin_size = 256;
        let perlin_detail = 2.;
        let perlin_func = perlin_3d((perlin_size as f32 * perlin_detail) as usize);
        let layered_perlin_func = |x: f32, y: f32, z: f32| {
            let a = perlin_func(x / 3.0, y / 3.0, z / 3.0) * 1.0;
            let b = perlin_func(x / 13.0, y / 13.0, z / 13.0) * 4.0;
            let c = perlin_func(x / 43.0, y / 43.0, z / 43.0) * 16.0;
            let d = perlin_func(x / 197.0, y / 197.0, z / 197.0) * 64.0;
            let e = -(y - 128.0).powi(2) / 16384.0;
            a + b + c + d + e
        };

        // Generate noise texture with fixed detail
        let mut perlin_data = vec![0; perlin_size * perlin_size * perlin_size * 4]; // 4 bytes per float
        for z in 0..perlin_size {
            for y in 0..perlin_size {
                for x in 0..perlin_size {
                    let cell_idx = x + y * perlin_size + z * perlin_size * perlin_size;
                    let perlin = layered_perlin_func(x as f32 * perlin_detail, y as f32 * perlin_detail, z as f32 * perlin_detail);
                    let data = perlin.to_ne_bytes();
                    perlin_data[4 * cell_idx + 0] = data[0];
                    perlin_data[4 * cell_idx + 1] = data[1];
                    perlin_data[4 * cell_idx + 2] = data[2];
                    perlin_data[4 * cell_idx + 3] = data[3];
                }
            }
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
                format: TextureFormat::R32Float,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            sampler_descriptor: ImageSampler::Descriptor(SamplerDescriptor {
                label: "3D Perlin Noise Sampler".into(),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                lod_min_clamp: 1.0,
                lod_max_clamp: 1.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None
            }),
            texture_view_descriptor: Some(TextureViewDescriptor {
                label: "3D Perlin Noise View".into(),
                format: Some(TextureFormat::R32Float),
                dimension: Some(TextureViewDimension::D3),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }),
        };
        let img_handle = images.add(image);
        let mat: Handle<SkyPlaneMaterial> = materials.add(SkyPlaneMaterial {
            noise_3d: img_handle,
            step_size: 1.0,
            noise_size: 1.0,
            noise_scale: 0.03,
            noise_scroll: 0.0,
            noise_bias: 0.0,
            noise_thresh: 10.0,
            step_count: 200,
            camera_pos: Vec3::ZERO
        });

        SkyPlane { mesh: meshes.add(mesh), material: mat }
    }
}

#[derive(TypeUuid, Clone, AsBindGroup, Reflect, InspectorOptions, Resource, Default, Debug)]
#[reflect(InspectorOptions, Resource)]
#[uuid="19f0b5b0-9ef1-414b-89b5-dcf4add69384"]
pub struct SkyPlaneMaterial {
    #[texture(0, dimension = "3d")]
    #[sampler(1)]
    noise_3d: Handle<Image>,

    #[uniform(2)]
    #[inspector(speed = 0.01)]
    step_size: f32,
    #[uniform(2)]
    #[inspector(speed = 0.01)]
    noise_size: f32,
    #[uniform(2)]
    #[inspector(speed = 0.01)]
    noise_scale: f32,
    #[uniform(2)]
    #[inspector(speed = 0.01)]
    noise_scroll: f32,
    #[uniform(2)]
    #[inspector(speed = 0.01)]
    noise_bias: f32,
    #[uniform(2)]
    #[inspector(speed = 0.01)]
    noise_thresh: f32,
    #[uniform(2)]
    step_count: u32,
    #[uniform(2)]
    camera_pos: Vec3
}

impl Material for SkyPlaneMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        ShaderRef::Path("shaders/sky_plane.vert".into())
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        ShaderRef::Path("shaders/sky_plane.frag".into())
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
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

fn update(mut assets: ResMut<Assets<SkyPlaneMaterial>>, camera: Query<&Transform, With<Camera3d>>) {
    let camera_pos = camera.single().translation;
    assets.iter_mut().for_each(|(_, mat)| {
        mat.camera_pos = camera_pos;
    });
}
