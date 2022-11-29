mod shape;
mod vec;

use std::{cell::RefCell, rc::Rc};

use js_sys::Uint32Array;
use shape::Shape;
use vec::Vector2f;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{window, FileReader, HtmlElement, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

enum Mode {
    Draw,
    Rotate,
    Scale,
    Shift,
}

enum Msg {
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    MouseLeave(MouseEvent),
    ModeChange(Mode),
    Clear,
    Save,
    Load(String),
    ShiftShape { vector: Vector2f },
    RotateShape { angle: f32, pivot: Vector2f },
    ScaleShape { scale: Vector2f, pivot: Vector2f },
    None,
}

struct App {
    mode: Mode,
    shapes: Vec<Rc<RefCell<Shape>>>,
    pivot: Option<vec::Vector2f>,
    canvas: NodeRef,
    canvas_ctx: Option<web_sys::CanvasRenderingContext2d>,
    is_mouse_down: bool,
    mouse_origin: Option<vec::Vector2f>,
    mouse_pos: Option<vec::Vector2f>,
    selected_shape: Option<Rc<RefCell<Shape>>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            mode: Mode::Draw,
            shapes: Vec::new(),
            canvas: NodeRef::default(),
            canvas_ctx: None,
            pivot: None,
            is_mouse_down: false,
            mouse_origin: None,
            mouse_pos: None,
            selected_shape: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let canvas = html! {
            <canvas
                ref={self.canvas.clone()}
                width="800"
                height="600"
                style="border: 1px solid black;"
                onmousedown={ctx.link().callback(Msg::MouseDown)}
                onmouseup={ctx.link().callback(Msg::MouseUp)}
                onmousemove={ctx.link().callback(Msg::MouseMove)}
                onmouseleave={ctx.link().callback(Msg::MouseLeave)}
            />
        };

        let mode_select = html! {
            <select
                onchange={ctx.link().callback(|e: Event| {
                    let target = e.target().unwrap();
                    let target = target.dyn_into::<HtmlSelectElement>().unwrap();
                    let value = target.value();
                    let mode = match value.as_str() {
                        "Draw" => Mode::Draw,
                        "Rotate" => Mode::Rotate,
                        "Scale" => Mode::Scale,
                        "Shift" => Mode::Shift,
                        _ => Mode::Draw,
                    };
                    Msg::ModeChange(mode)
                })}
            >
                <option value="Draw">{"Draw"}</option>
                <option value="Rotate">{"Rotate"}</option>
                <option value="Scale">{"Scale"}</option>
                <option value="Shift">{"Shift"}</option>
            </select>
        };

        let clear_button = html! {
            <button onclick={ctx.link().callback(|_| Msg::Clear)}>{"Clear"}</button>
        };

        let save_button = html! {
            <button onclick={ctx.link().callback(|_| Msg::Save)}>{"Save"}</button>
        };

        let load_cb = ctx
            .link()
            .callback(|json_string: String| Msg::Load(json_string));
        let load_button = html! {
            <input
                type="file"
                onchange={ctx.link().callback(move |e: Event| {
                    let load_cb = load_cb.clone();
                    let target = e.target().unwrap();
                    let target: HtmlInputElement = target.dyn_into().unwrap();
                    let files = target.files().unwrap();
                    let file = files.get(0).unwrap();
                    let reader = web_sys::FileReader::new().unwrap();
                    let callback = Closure::wrap(Box::new(move |e: web_sys::ProgressEvent| {
                        let target = e.target().unwrap();
                        let target: FileReader = target.dyn_into().unwrap();
                        let result = target.result().unwrap();
                        let result = result.as_string().unwrap();
                        load_cb.emit(result);
                    }) as Box<dyn FnMut(_)>);
                    reader.set_onload(Some(callback.as_ref().unchecked_ref()));
                    reader.read_as_text(&file).unwrap();
                    callback.forget();
                    Msg::None
                })}
            />
        };

        html! {
            <div>
                <div>
                    {mode_select}
                    {clear_button}
                    {save_button}
                    {load_button}
                </div>
                <div>
                    {canvas}
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MouseDown(event) => {
                let mouse_pos =
                    vec::Vector2f::new_with_data(event.offset_x() as f32, event.offset_y() as f32);

                match self.mode {
                    Mode::Draw => {
                        if self.shapes.is_empty() {
                            self.shapes.push(Rc::new(RefCell::new(Shape::new())));
                        }

                        self.shapes
                            .last_mut()
                            .unwrap()
                            .borrow_mut()
                            .add_point(mouse_pos);
                    }
                    Mode::Rotate | Mode::Scale => {
                        if self.pivot.is_none() {
                            self.pivot = Some(mouse_pos);
                        } else {
                            self.mouse_origin = Some(mouse_pos);
                        }
                    }
                    Mode::Shift => {
                        if self.mouse_origin.is_none() {
                            self.mouse_origin = Some(mouse_pos);
                        }
                    }
                }

                self.selected_shape = self.shapes.iter().find_map(|shape| {
                    if shape.borrow().intersect_with_point(mouse_pos) {
                        Some(shape.clone())
                    } else {
                        None
                    }
                });

                self.is_mouse_down = true;

                true
            }
            Msg::MouseUp(_) => {
                match self.mode {
                    Mode::Shift => {
                        if let (Some(mouse_origin), Some(mouse_pos)) =
                            (self.mouse_origin, self.mouse_pos)
                        {
                            ctx.link().send_message(Msg::ShiftShape {
                                vector: mouse_pos - mouse_origin,
                            });
                        }
                    }
                    _ => {}
                }

                self.is_mouse_down = false;
                self.mouse_origin = None;

                true
            }
            Msg::MouseMove(event) => {
                let mouse_pos =
                    vec::Vector2f::new_with_data(event.offset_x() as f32, event.offset_y() as f32);

                if self.is_mouse_down {
                    self.mouse_pos = Some(mouse_pos);
                } else {
                    self.mouse_pos = None;
                }

                match self.mode {
                    Mode::Rotate => {
                        if let (Some(pivot), Some(mouse_pos), Some(mouse_origin)) =
                            (self.pivot, self.mouse_pos, self.mouse_origin)
                        {
                            let angle =
                                (mouse_pos - pivot).angle() - (mouse_origin - pivot).angle();
                            ctx.link().send_message(Msg::RotateShape { angle: angle / 30.0, pivot });
                        }
                    }
                    Mode::Scale => {
                        if let (Some(pivot), Some(mouse_pos)) = (self.pivot, self.mouse_pos) {
                            ctx.link().send_message(Msg::ScaleShape {
                                pivot,
                                scale: vec::Vector2f::new_with_data(
                                    (mouse_pos.x() - pivot.x()).abs(),
                                    (mouse_pos.y() - pivot.y()).abs(),
                                ),
                            });
                        }
                    }
                    _ => {}
                }

                true
            }
            Msg::MouseLeave(_) => false,
            Msg::ModeChange(mode) => {
                self.mode = mode;

                true
            }
            Msg::Clear => {
                self.shapes.clear();

                true
            }
            Msg::Save => {
                let json = serde_json::to_string(
                    &self
                        .shapes
                        .iter()
                        .map(|s| (*s.borrow()).clone())
                        .collect::<Vec<Shape>>(),
                )
                .unwrap();
                let data_str = format!("data:text/json;charset=utf-8,{}", json);
                let a = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("a")
                    .unwrap();
                a.set_attribute("href", &data_str).unwrap();
                a.set_attribute("download", "shapes.json").unwrap();
                a.set_attribute("style", "display: none").unwrap();

                let a = a.dyn_into::<HtmlElement>().unwrap();
                a.click();
                a.remove();

                false
            }
            Msg::Load(json_str) => {
                if let Ok(shapes) = serde_json::from_str::<Vec<Shape>>(&json_str) {
                    self.shapes = shapes
                        .iter()
                        .map(|s| Rc::new(RefCell::new(s.clone())))
                        .collect();

                    true
                } else {
                    window()
                        .unwrap()
                        .alert_with_message("Invalid JSON file")
                        .unwrap();

                    false
                }
            }
            Msg::None => false,
            Msg::ShiftShape { vector } => {
                if let Some(shape) = self.selected_shape.clone() {
                    shape.borrow_mut().shift(vector);
                }

                true
            }
            Msg::RotateShape { angle, pivot } => {
                if let Some(shape) = self.selected_shape.clone() {
                    shape.borrow_mut().rotate_rel_to_point(angle, pivot);
                }

                true
            }
            Msg::ScaleShape { scale, pivot } => {
                if let Some(shape) = self.selected_shape.clone() {
                    shape.borrow_mut().scale_rel_to_point(scale, pivot);
                }

                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let canvas = self.canvas.cast::<web_sys::HtmlCanvasElement>().unwrap();
            let canvas_ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            self.canvas_ctx = Some(canvas_ctx);
        }

        let ctx = self.canvas_ctx.as_ref().unwrap();

        ctx.clear_rect(0.0, 0.0, 800.0, 600.0);

        for shape in self.shapes.iter() {
            let shape = shape.borrow();
            let points = shape.get_points();

            if points.len() > 2 {
                ctx.set_fill_style(&"black".into());
                ctx.set_stroke_style(&"black".into());
                ctx.begin_path();
                ctx.move_to(points[0].x().into(), points[0].y().into());

                for point in points.iter().skip(1) {
                    ctx.line_to(point.x().into(), point.y().into());
                }

                if points.len() > 2 {
                    ctx.line_to(points[0].x().into(), points[0].y().into());
                }

                ctx.stroke();
            }

            for point in points.iter() {
                ctx.begin_path();
                ctx.set_fill_style(&"red".into());
                ctx.arc(
                    point.x().into(),
                    point.y().into(),
                    5.0,
                    0.0,
                    2.0 * std::f64::consts::PI,
                )
                .expect("Failed to draw point");
                ctx.fill();
            }

            if let Some(pivot) = self.pivot {
                ctx.begin_path();
                ctx.set_fill_style(&"blue".into());
                ctx.arc(
                    pivot.x().into(),
                    pivot.y().into(),
                    5.0,
                    0.0,
                    2.0 * std::f64::consts::PI,
                )
                .expect("Failed to draw pivot");
                ctx.fill();
            }

            if let (Some(mouse_pos), Some(mouse_down_origin)) = (self.mouse_pos, self.mouse_origin)
            {
                ctx.set_stroke_style(&"blue".into());
                ctx.begin_path();
                ctx.move_to(mouse_down_origin.x().into(), mouse_down_origin.y().into());
                ctx.line_to(mouse_pos.x().into(), mouse_pos.y().into());

                let arrow_length = 10.0;
                let arrow_angle = 0.5;

                let arrow_dir = (mouse_down_origin - mouse_pos).normalize();
                let arrow_left = arrow_dir.rotate(arrow_angle);
                let arrow_right = arrow_dir.rotate(-arrow_angle);

                ctx.move_to(mouse_pos.x().into(), mouse_pos.y().into());
                ctx.line_to(
                    (mouse_pos + arrow_left * arrow_length).x().into(),
                    (mouse_pos + arrow_left * arrow_length).y().into(),
                );

                ctx.move_to(mouse_pos.x().into(), mouse_pos.y().into());

                ctx.line_to(
                    (mouse_pos + arrow_right * arrow_length).x().into(),
                    (mouse_pos + arrow_right * arrow_length).y().into(),
                );

                ctx.stroke();
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
