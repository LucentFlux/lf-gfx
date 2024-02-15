use wasm_bindgen::JsCast;

pub(crate) fn get_canvas() -> web_sys::HtmlCanvasElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document
        .get_element_by_id("game")
        .expect("you must include a canvas with ID `game` to render into: `<canvas id=\"game\"/>`")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}
