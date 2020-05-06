
use std::time::{Instant, Duration};

use ale::{Ale, BundledRom};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 210;

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() -> Result<(), Error> {
	let mut ale = Ale::new();
	ale.load_rom(BundledRom::Breakout).expect("Illegal rom");

	let event_loop = EventLoop::new();
	let mut input = WinitInputHelper::new();
	let (window, surface, mut p_width, mut p_height, mut hidpi_factor) =
		create_window("Breakout", &event_loop, SCREEN_WIDTH, SCREEN_HEIGHT);

	let surface_texture = SurfaceTexture::new(p_width, p_height, surface);

	let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?;
	let mut paused = false;
	let mut prev_update = Instant::now();
	println!("=== CONTROLS ===");
	println!("Space - Start");
	println!("A / Left - Flipper left");
	println!("D / Right - Flipper right");
	println!("P - Toggle pause");
	println!();
	println!("Paused: false");

	let mut screen = vec![];
	screen.resize(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 3, 0u8);

	event_loop.run(move |event, _, control_flow| {
		// The one and only event that winit_input_helper doesn't have for us...
		if let Event::RedrawRequested(_) = event {
			let screen_width = ale.screen_width();
			let screen_height = ale.screen_height();
			screen.resize(screen_width * screen_height * 3, 0);
			ale.get_screen_rgb(&mut screen);

			let frame = pixels.get_frame();
			let mut x = 0;
			let mut y = 0;
			for pixel in frame.chunks_exact_mut(4) {
				if x < screen_width && y < screen_height {
					pixel[0] = screen[(y * screen_width + x) as usize * 3    ]; // R
					pixel[1] = screen[(y * screen_width + x) as usize * 3 + 1]; // G
					pixel[2] = screen[(y * screen_width + x) as usize * 3 + 2]; // B
				} else {
					pixel[0] = 0xff; // R
					pixel[1] = 0x00; // G
					pixel[2] = 0x00; // B
				}
				pixel[3] = 0xff; // A
				x += 1;
				if x >= screen_width {
					x = 0;
					y += 1;
				}
			}
			pixels.render();
		}

		// For everything else, for let winit_input_helper collect events to build its state.
		// It returns `true` when it is time to update our game state and request a redraw.
		if input.update(event) {
			// Close event
			if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
				*control_flow = ControlFlow::Exit;
				return;
			}
			// Pause
			if input.key_pressed(VirtualKeyCode::P) {
				paused = !paused;
				println!("Paused: {}", paused);
				if !paused {
					prev_update = Instant::now();
				}
			}
			// Reset
			if input.key_pressed(VirtualKeyCode::R) {
				ale.reset_game();
				println!("RESET");
			}
			
			// Get action
			let action = if input.key_held(VirtualKeyCode::Left) || input.key_held(VirtualKeyCode::A) {
				Some(4)
			} else if input.key_held(VirtualKeyCode::Right) || input.key_held(VirtualKeyCode::D) {
				Some(3)
			} else if input.key_held(VirtualKeyCode::Space) {
				Some(1)
			} else if !paused {
				Some(0)
			} else {
				None
			};
			
			// Update
			let now = Instant::now();
			let mut diff = now - prev_update;
			if diff > 5 * FRAME_DURATION {
				diff = 5 * FRAME_DURATION;
				println!("Warning: skip of {}s occured", (diff - 5 * FRAME_DURATION).as_secs_f64());
			}
			while diff > FRAME_DURATION {
				diff -= FRAME_DURATION;
				prev_update = now;
				if let Some(action) = action {
					if ale.legal_action_set().contains(&action) {
						//println!("Update: {}", action);
						ale.act(action);
					} else {
						println!("Warning: illegal action: {}", action);
					}
				}
			}
			
			// Game over
			if ale.is_game_over() {
				println!("Game OVER");
				ale.reset_game();
				println!("RESET");
			}
			
			// Adjust high DPI factor
			if let Some(factor) = input.scale_factor_changed() {
				hidpi_factor = factor;
			}
			// Resize the window
			if let Some(size) = input.window_resized() {
				p_width = size.width;
				p_height = size.height;
				pixels.resize(p_width, p_height);
			}
			
			// Request redraw
			window.request_redraw();
		}
	});
}

/// Create a window for the game.
///
/// Automatically scales the window to cover about 2/3 of the monitor height.
///
/// # Returns
///
/// Tuple of `(window, surface, width, height, hidpi_factor)`
/// `width` and `height` are in `PhysicalSize` units.
fn create_window(
	title: &str,
	event_loop: &EventLoop<()>,
	width: u32,
	height: u32,
) -> (winit::window::Window, pixels::wgpu::Surface, u32, u32, f64) {
	// Create a hidden window so we can estimate a good default window size
	let window = winit::window::WindowBuilder::new()
		.with_visible(false)
		.with_title(title)
		.build(&event_loop)
		.unwrap();
	let hidpi_factor = window.scale_factor();

	// Get dimensions
	let width = width as f64;
	let height = height as f64;
	let (monitor_width, monitor_height) = {
		let size = window.current_monitor().size();
		(size.width as f64 / hidpi_factor, size.height as f64 / hidpi_factor)
	};
	let scale = (monitor_height / height * 2.0 / 3.0).round();

	// Resize, center, and display the window
	let min_size: winit::dpi::LogicalSize<f64> = PhysicalSize::new(width, height).to_logical(hidpi_factor);
	let default_size = LogicalSize::new(width * scale, height * scale);
	let center = LogicalPosition::new(
		(monitor_width - width * scale) / 2.0,
		(monitor_height - height * scale) / 2.0,
	);
	window.set_inner_size(default_size);
	window.set_min_inner_size(Some(min_size));
	window.set_outer_position(center);
	window.set_visible(true);

	let surface = pixels::wgpu::Surface::create(&window);
	let size: winit::dpi::PhysicalSize<f64> = default_size.to_physical(hidpi_factor);

	(
		window,
		surface,
		size.width.round() as u32,
		size.height.round() as u32,
		hidpi_factor,
	)
}
