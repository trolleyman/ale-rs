
use ale::{Ale, BundledRom};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
	env_logger::init();
	let event_loop = EventLoop::new();
	let mut input = WinitInputHelper::new();
	let (window, surface, mut p_width, mut p_height, mut hidpi_factor) =
		create_window("Conway's Game of Life", &event_loop);

	let surface_texture = SurfaceTexture::new(p_width, p_height, surface);

	let mut life = ConwayGrid::new_random(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);
	let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?;
	let mut paused = false;

	let mut draw_state: Option<bool> = None;

	event_loop.run(move |event, _, control_flow| {
		// The one and only event that winit_input_helper doesn't have for us...
		if let Event::WindowEvent {
			event: WindowEvent::RedrawRequested,
			..
		} = event
		{
			life.draw(pixels.get_frame());
			pixels.render();
		}

		// For everything else, for let winit_input_helper collect events to build its state.
		// It returns `true` when it is time to update our game state and request a redraw.
		if input.update(event) {
			// Close events
			if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
				*control_flow = ControlFlow::Exit;
				return;
			}
			if input.key_pressed(VirtualKeyCode::P) {
				paused = !paused;
			}
			
			let action = if input.key_pressed(VirtualKeyCode::Left) || input.key_pressed(VirtualKeyCode::A) {
				action = Some(1);
			} else if input.key_pressed(VirtualKeyCode::Right) || input.key_pressed(VirtualKeyCode::D) {
				action = Some(2);
			} else if input.key_pressed(VirtualKeyCode::Space) {
				action = Some(3);
			} else if !paused {
				action = Some(0);
			} else {
				None
			};

			// Adjust high DPI factor
			if let Some(factor) = input.hidpi_changed() {
				hidpi_factor = factor;
			}
			// Resize the window
			if let Some(size) = input.window_resized() {
				let size = size.to_physical(hidpi_factor);
				p_width = size.width.round() as u32;
				p_height = size.height.round() as u32;
				pixels.resize(p_width, p_height);
			}
			// Act
			if let Some(action) = action {
				if ale.legal_action_set().contains(action) {
					ale.act(action);
				}
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
	let hidpi_factor = window.hidpi_factor();

	// Get dimensions
	let width = width as f64;
	let height = height as f64;
	let (monitor_width, monitor_height) = {
		let size = window.current_monitor().size();
		(size.width / hidpi_factor, size.height / hidpi_factor)
	};
	let scale = (monitor_height / height * 2.0 / 3.0).round();

	// Resize, center, and display the window
	let min_size = PhysicalSize::new(width, height).to_logical(hidpi_factor);
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
	let size = default_size.to_physical(hidpi_factor);

	(
		window,
		surface,
		size.width.round() as u32,
		size.height.round() as u32,
		hidpi_factor,
	)
}
