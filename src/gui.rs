use crate::GLFWState;

use imgui_glfw_rs::imgui::{Window,im_str,Condition};


pub struct GUIState {
    pub using_peeling: bool,
    pub used_mouse: bool,
}

impl Clone for GUIState {
    fn clone(&self) -> GUIState {
	GUIState { using_peeling: self.using_peeling,
		   used_mouse:    self.used_mouse }
    }
}

pub fn run_gui(glfw_state: &mut GLFWState, gui_state: &mut GUIState) {
    let ui = glfw_state.imgui_glfw_context.frame(&mut glfw_state.window, &mut glfw_state.imgui_context);
    // ui.show_demo_window(&mut true);

    Window::new(im_str!("Hello"))
	.size([300.0, 100.0], Condition::FirstUseEver)
	.position([0.0, 0.0], Condition::Always)
	.build(&ui, || {
	    ui.text(im_str!("Hello again"));
	    ui.text(im_str!("Halla"));
	    ui.text(im_str!("Welcome to the void"));
	    ui.separator();
	    ui.text(im_str!("Use peeling"));
	    ui.radio_button(im_str!("On"), &mut gui_state.using_peeling, true);
	    ui.radio_button(im_str!("Off"), &mut gui_state.using_peeling, false);
	    let mouse_pos = ui.io().mouse_pos;

	    ui.separator();
	    ui.text(format!(
		"Mouse Position: ({:.1},{:.1})",
		mouse_pos[0], mouse_pos[1]
	    ));
	});

    gui_state.used_mouse = ui.io().want_capture_mouse;
    
    glfw_state.imgui_glfw_context.draw(ui, &mut glfw_state.window);
}

