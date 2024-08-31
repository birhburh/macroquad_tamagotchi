// Based on https://github.com/zimond/lottie-rs/

// nanoserde TODO:
// - implement ((serialize|deserialize)_)?with
// - implement untagged

use {
    nanoserde::{DeJson, SerJson},
    std::fmt::Debug,
};


trait FromTo<T> {
    fn from(v: T) -> Self;
    fn to(self) -> T;
}

impl FromTo<Vec<f32>> for f32 {
    fn from(v: Vec<f32>) -> Self {
        v[0]
    }

    fn to(self) -> Vec<f32> {
        todo!();
    }
}

mod vector_2_d {
    use {
        nanoserde::{DeJson, DeJsonErr, DeJsonState, SerJson, SerJsonState},
        std::str::Chars
    };

    #[derive(PartialEq, Debug, Default, Clone)]
    pub struct Vector2D<T>(euclid::default::Vector2D<T>);

    impl<T> Vector2D<T> {
        pub const fn new(x: T, y: T) -> Self {
            Vector2D(euclid::default::Vector2D::new(x, y))
        }
    }

    impl DeJson for Vector2D<f32> {
        #[allow(clippy::ignored_unit_patterns)]
        fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
            println!("de_json for Vector2D");
            dbg!(&s.col);
            s.block_open(i)?;
            let x: f32 = {
                let r = DeJson::de_json(s, i)?;
                s.eat_comma_block(i)?;
                r
            };
            let y: f32 = {
                let r = DeJson::de_json(s, i)?;
                s.eat_comma_block(i)?;
                r
            };
            let r = Vector2D::new(x, y);
            s.block_close(i)?;
            Ok(r)
        }
    }

    impl<T: SerJson> SerJson for Vector2D<T> {
        fn ser_json(&self, d: usize, s: &mut SerJsonState) {
            s.st_pre();
            let mut first_field_was_serialized = false;
            if first_field_was_serialized {
                s.conl();
            }
            first_field_was_serialized = true;
            s.field(d + 1, "x");
            self.0.x.ser_json(d + 1, s);
            if first_field_was_serialized {
                s.conl();
            }
            s.field(d + 1, "y");
            self.0.y.ser_json(d + 1, s);
            s.st_post(d);
        }
    }
}

type Vector2D = vector_2_d::Vector2D<f32>;

impl FromTo<Vec<f32>> for Vector2D {
    fn from(v: Vec<f32>) -> Self {
        Vector2D::new(v[0].clone().into(), v.get(1).cloned().unwrap_or(0.0))
    }

    fn to(self) -> Vec<f32>
    {
        todo!();
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Model {
    #[nserde(rename = "nm")]
    pub name: Option<String>,
    #[nserde(rename = "v", default)]
    version: Option<String>,
    #[nserde(rename = "ip")]
    pub start_frame: f32,
    #[nserde(rename = "op")]
    pub end_frame: f32,
    #[nserde(rename = "fr")]
    pub frame_rate: f32,
    #[nserde(rename = "w")]
    pub width: u32,
    #[nserde(rename = "h")]
    pub height: u32,
    pub layers: Vec<Layer>,
    // #[nserde(default)]
    // pub assets: Vec<Asset>,
    // #[nserde(default)]
    // pub fonts: FontList,
}

#[derive(PartialEq, Debug, DeJson, SerJson)]
#[nserde(transparent)]
pub struct BoolFromInt(u32);

impl From<&bool> for BoolFromInt {
    fn from(e: &bool) -> BoolFromInt {
        dbg!(e);
        BoolFromInt(*e as u32)
    }
}
impl From<&BoolFromInt> for bool {
    fn from(n: &BoolFromInt) -> bool {
        dbg!(n);
        match n.0 {
            0 => false,
            1 => true,
            _ => panic!("wrong number for boolean representation"),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Layer {
    #[nserde(proxy = "BoolFromInt", rename = "ddd", default)]
    is_3d: bool,
    #[nserde(rename = "hd", default)]
    pub hidden: bool,
    #[nserde(rename = "ind", default)]
    pub index: Option<u32>,
    #[nserde(rename = "parent", default)]
    pub parent_index: Option<u32>,
    #[nserde(skip)]
    pub id: u32,
    #[nserde(rename = "ao", proxy = "BoolFromInt", default)]
    pub auto_orient: bool,
    #[nserde(rename = "ip")]
    pub start_frame: f32,
    #[nserde(rename = "op")]
    pub end_frame: f32,
    #[nserde(rename = "st")]
    pub start_time: f32,
    #[nserde(rename = "nm")]
    pub name: Option<String>,
    #[nserde(rename = "ks", default)]
    pub transform: Option<Transform>,
    // #[nserde(flatten)]
    // pub content: LayerContent,
    // #[nserde(rename = "tt", default)]
    // pub matte_mode: Option<MatteMode>,
    // #[nserde(rename = "bm", default)]
    // pub blend_mode: Option<BlendMode>,
    // #[nserde(default, rename = "hasMask")]
    // pub has_mask: bool,
    // #[nserde(default, rename = "masksProperties")]
    // pub masks_properties: Vec<Mask>,
}

pub fn default_vec2_100() -> Animated<Vector2D> {
    Animated {
        animated: false,
        keyframes: vec![KeyFrame::from_value(Vector2D::new(100.0, 100.0))],
    }
}

pub fn default_number_100() -> Animated<f32> {
    Animated {
        animated: false,
        keyframes: vec![KeyFrame::from_value(100.0)],
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Transform {
    #[nserde(rename = "a", default)]
    pub anchor: Option<Animated<Vector2D>>,
    #[nserde(rename = "p", default)]
    pub position: Option<Animated<Vector2D>>,
    #[nserde(rename = "s", default_with = "default_vec2_100")]
    pub scale: Animated<Vector2D>,
    #[nserde(rename = "r", default)]
    pub rotation: Animated<f32>,
    #[nserde(skip)]
    pub auto_orient: bool,
    #[nserde(rename = "o", default_with = "default_number_100")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "sk", default)]
    pub skew: Option<Animated<f32>>,
    #[nserde(rename = "sa", default)]
    pub skew_axis: Option<Animated<f32>>,
}

#[derive(PartialEq, Debug)]
// deserialize_with = "keyframes_from_array"
pub enum KeyFramesFromArray {
    Plain(f32),
    List(Vec<f32>),
    // TODO: Use legacy keyframes or something
}

impl DeJson for KeyFramesFromArray {
    fn de_json(
        s: &mut nanoserde::DeJsonState,
        i: &mut core::str::Chars,
    ) -> Result<Self, nanoserde::DeJsonErr> {
        println!("de_json for KeyFramesFromArray");
        dbg!(&s.tok);
        match s.tok {
            nanoserde::DeJsonTok::U64(v) => {
                s.next_tok(i)?;
                Ok(Self::Plain(v as f32))
            }
            nanoserde::DeJsonTok::BlockOpen => {
                let r = DeJson::de_json(s, i)?;
                Ok(Self::List(r))
            }
            _ => Err(s.err_token("F64 or ["))
        }
    }
}

impl SerJson for KeyFramesFromArray {
    fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {
        match self {
            Self::Plain(f0) => {
                f0.ser_json(d, s);
            }
            Self::List(f0) => {
                f0.ser_json(d, s);
            }
        }
    }
}

impl<T: Default> From<&Vec<KeyFrame<T>>> for KeyFramesFromArray {
    fn from(value: &Vec<KeyFrame<T>>) -> KeyFramesFromArray {
        KeyFramesFromArray::List(value.iter().map(|v| v.start_frame).collect())
    }
}

impl<T: Clone + Default + FromTo<Vec<f32>>> From<&KeyFramesFromArray> for Vec<KeyFrame<T>> {
    fn from(val: &KeyFramesFromArray) -> Vec<KeyFrame<T>> {
        match val {
            KeyFramesFromArray::Plain(v) => {
                vec![KeyFrame {
                    start_value: T::from(vec![*v]),
                    end_value: T::from(vec![*v]),
                    start_frame: 0.0,
                    end_frame: 0.0,
                    easing_in: None,
                    easing_out: None,
                }]
            }
            KeyFramesFromArray::List(v) => {
                vec![KeyFrame {
                    start_value: T::from(v.clone()),
                    end_value: T::from(v.clone()),
                    start_frame: 0.0,
                    end_frame: 0.0,
                    easing_in: None,
                    easing_out: None,
                }]
            }
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Animated<T: Debug + Default + Clone + FromTo<Vec<f32>>> {
    #[nserde(proxy = "BoolFromInt", rename = "a", default)]
    pub animated: bool,
    #[nserde(proxy = "KeyFramesFromArray", rename = "k")]
    pub keyframes: Vec<KeyFrame<T>>,
}

#[derive(SerJson, DeJson, Default, Debug, Clone, PartialEq)]
pub struct KeyFrame<T: Default> {
    #[nserde(rename = "s")]
    pub start_value: T,
    #[nserde(skip)]
    pub end_value: T,
    #[nserde(rename = "t", default)]
    pub start_frame: f32,
    // TODO: could end_frame & next start_frame create a gap?
    #[nserde(skip)]
    pub end_frame: f32,
    #[nserde(rename = "o", default)]
    pub easing_out: Option<Easing>,
    #[nserde(rename = "i", default)]
    pub easing_in: Option<Easing>,
}

impl<T: Clone + Default> KeyFrame<T> {
    pub fn from_value(value: T) -> Self {
        KeyFrame {
            start_value: value.clone(),
            end_value: value,
            start_frame: 0.0,
            end_frame: 0.0,
            easing_out: None,
            easing_in: None,
        }
    }
}

#[derive(PartialEq, Debug, SerJson)]
enum ArrayFromArrayOfNumber {
    List(Vec<f32>),
    Primitive(f32),
}

impl DeJson for ArrayFromArrayOfNumber {
    fn de_json(
        s: &mut nanoserde::DeJsonState,
        i: &mut core::str::Chars,
    ) -> Result<Self, nanoserde::DeJsonErr> {
        println!("de_json for ArrayFromArrayOfNumber");
        match s.tok {
            nanoserde::DeJsonTok::F64(_) => {
                let r = Self::Primitive(s.as_f64()? as f32);
                s.next_tok(i)?;
                Ok(r)
            }
            nanoserde::DeJsonTok::BlockOpen => {
                let r: Vec<f32> = DeJson::de_json(s, i)?;
                s.next_tok(i)?;
                Ok(Self::List(r))
            }
            _ => Err(s.err_token("F64 or [")),
        }
    }
}

impl From<&Vec<f32>> for ArrayFromArrayOfNumber {
    fn from(val: &Vec<f32>) -> ArrayFromArrayOfNumber {
        ArrayFromArrayOfNumber::List(val.clone())
    }
}
impl From<&ArrayFromArrayOfNumber> for Vec<f32> {
    fn from(val: &ArrayFromArrayOfNumber) -> Vec<f32> {
        match val {
            ArrayFromArrayOfNumber::Primitive(val) => vec![*val],
            ArrayFromArrayOfNumber::List(val) => val.clone(),
        }
    }
}
#[derive(SerJson, DeJson, Debug, Clone, Default, PartialEq)]
pub struct Easing {
    #[nserde(proxy = "ArrayFromArrayOfNumber")]
    pub x: Vec<f32>,
    #[nserde(proxy = "ArrayFromArrayOfNumber")]
    pub y: Vec<f32>,
}
