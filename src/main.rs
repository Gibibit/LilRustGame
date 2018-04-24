#[macro_use] extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;
extern crate rand;
extern crate image;

use gfx::traits::FactoryExt;
use gfx::Device;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;
const SCREEN_FWIDTH: f32 = SCREEN_WIDTH as f32;
const SCREEN_FHEIGHT: f32 = SCREEN_HEIGHT as f32;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
    	switch: gfx::Global<i32> = "i_Switch",
        awesome: gfx::TextureSampler<[f32; 4]> = "t_Awesome",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

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
}

#[derive(Debug, Clone, Copy)]
struct Square {
	pub pos: (f32, f32),
	pub size: (f32, f32),
	pub color: [f32; 3]
}

#[derive(Debug, Clone, Copy)]
enum Cursor {
	Plain((f32, f32), [f32; 3]),
	Growing((f32, f32), (f32, f32), [f32; 3])
}

impl Cursor {
	fn to_square(self) -> Square {
		match self {
			Cursor::Plain(xy, color) => Square { pos: xy, size: (0.05, 0.05), color },
			Cursor::Growing(xy, size, color) => Square { pos: xy, size, color },
		}
	}
}



#[derive(Debug)]
struct Pseudocube {
	cursor: Cursor,
	head: Square,
	eye_container: Square,
	eye: Square,
	squares: Vec<Square>,
}

impl Pseudocube {
	pub fn new() -> Self {
		Pseudocube {
			cursor: Cursor::Plain((0.0, 0.0), WHITE),
			head: Square { pos: (200.0, 200.0), size: (200.0, 200.0), color: rand::random() },
			eye_container: Square { pos: (260.0, 320.0), size: (80.0, 80.0), color: rand::random() },
			eye: Square { pos: (285.0, 370.0), size: (30.0, 30.0), color: rand::random() },
			squares: vec![],
		}
	}

	pub fn add_square(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 3]) {
		let sq = Square {
			pos: (x, y),
			size: (width, height), color
		};
		self.squares.push(sq);
	}

	pub fn get_vertices_indices(&self) -> (Vec<Vertex>, Vec<u16>) {
		let (mut vs, mut is) = (vec![], vec![]);
		let cursor = self.cursor.to_square();

		for(i, sq) in self.squares.iter()
			.chain(Some(&self.head))
			.chain(Some(&self.eye_container))
			.chain(Some(&self.eye))
			.chain(Some(&cursor))
			.enumerate()
		{
			let pos = (sq.pos.0/SCREEN_FWIDTH, sq.pos.1/SCREEN_FHEIGHT);
			let size = (sq.size.0/SCREEN_FWIDTH, sq.size.1/SCREEN_FHEIGHT);
			let i = i as u16;

			let x1 = pos.0*2.0 - 1.0;
			let y1 = -pos.1*2.0 + 1.0;
			let x2 = x1 + size.0*2.0;
			let y2 = y1 - size.1*2.0;

			vs.extend(&[
				Vertex { pos: [x2, y1], uv: [1.0, 1.0], color: sq.color },
                Vertex { pos: [x1, y1], uv: [0.0, 1.0], color: sq.color },
                Vertex { pos: [x1, y2], uv: [0.0, 0.0], color: sq.color },
                Vertex { pos: [x2, y2], uv: [1.0, 0.0], color: sq.color },
			]);
			is.extend(&[
				4*i, 4*i + 1, 4*i + 2, 4*i + 2, 4*i + 3, 4*i
			]);
		}

		(vs, is)
	}

	pub fn update_cursor_position(&mut self, x: f32, y: f32) {
		let x = 2.0*x - 1.0;
		let y = -2.0*y + 1.0;
		let cursor = match self.cursor {
			Cursor::Plain(_, color) => Cursor::Plain((x, y), color),
            Cursor::Growing(_, size, color) => Cursor::Growing((x, y), size, color),
		};
		self.cursor = cursor;
	}

	pub fn update_eye_position(&mut self, mouse_x: f32, mouse_y: f32) {
		let cpos = self.eye_container.pos;
		let csize = self.eye_container.size;
		let pos = (
			fclamp(mouse_x, cpos.0, cpos.0 + csize.0 - self.eye.size.0),
			self.eye.pos.1
		);
		let mut neweye = self.eye;
		neweye.pos = pos;
		self.eye = neweye;
	}

	pub fn start_growing(&mut self) {
		if let Cursor::Plain(xy, color) = self.cursor {
			self.cursor = Cursor::Growing(xy, (0.05, 0.05), color)
		}
	}

	pub fn stop_growing(&mut self) {
		if let Cursor::Growing(xy, size, color) = self.cursor {
			self.squares.push(Cursor::Growing(xy, size, color).to_square());
			self.cursor = Cursor::Plain(xy, rand::random());
		}
	}

	pub fn tick(&mut self) {
		if let Cursor::Growing(xy, size, color) = self.cursor {
			let newsize = (size.0 + 0.01, size.1 + 0.01);
			self.cursor = Cursor::Growing(xy, newsize, color)
		}
	}
}

pub fn main() {
	let mut cube = Pseudocube::new();

    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Square Toy".to_string())
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
    window.set_inner_size(SCREEN_WIDTH, SCREEN_HEIGHT);

	let (vertices, indices) = cube.get_vertices_indices();
	let (vertex_buffer, mut slice) = 
		factory.create_vertex_buffer_with_slice(&vertices, &*indices);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
	    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslv")),
	    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslf")),
	    pipe::new()
	).unwrap();

    let texture = load_texture(&mut factory, "assets/awesome.png");
    let sampler = factory.create_sampler_linear();
	let mut data = pipe::Data {
	    vbuf: vertex_buffer,
	    switch: 1,
	    awesome: (texture, sampler),
	    out: main_color,
	};

    let mut running = true;
    let mut needs_update = false;
    let mut window_size = (SCREEN_FWIDTH, SCREEN_FHEIGHT);
    while running {
    	if true {
    		let (vs, is) = cube.get_vertices_indices();
    		let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);

    		data.vbuf = vbuf;
    		slice = sl;

    		needs_update = false;
    	}

        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            use glutin::WindowEvent::*;
            use glutin::{MouseButton, ElementState, VirtualKeyCode};
            match event {
                KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _)
                | Closed => running = false,
                Resized(w, h) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    window_size = (w as f32, h as f32);
                    needs_update = true;
                },
                MouseMoved(x, y) => {
                	let mouse_x = x as f32;
					let mouse_y = y as f32;
                	cube.update_cursor_position(mouse_x, mouse_y);
            		cube.update_eye_position(mouse_x, mouse_y);
            		needs_update = true;
                },
                MouseInput(ElementState::Pressed, MouseButton::Left) => {
                	cube.start_growing();
                	needs_update = true;
            	}
                MouseInput(ElementState::Released, MouseButton::Left) => cube.stop_growing(),
                KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space), _) =>
                	if data.switch == 0 {
                		data.switch = 1;
                	} else {
                		data.switch = 0;
                	},
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