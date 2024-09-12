mod model;
use {
    lottie::prelude::Bezier, macroquad::prelude::*, model::{LayerContent, Model, Shape}, nanoserde::{DeJson, SerJson}, serde_json, std::fs
};

fn load_lottie_file(compare_with_serde: bool) -> Model {
    let path = "assets/glaxnimate_white_triangle.json";
    // let path = "assets/glaxnimate_triangle.json";
    // let path = "assets/glaxnimate_rectangles.json";
    // let path = "pylottie_circle.json";
    // let path = "../lottie-rs/fixtures/ui/bouncy_ball.json";
    // let path = "../lottie-rs/fixtures/ui/7148-the-nyan-cat.json";
    // let path = "../lottie-rs/fixtures/ui/delete.json";

    let data = fs::read_to_string(path).expect("Unable to read file");

    let s_model: Option<lottie::Model> = if compare_with_serde {
        Some(serde_json::from_str(&data).expect("serde cannot deserialize model"))
    } else {
        None
    };
    // dbg!(&s_model);

    let ns_model: Model =
        DeJson::deserialize_json(&data).expect("nanoserde cannot deserialize model");
    // dbg!(&ns_model);

    if compare_with_serde {
        let s_ser_model = serde_json::to_string(&s_model).expect("serde cannot serialize");
        println!("serde ser: {}", s_ser_model);
        println!("");
        let ns_ser_model = SerJson::serialize_json(&ns_model);
        println!("nanoserde ser: {}", ns_ser_model);
        assert_eq!(s_ser_model, ns_ser_model);
    }
    ns_model
}

#[macroquad::main("Lottie Example")]
async fn main() {
    let model = load_lottie_file(false);
    // dbg!(&model);

    loop {
        clear_background(LIGHTGRAY);

        for layer in &model.layers {
            match &layer.content {
                LayerContent::Shape(shape) => {
                    let mut _fill = None;
                    let mut _gradient_fill = None;
                    let mut _stroke = None;
                    for shape in &shape.shapes {
                        match &shape.shape {
                            Shape::Fill(fill) => _fill = Some(fill.clone()),
                            Shape::GradientFill(fill) => _gradient_fill = Some(fill.clone()),
                            Shape::Stroke(stroke) => _stroke = Some(stroke.clone()),
                            _ => (),
                        }
                    }
                    for shape in &shape.shapes {
                        match &shape.shape {
                            Shape::Rectangle(rectangle) => {
                                if !rectangle.position.animated && !rectangle.size.animated {
                                    let x = &rectangle.position.keyframes[0].start_value.0.x;
                                    let y = &rectangle.position.keyframes[0].start_value.0.y;
                                    let w = &rectangle.size.keyframes[0].start_value.0.x;
                                    let h = &rectangle.size.keyframes[0].start_value.0.y;
                                    if let Some(fill) = &_fill {
                                        if !fill.color.animated {
                                            let color = fill.color.keyframes[0].start_value;
                                            let color =
                                                Color::from_rgba(color.r, color.g, color.b, 255);
                                            draw_rectangle(
                                                *x - *w / 2.,
                                                *y - *h / 2.,
                                                *w,
                                                *h,
                                                color,
                                            );
                                        }
                                    }
                                }
                            }
                            Shape::Path { data, direction, .. } => {
                                if !data.animated {
                                    let bezier = &data.keyframes[0].start_value;
                                    if let Some(fill) = &_fill {
                                        if !fill.color.animated {
                                            let color = fill.color.keyframes[0].start_value;
                                            let color =
                                                Color::from_rgba(color.r, color.g, color.b, 255);
                                            draw_rectangle(1., 1., 20., 20., color);
                                        }
                                    }
                                }
                            },
                            Shape::GradientFill(_) | Shape::Fill(_) | Shape::Stroke(_) => (),
                            _ => unimplemented!(),
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        // break;
        next_frame().await;
    }
}
