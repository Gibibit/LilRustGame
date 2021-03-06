#[macro_use] extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;
extern crate rand;
extern crate image;
extern crate cgmath;

use gfx::traits::FactoryExt;
use gfx::Device;
use cgmath::{Deg, Vector3, Point3, Matrix4};
use cgmath::prelude::*;
use cgmath::BaseNum;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;
const SCREEN_FWIDTH: f32 = SCREEN_WIDTH as f32;
const SCREEN_FHEIGHT: f32 = SCREEN_HEIGHT as f32;

const SCROLL_SPEED: f32 = 15.0;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Locals {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    }
}

/*
pub trait ToArr {
    type Output;
    fn to_arr(&self) -> Self::Output;
}

impl<T: BaseNum> ToArr for Matrix4<T> {
    type Output = [[T; 4]; 4];
    fn to_arr(&self) -> Self::Output {
        (*self).into()
    }
}*/

/*
fn clamp<T>(value: T, min: T, max: T) -> T where T: std::cmp::Ord { value.max(min).min(max) }

fn fclamp(value: f32, min: f32, max: f32) -> f32 { value.max(min).min(max) }

fn load_texture<F, R>(factory: &mut F, path: &str) ->
	gfx::handle::ShaderResourceView<R, [f32; 4]> where F: gfx::Factory<R>, R: gfx::Resources
{
    let img = image::open(path).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(kind, &[&img]).unwrap();
    view
}*/

#[derive(Debug, Clone, Copy)]
struct Rectangle {
	pub pos: (f32, f32),
	pub size: (f32, f32),
	pub color: [f32; 3]
}

#[derive(Debug)]
struct Pseudocube {
	squares: Vec<Rectangle>,
	offset: (f32, f32)
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
    Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
    Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] }
];

impl Pseudocube {
	pub fn new() -> Self {
		Pseudocube {
			squares: vec![],
			offset: (0.0, 0.0),
		}
	}

	pub fn add_square(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 3]) {
		let sq = Rectangle {
			pos: (x, y),
			size: (width, height), color
		};

		self.squares.push(sq);
	}

	pub fn get_vertices_indices(&self) -> (Vec<Vertex>, Vec<u16>) {
		let (mut vs, mut is) = (vec![], vec![]);

		for(i, sq) in self.squares.iter()
			.enumerate()
		{
			let pos = (sq.pos.0/SCREEN_FWIDTH, sq.pos.1/SCREEN_FHEIGHT);
			let size = (sq.size.0/SCREEN_FWIDTH, sq.size.1/SCREEN_FHEIGHT);
			let i = i as u16;

			let x1 = pos.0*2.0 - 1.0 + 2.0*self.offset.0/SCREEN_FWIDTH;
			let y1 = -pos.1*2.0 + 1.0 - 2.0*self.offset.1/SCREEN_FHEIGHT;
			let x2 = x1 + size.0*2.0;
			let y2 = y1 - size.1*2.0;

			vs.extend(&[
				Vertex { pos: [x2, y1], color: sq.color },
                Vertex { pos: [x1, y1], color: sq.color },
                Vertex { pos: [x1, y2], color: sq.color },
                Vertex { pos: [x2, y2], color: sq.color },
			]);
			is.extend(&[
				4*i, 4*i + 1, 4*i + 2, 4*i + 2, 4*i + 3, 4*i
			]);
		}

		(vs, is)
	}

	pub fn tick(&mut self) {

	}

	pub fn add_offset(&mut self, xoffset: f32, yoffset: f32) {
		let offset = (self.offset.0 + xoffset, self.offset.1 + yoffset);
		self.offset = offset;
	}
}

pub fn main() {
	let mut cube = Pseudocube::new();
	cube.add_square(50.0, 50.0, 100.0, 50.0, rand::random());

	let mut model = Matrix4::identity();
	let view = Matrix4::look_at(
		Point3::new(2.0, 2.0, 1.0),
		Point3::new(0.0, 0.0, 0.0),
		Vector3::unit_z(),
	).into();
	let projection = cgmath::perspective(Deg(60.0f32), 1.3, 0.1, 1000.0).into();

    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Rectangle Toy".to_string())
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
    window.set_inner_size(SCREEN_WIDTH, SCREEN_HEIGHT);

	//let (vertices, indices) = cube.get_vertices_indices();
	let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
	    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslv")),
	    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslf")),
	    pipe::new()
	).unwrap();

	let locals_buffer = factory.create_constant_buffer(1);

	let mut data = pipe::Data {
	    vbuf: vertex_buffer,
	    locals: locals_buffer,
	    out: main_color,
	    model: model.into(),
	    view: view,
	    proj: projection
	};

    let mut running = true;
    let mut window_size = (SCREEN_FWIDTH, SCREEN_FHEIGHT);
    while running {
    	/*
    	if true {
    		let (vs, is) = cube.get_vertices_indices();
    		let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);

    		data.vbuf = vbuf;
    		slice = sl;
    	}*/
    	model = model*Matrix4::from_angle_z(cgmath::Rad(0.01));

    	data.model = model.into();

        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            use glutin::WindowEvent::*;
            use glutin::{MouseButton, ElementState, VirtualKeyCode};
            match event {
                KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _)
                | Closed => running = false,
                Resized(w, h) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    window_size = (w as f32, h as f32);
                },
                MouseMoved(_, _) => {},
                MouseInput(ElementState::Pressed, MouseButton::Left) => {}
                MouseInput(ElementState::Released, MouseButton::Left) => {},
                KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space), _) => {},
            	KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left), _) =>
            		cube.add_offset(-SCROLL_SPEED, 0.0),
            	KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right), _) =>
            		cube.add_offset(SCROLL_SPEED, 0.0),
            	KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up), _) =>
            		cube.add_offset(0.0, -SCROLL_SPEED),
            	KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down), _) =>
            		cube.add_offset(0.0, SCROLL_SPEED),
                _ => (),
            }
        });

        cube.tick();

        encoder.clear(&data.out, BLACK);
		encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}