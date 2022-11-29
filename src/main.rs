mod shape;
mod vec;

use std::{cell::RefCell, rc::Rc};

use shape::Shape;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, FileReader, HtmlElement, HtmlInputElement, HtmlSelectElement};
use yew::{prelude::*};

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
    ShiftDown,
    ShiftUp,
    ModeChange(Mode),
    Clear,
    Save,
    Load(String),
    None,
    FinishShape,
    ShiftVectorChange(vec::Vector2f),
    ScaleVectorChange(vec::Vector2f),
    RotateAngleChange(f32),
    ApplyTransform,
    PivotChange(vec::Vector2f),
    CtrlDown,
    CtrlUp,
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
    mouse_delta: Option<vec::Vector2f>,
    selected_shape: Option<Rc<RefCell<Shape>>>,
    shift_is_down: bool,
    ctrl_is_down: bool,

    shift_vector: vec::Vector2f,
    scale_vector: vec::Vector2f,
    rotate_angle: f32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let on_shift_down = ctx.link().callback(|_| Msg::ShiftDown);
        let on_shift_up = ctx.link().callback(|_| Msg::ShiftUp);
        let on_ctrl_down = ctx.link().callback(|_| Msg::CtrlDown);
        let on_ctrl_up = ctx.link().callback(|_| Msg::CtrlUp);
        let on_shift_down_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if event.shift_key() {
                on_shift_down.emit(());
            }
            if event.ctrl_key() {
                on_ctrl_down.emit(());
            }
        }) as Box<dyn FnMut(_)>);
        let on_shift_up_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if !event.shift_key() {
                on_shift_up.emit(());
            }
            if !event.ctrl_key() {
                on_ctrl_up.emit(());
            }
        }) as Box<dyn FnMut(_)>);

        window()
            .unwrap()
            .add_event_listener_with_callback(
                "keydown",
                on_shift_down_closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        window()
            .unwrap()
            .add_event_listener_with_callback("keyup", on_shift_up_closure.as_ref().unchecked_ref())
            .unwrap();

        on_shift_down_closure.forget();
        on_shift_up_closure.forget();

        Self {
            mode: Mode::Draw,
            shapes: Vec::new(),
            canvas: NodeRef::default(),
            canvas_ctx: None,
            pivot: None,
            is_mouse_down: false,
            mouse_origin: None,
            mouse_pos: None,
            mouse_delta: None,
            selected_shape: None,
            shift_is_down: false,
            ctrl_is_down: false,

            shift_vector: vec::Vector2f::new(0.0, 0.0),
            scale_vector: vec::Vector2f::new(1.0, 1.0),
            rotate_angle: 0.0,
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

        let finish_shape_button = html! {
            <button onclick={ctx.link().callback(|_| Msg::FinishShape)}>{"Finish Shape"}</button>
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

        let shift_vector = self.shift_vector.clone();
        let scale_vector = self.scale_vector.clone();
        let pivot = self.pivot.unwrap_or(vec::Vector2f::new(0.0, 0.0));
        let input_boxes = html! {
            <>
                <div>
                    <label>{"Pivot: "}</label>
                    <input
                        type="number"
                        min="0"
                        max="800"
                        value={pivot.x().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let pivot = pivot.clone();
                            let target = e.target().unwrap();
                            let target: HtmlInputElement = target.dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::PivotChange(vec::Vector2f::new(value, pivot.y()))
                        })}
                    />
                    <input
                        type="number"
                        min="0"
                        max="600"
                        value={pivot.y().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let pivot = pivot.clone();
                            let target = e.target().unwrap();
                            let target: HtmlInputElement = target.dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::PivotChange(vec::Vector2f::new(pivot.x(), value))
                        })}
                    />
                </div>
                <div>
                    <label>{"Shift vector: "}</label>
                    <input
                        type="number"
                        min="-1000"
                        max="1000"
                        value={self.shift_vector.x().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let shift_vector = shift_vector.clone();
                            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::ShiftVectorChange(vec::Vector2f::new(value, shift_vector.y()))
                        })}
                    />
                    <input
                        type="number"
                        min="-1000"
                        max="1000"
                        value={self.shift_vector.y().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let shift_vector = shift_vector.clone();
                            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::ShiftVectorChange(vec::Vector2f::new(shift_vector.x(), value))
                        })}
                    />
                </div>
                <div>
                    <label>{"Scale vector: "}</label>
                    <input
                        type="number"
                        step="0.01"
                        min="-1000"
                        max="1000"
                        value={self.scale_vector.x().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let scale_vector = scale_vector.clone();
                            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::ScaleVectorChange(vec::Vector2f::new(value, scale_vector.y()))
                        })}
                    />
                    <input
                        type="number"
                        step="0.01"
                        min="-1000"
                        max="1000"
                        value={self.scale_vector.y().to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let scale_vector = scale_vector.clone();
                            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::ScaleVectorChange(vec::Vector2f::new(scale_vector.x(), value))
                        })}
                    />
                </div>
                <div>
                    <label>{"Rotate angle: "}</label>
                    <input
                        type="number"
                        step="0.01"
                        min="-1000"
                        max="1000"
                        value={self.rotate_angle.to_string()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            let value = target.value_as_number() as f32;
                            Msg::RotateAngleChange(value)
                        })}
                    />
                </div>
                <button onclick={ctx.link().callback(|_| Msg::ApplyTransform)}>{"Apply Transform"}</button>
            </>
        };

        html! {
            <div>
                <div>
                    {mode_select}
                    {clear_button}
                    {save_button}
                    {load_button}
                    {finish_shape_button}
                </div>
                <div>
                    {canvas}
                    {input_boxes}
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MouseDown(event) => {
                let mouse_pos =
                    vec::Vector2f::new(event.offset_x() as f32, event.offset_y() as f32);

                if self.shift_is_down {
                    self.pivot = Some(mouse_pos);

                    return true;
                }

                if self.ctrl_is_down {
                    self.selected_shape = self.shapes.iter().find_map(|shape| {
                        if shape.borrow().intersect_with_point(mouse_pos) {
                            Some(shape.clone())
                        } else {
                            None
                        }
                    });
                    
                    return true;
                }

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
                        self.mouse_pos = Some(mouse_pos);
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
                self.is_mouse_down = false;
                self.mouse_origin = None;
                self.mouse_pos = None;
                self.mouse_delta = None;

                true
            }
            Msg::MouseMove(event) => {
                let mouse_pos =
                    vec::Vector2f::new(event.offset_x() as f32, event.offset_y() as f32);

                self.mouse_delta = match self.mouse_pos {
                    Some(prev_pos) => Some(mouse_pos - prev_pos),
                    _ => None,
                };

                if self.is_mouse_down {
                    self.mouse_pos = Some(mouse_pos);
                } else {
                    self.mouse_pos = None;
                }

                match self.mode {
                    Mode::Rotate => {
                        if let (Some(pivot), Some(mouse_delta), Some(mouse_pos)) =
                            (self.pivot, self.mouse_delta, self.mouse_pos)
                        {
                            if let Some(selected_shape) = &self.selected_shape {
                                let angle = (mouse_pos - pivot).angle()
                                    - (mouse_pos - mouse_delta - pivot).angle();
                                selected_shape
                                    .borrow_mut()
                                    .rotate_rel_to_point(angle, pivot);
                            }
                        }
                    }
                    Mode::Scale => {
                        if let (Some(pivot), Some(mouse_delta), Some(mouse_pos)) =
                            (self.pivot, self.mouse_delta, self.mouse_pos)
                        {
                            if let Some(selected_shape) = &self.selected_shape {
                                let scale = (mouse_pos - pivot).length()
                                    / (mouse_pos - mouse_delta - pivot).length();
                                let scale = vec::Vector2f::new(scale, scale);
                                selected_shape.borrow_mut().scale_rel_to_point(scale, pivot);
                            }
                        }
                    }
                    Mode::Shift => {
                        if let Some(mouse_delta) = self.mouse_delta {
                            if let Some(selected_shape) = &self.selected_shape {
                                selected_shape.borrow_mut().shift(mouse_delta);
                            }
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
            Msg::FinishShape => {
                self.shapes.push(Rc::new(RefCell::new(Shape::new())));

                true
            }
            Msg::ShiftDown => {
                self.shift_is_down = true;

                true
            }
            Msg::ShiftUp => {
                self.shift_is_down = false;

                true
            }
            Msg::ShiftVectorChange(vec) => {
                self.shift_vector = vec;

                true
            }
            Msg::ScaleVectorChange(vec) => {
                self.scale_vector = vec;

                true
            }
            Msg::RotateAngleChange(angle) => {
                self.rotate_angle = angle;

                true
            }
            Msg::PivotChange(vec) => {
                self.pivot = Some(vec);

                true
            }
            Msg::ApplyTransform => {
                if let Some(selected_shape) = &self.selected_shape {
                    let radians = self.rotate_angle.to_radians();
                    selected_shape.borrow_mut().shift(self.shift_vector);
                    selected_shape.borrow_mut().scale_rel_to_point(
                        self.scale_vector,
                        self.pivot.unwrap_or(vec::Vector2f::new(0.0, 0.0)),
                    );
                    selected_shape.borrow_mut().rotate_rel_to_point(
                        radians,
                        self.pivot.unwrap_or(vec::Vector2f::new(0.0, 0.0)),
                    );
                }

                true
            }
            Msg::CtrlDown => {
                self.ctrl_is_down = true;

                true
            }
            Msg::CtrlUp => {
                self.ctrl_is_down = false;

                true
            }
            Msg::None => false,
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

        if let (Some(mouse_pos), Some(mouse_down_origin)) = (self.mouse_pos, self.mouse_origin) {
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

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
