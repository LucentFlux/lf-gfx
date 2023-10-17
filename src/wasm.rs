use std::num::NonZeroU32;
use wasm_bindgen::JsCast;
use wgpu::Surface;
use winit::platform::web::WindowExtWebSys;
use winit::window::Window;

pub(crate) fn get_canvas() -> web_sys::HtmlCanvasElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    document
        .get_element_by_id("game")
        .expect("you must include a canvas with ID `game` to render into: `<canvas id=\"game\"/>`")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}
