/*
    Example program to show how to use three-d with eframe.

    Code adapted from:
    https://github.com/emilk/egui/blob/08fb447fb55293b2d49343cf5ade2c59d436bc58/examples/custom_3d_glow/src/main.rs
    https://github.com/asny/three-d/blob/0e338e3ccea8ea4187397803eafb8e7f894e0a77/examples/triangle/src/main.rs
    https://github.com/emilk/egui/pull/1407
*/
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release builds.

use std::sync::Arc;

use eframe::{egui, egui::mutex::Mutex, egui_glow, egui_glow::glow};

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(535.0, 570.0)),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using three-d",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

struct MyApp {
    custom_3d: Arc<Mutex<Custom3d>>,
    angle: f32,
}

impl MyApp {
    pub fn new(cc : &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.as_ref().expect("You need to run eframe with the glow backend!");
        Self {
            custom_3d: Arc::new(Mutex::new(Custom3d::new(gl))),
            angle: 0.0
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("three-d", "https://github.com/asny/three-d");
                ui.label(", a 3D rendering library for Rust.")
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(512.0), egui::Sense::drag());

        self.angle += response.drag_delta().x * 0.01;

        let angle = self.angle;
        let custom_3d = self.custom_3d.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |info, _painter| {
                custom_3d.lock().paint(&info, angle);
            })),
        };
        ui.painter().add(callback);
    }
}

struct Custom3d {
    three_d: three_d::Context,
    camera: three_d::Camera,
    model: three_d::Gm<three_d::Mesh, three_d::ColorMaterial>,
}

impl Custom3d {
    fn new(gl: &Arc<glow::Context>) -> Self {
        use three_d::*;

        let three_d = Context::from_gl_context(gl.clone()).unwrap();

        let positions = vec![
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ];
        let colors = vec![
            Srgba::new(255, 0, 0, 255), // bottom right
            Srgba::new(0, 255, 0, 255), // bottom left
            Srgba::new(0, 0, 255, 255), // top
        ];
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            colors: Some(colors),
            ..Default::default()
        };

        // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
        let model = Gm::new(Mesh::new(&three_d, &cpu_mesh), ColorMaterial::default());

        Self {
            three_d: three_d::Context::from_gl_context(gl.clone()).unwrap(),
            camera: Camera::new_perspective(
                Viewport {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                vec3(0.0, 0.0, 2.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                degrees(45.0),
                0.1,
                10.0,
            ),
            model,
        }
    }

    fn paint(&mut self, info: &egui::PaintCallbackInfo, angle: f32) {
        use three_d::*;

        let _three_d = &self.three_d;
            
        let viewport_pixels = info.viewport_in_pixels();

        let viewport = Viewport {
                x: viewport_pixels.left_px.round() as _,
                y: viewport_pixels.from_bottom_px.round() as _,
                width: viewport_pixels.width_px.round() as _,
                height: viewport_pixels.height_px.round() as _,
        };

        //We need to update the viewport each frame to ensure three-d is actually rendering inside the Canvas each time.
        self.camera.set_viewport(viewport);

        // Set the current transformation of the triangle
        self.model.set_transformation(Mat4::from_angle_y(radians(angle)));

        // Render the triangle with the color material which uses the per vertex colors defined at construction
        self.model.render(&self.camera, &[]);
    }
}
