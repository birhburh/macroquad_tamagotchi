mod model;
use {
    macroquad::prelude::*, nanoserde::{DeJson, SerJson}, std::{ffi::OsStr, fs, io::Read, path::Path},
    crate::model::Model,
};

fn os_str_as_u8_slice(s: &OsStr) -> &[u8] {
    unsafe { &*(s as *const OsStr as *const [u8]) }
}

unsafe fn u8_slice_as_os_str(s: &[u8]) -> &OsStr {
    // SAFETY: see the comment of `os_str_as_u8_slice`
    unsafe { &*(s as *const [u8] as *const OsStr) }
}

fn split_file_at_dot(file: &OsStr) -> (&OsStr, Option<&OsStr>) {
    let slice = os_str_as_u8_slice(file);
    if slice == b".." {
        return (file, None);
    }

    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let i = match slice[1..].iter().position(|b| *b == b'.') {
        Some(i) => i + 1,
        None => return (file, None),
    };
    let before = &slice[..i];
    let after = &slice[i + 1..];
    unsafe { (u8_slice_as_os_str(before), Some(u8_slice_as_os_str(after))) }
}

pub fn file_prefix(path: &Path) -> Option<&OsStr> {
    path.file_name()
        .map(split_file_at_dot)
        .and_then(|(before, _after)| Some(before))
}

// #[macroquad::main("BasicShapes")]
// async
fn main() {
    let data = fs::read_to_string("bouncy_ball.json").expect("Unable to read file");
    println!("{}", data);
    let animation: Model = nanoserde::DeJson::deserialize_json(&data).expect("Cannot deserialize animation file");
    println!("{:?}", animation);
    // loop {
    //     clear_background(BLACK);

    //     draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
    //     draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
    //     draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

    //     draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

    //     next_frame().await
    // }
}
