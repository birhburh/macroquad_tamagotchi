mod model;
use {
    model::Model,
    nanoserde::{DeJson, SerJson},
    serde_json,
    std::fs,
};

fn main() {
    // let data = fs::read_to_string("pylottie_ball.json").expect("Unable to read file");
    let data = fs::read_to_string("test.json").expect("Unable to read file");

    let s_model: lottie::Model = serde_json::from_str(&data).expect("serde cannot deserialize model");
    dbg!(&s_model);
    let s_ser_model = serde_json::to_string(&s_model).expect("serde cannot serialize");
    println!("serde ser: {}", s_ser_model);
    println!("");

    let ns_model: Model = DeJson::deserialize_json(&data).expect("nanoserde cannot deserialize model");
    dbg!(&ns_model);
    let ns_ser_model = SerJson::serialize_json(&ns_model);
    println!("nanoserde ser: {}", ns_ser_model);

    assert_eq!(s_ser_model, ns_ser_model);
}
