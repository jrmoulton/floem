use std::collections::HashMap;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::{mem, num::NonZeroUsize};

use anyhow::Result;
use floem_renderer::cosmic_text::{TextLayout, FONT_SYSTEM};
use floem_renderer::{Img, Renderer};

use vello::glyph::Glyph;
use vello::kurbo::Stroke;
use vello::peniko::{
    kurbo::{Affine, Point, Rect, Shape},
    BrushRef, Color,
};
use vello::peniko::{Blob, Fill};
use vello::{AaConfig, RendererOptions, Scene};
use wgpu::{Backends, Device, DeviceType, Queue, Surface, SurfaceConfiguration, TextureFormat};

pub struct VgerRenderer {
    device: Arc<Device>,
    #[allow(unused)]
    queue: Arc<Queue>,
    surface: Surface<'static>,
    scene: vello::Scene,
    renderer: vello::Renderer,
    alt_vger: Option<vello::Scene>,
    config: SurfaceConfiguration,
    scale: f64,
    transform: Affine,
    capture: bool,
    font_cache: HashMap<floem_renderer::cosmic_text::fontdb::ID, vello::peniko::Font>,
}

impl VgerRenderer {
    pub fn new<W: wgpu::WindowHandle + 'static>(
        window: W,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(Backends::all()),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .ok_or_else(|| anyhow::anyhow!("can't get adapter"))?;

        if adapter.get_info().device_type == DeviceType::Cpu {
            return Err(anyhow::anyhow!("only cpu adapter found"));
        }

        let mut required_downlevel_flags = wgpu::DownlevelFlags::empty();
        required_downlevel_flags.set(wgpu::DownlevelFlags::VERTEX_STORAGE, true);

        if !adapter
            .get_downlevel_capabilities()
            .flags
            .contains(required_downlevel_flags)
        {
            return Err(anyhow::anyhow!(
                "adapter doesn't support required downlevel flags"
            ));
        }

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                ..Default::default()
            },
            None,
        ))?;
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let texture_format = surface_caps
            .formats
            .into_iter()
            .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
            .ok_or_else(|| anyhow::anyhow!("surface should support Rgba8Unorm or Bgra8Unorm"))?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let vger = vello::Scene::new();
        let renderer = vello::Renderer::new(
            &device.clone(),
            RendererOptions {
                surface_format: Some(texture_format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .unwrap();

        Ok(Self {
            device,
            queue,
            surface,
            scene: vger,
            renderer,
            alt_vger: None,
            scale,
            config,
            transform: Affine::IDENTITY,
            capture: false,
            font_cache: HashMap::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32, scale: f64) {
        if width != self.config.width || height != self.config.height {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
        self.scale = scale;
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }
}

impl Renderer for VgerRenderer {
    fn begin(&mut self, capture: bool) {
        // Switch to the capture Vger if needed
        if self.capture != capture {
            self.capture = capture;
            if self.alt_vger.is_none() {
                self.alt_vger = Some(Scene::new());
            }
            mem::swap(&mut self.scene, self.alt_vger.as_mut().unwrap())
        };
    }

    fn stroke<'b>(&mut self, shape: &impl Shape, brush: impl Into<BrushRef<'b>>, width: f64) {
        let width = (width * self.scale).round();
        self.scene.stroke(
            &Stroke::new(width),
            self.transform.then_scale(self.scale),
            brush,
            Some(Affine::IDENTITY.then_scale(0.1)),
            shape,
        );
    }

    fn fill<'b>(&mut self, path: &impl Shape, brush: impl Into<BrushRef<'b>>, _blur_radius: f64) {
        self.scene.fill(
            vello::peniko::Fill::EvenOdd,
            self.transform.then_scale(self.scale),
            brush,
            None,
            path,
        );
    }

    fn draw_text(&mut self, layout: &TextLayout, pos: impl Into<Point>) {
        let offset = self.transform.translation();
        let pos: Point = pos.into();
        for line in layout.layout_runs() {
            // Create a peekable iterator
            let mut glyph_runs = line.glyphs.iter().peekable();

            // Iterate through glyphs with segmentation
            while let Some(start_glyph) = glyph_runs.peek() {
                // Determine the initial properties for comparison
                let start_color = start_glyph.color;
                let start_font_size = start_glyph.font_size;
                let start_font_id = start_glyph.cache_key.font_id;
                let font = self
                    .font_cache
                    .get(&start_font_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        dbg!("creating font");
                        let font = FONT_SYSTEM.get_font(start_font_id).unwrap();
                        let font = vello::peniko::Font::new(Blob::new(font.arc_data().clone()), 0);
                        self.font_cache.insert(start_font_id, font.clone());
                        font
                    });
                // let axes = font_ref.axes();
                // let var_loc = axes.location(variations.iter().copied());
                self.scene
                    .draw_glyphs(&font)
                    .font_size(start_font_size)
                    // .normalized_coords(var_loc.coords())
                    .brush(BrushRef::Solid(start_color))
                    .hint(false)
                    .glyph_transform(Some(Affine::IDENTITY.then_scale(self.scale)))
                    .draw(
                        Fill::NonZero,
                        glyph_runs
                            .by_ref()
                            .take_while(|glyph| {
                                glyph.color == start_color
                                    && glyph.font_size == start_font_size
                                    && glyph.cache_key.font_id == start_font_id
                            })
                            .map(|glyph| {
                                let x = glyph.x + pos.x as f32 + offset.x as f32;
                                let y = line.line_y + pos.y as f32 + offset.y as f32;
                                let glyph_x = x * self.scale as f32;

                                let glyph_y = (y * self.scale as f32).round();
                                Glyph {
                                    id: glyph.cache_key.glyph_id as u32,
                                    x: glyph_x,
                                    y: glyph_y,
                                }
                            }),
                    );
            }
        }
    }

    fn draw_img(&mut self, img: Img<'_>, rect: Rect) {
        let width = (rect.width() * self.scale).round() as u32;
        let height = (rect.height() * self.scale).round() as u32;
        let width = width.max(1);
        let height = height.max(1);

        let scale =
            (width as f32 / img.img.width as f32).min(height as f32 / img.img.height as f32);

        self.scene.draw_image(
            &img.img,
            self.transform
                .with_translation(rect.origin().to_vec2())
                .then_scale(scale as f64),
        );
    }

    fn draw_svg<'b>(
        &mut self,
        svg: floem_renderer::Svg<'b>,
        rect: Rect,
        brush: Option<impl Into<BrushRef<'b>>>,
    ) {
        let width = (rect.width() * self.scale).round() as u32;
        let height = (rect.height() * self.scale).round() as u32;
        let width = width.max(1);
        let height = height.max(1);

        let scale =
            (width as f32 / svg.tree.size().width()).min(height as f32 / svg.tree.size().height());

        let mut scene = Scene::new();
        vello_svg::render_tree(&mut scene, svg.tree);
        self.scene.append(
            &scene,
            Some(
                self.transform
                    .with_translation(rect.origin().to_vec2())
                    .then_scale(scale as f64),
            ),
        )
    }

    fn transform(&mut self, transform: Affine) {
        self.transform = transform;
    }

    fn set_z_index(&mut self, _z_index: i32) {}

    fn clip(&mut self, shape: &impl Shape) {
        self.scene.push_layer(
            // default blend is clip
            vello::peniko::BlendMode::default(),
            1.0,
            Affine::IDENTITY,
            shape,
        );
    }

    fn clear_clip(&mut self) {
        self.scene.pop_layer();
    }

    fn finish(&mut self) -> Option<vello::peniko::Image> {
        if self.capture {
            self.render_image()
        } else {
            if let Ok(frame) = self.surface.get_current_texture() {
                self.renderer
                    .render_to_surface(
                        &self.device.clone(),
                        &self.queue,
                        &self.scene,
                        &frame,
                        &vello::RenderParams {
                            base_color: Color::BLACK, // Background color
                            width: self.config.width * self.scale as u32,
                            height: self.config.height * self.scale as u32,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .unwrap();
                frame.present();
                self.scene.reset();
            }
            None
        }
    }
}

impl VgerRenderer {
    fn render_image(&mut self) -> Option<vello::peniko::Image> {
        let width_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1;
        let width = (self.config.width + width_align) & !width_align;
        let height = self.config.height;
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.config.width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: Some("render_texture"),
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        };
        let texture = self.device.create_texture(&texture_desc);

        self.renderer
            .render_to_surface(
                &self.device.clone(),
                &self.queue,
                &self.scene,
                &self.surface.get_current_texture().unwrap(),
                &vello::RenderParams {
                    base_color: Color::BLACK, // Background color
                    width: self.config.width * self.scale as u32,
                    height: self.config.height * self.scale as u32,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .unwrap();

        let bytes_per_pixel = 4;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (width as u64 * height as u64 * bytes_per_pixel),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bytes_per_row = width * bytes_per_pixel as u32;
        assert!(bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            texture_desc.size,
        );
        let command_buffer = encoder.finish();
        self.queue.submit(Some(command_buffer));
        self.device.poll(wgpu::Maintain::Wait);

        let slice = buffer.slice(..);
        let (tx, rx) = sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());

        loop {
            if let Ok(r) = rx.try_recv() {
                break r.ok()?;
            }
            if let wgpu::MaintainResult::Ok = self.device.poll(wgpu::MaintainBase::Wait) {
                rx.recv().ok()?.ok()?;
                break;
            }
        }

        let mut cropped_buffer = Vec::new();
        let buffer: Vec<u8> = slice.get_mapped_range().to_owned();

        let mut cursor = 0;
        let row_size = self.config.width as usize * bytes_per_pixel as usize;
        for _ in 0..height {
            cropped_buffer.extend_from_slice(&buffer[cursor..(cursor + row_size)]);
            cursor += bytes_per_row as usize;
        }

        Some(vello::peniko::Image::new(
            Blob::new(Arc::new(cropped_buffer)),
            vello::peniko::Format::Rgba8,
            self.config.width,
            height,
        ))
    }
}
