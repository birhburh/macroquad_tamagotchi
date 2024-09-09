// Based on https://github.com/zimond/lottie-rs/
// Also based on https://github.com/lottie/lottie-spec
// Also based on https://lottiefiles.github.io/lottie-docs/schema/
// Some of the fields I just added because they were in glaxnimate format itself

// nanoserde TODO:
// - implement ((serialize|deserialize)_)?with
// - implement untagged for enum
// - implement flatten for attribute
// - implemnet tag attribute for enum
// - implement skip_serializing_if = "Option::is_none")
// - Fix error "cannot add `{integer}` to `&model::Animated<Vec<model::Bezier>>`" when struct field name is 'd'
// - Implement something similar to serde_repr::Serialize_repr, serde_repr::Deserialize_repr

use {
    nanoserde::{DeJson, DeJsonErr, DeJsonState, DeJsonTok, SerJson, SerJsonState},
    std::{fmt::Debug, str::Chars, vec},
};

#[derive(Clone, Debug)]
pub enum Value {
    Primitive(f32),
    List(Vec<f32>),
    Bezier(Bezier),
    ComplexBezier(Vec<Bezier>),
    TextDocument(TextDocument),
}

impl Value {
    pub(crate) fn as_f32_vec(&self) -> Option<Vec<f32>> {
        Some(match self {
            Value::Primitive(p) => vec![*p],
            Value::List(l) => l.clone(),
            _ => return None,
        })
    }
}

impl DeJson for Value {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        match s.tok {
            DeJsonTok::U64(v) => {
                s.next_tok(i)?;
                Ok(Self::Primitive(v as f32))
            }
            DeJsonTok::BlockOpen => {
                s.next_tok(i)?;
                match s.tok {
                    DeJsonTok::F64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::List(res))
                    }
                    DeJsonTok::U64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::List(res))
                    }
                    DeJsonTok::CurlyOpen => {
                        let mut res = vec![DeJson::de_json(s, i)?];

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::ComplexBezier(res))
                    }
                    _ => Err(s.err_token("U64 or {")),
                }
            }
            _ => Err(s.err_token("U64 or [")),
        }
    }
}

impl SerJson for Value {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Primitive(f0) => {
                f0.ser_json(d, s);
            }
            Self::List(f0) => {
                f0.ser_json(d, s);
            }
            _ => todo!(),
        }
    }
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextDocument {
    #[nserde(rename = "t")]
    pub value: String,
    #[nserde(rename = "f")]
    pub font_name: String,
    #[nserde(rename = "s")]
    pub size: f32,
    #[nserde(
        rename = "fc",
        deserialize_with = "array_to_rgba",
        serialize_with = "array_from_rgba",
        default
    )]
    pub fill_color: Rgba,
    #[nserde(
        rename = "sc",
        deserialize_with = "array_to_rgba",
        serialize_with = "array_from_rgba",
        default
    )]
    stroke_color: Rgba,
    #[nserde(rename = "sw", default)]
    stroke_width: f32,
    #[nserde(rename = "of", default)]
    stroke_above_fill: bool,
    #[nserde(rename = "lh", default)]
    line_height: Option<f32>,
    #[nserde(rename = "j", default)]
    pub justify: TextJustify,
    #[nserde(rename = "ls", default)]
    pub baseline_shift: f32,
    // TODO:
    #[nserde(default)]
    sz: Vec<f32>,
    #[nserde(default)]
    ps: Vec<f32>,
    #[nserde(default)]
    ca: TextCaps,
}

impl Default for TextDocument {
    fn default() -> Self {
        TextDocument {
            font_name: String::new(),
            size: 14.0,
            fill_color: Rgba::new_u8(0, 0, 0, 255),
            stroke_color: Rgba::new_u8(0, 0, 0, 255),
            stroke_width: 0.0,
            stroke_above_fill: false,
            line_height: None,
            baseline_shift: 0.0,
            value: String::new(),
            justify: TextJustify::Left,
            sz: vec![],
            ps: vec![],
            ca: TextCaps::Regular,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextJustify {
    Left = 0,
    Right = 1,
    Center = 2,
    LastLineLeft = 3,
    LastLineRight = 4,
    LastLineCenter = 5,
    LastLineFull = 6,
}

impl Default for TextJustify {
    fn default() -> Self {
        TextJustify::Left
    }
}

impl DeJson for TextJustify {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    0 => Ok(Self::Left),
                    1 => Ok(Self::Right),
                    2 => Ok(Self::Center),
                    3 => Ok(Self::LastLineLeft),
                    4 => Ok(Self::LastLineRight),
                    5 => Ok(Self::LastLineCenter),
                    6 => Ok(Self::LastLineFull),
                    _ => Err(s.err_range("0..6")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for TextJustify {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Left => 0.ser_json(d, s),
            Self::Right => 1.ser_json(d, s),
            Self::Center => 2.ser_json(d, s),
            Self::LastLineLeft => 3.ser_json(d, s),
            Self::LastLineRight => 4.ser_json(d, s),
            Self::LastLineCenter => 5.ser_json(d, s),
            Self::LastLineFull => 6.ser_json(d, s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextCaps {
    Regular = 0,
    AllCaps = 1,
    SmallCaps = 2,
}

impl Default for TextCaps {
    fn default() -> Self {
        TextCaps::Regular
    }
}

impl DeJson for TextCaps {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    0 => Ok(Self::Regular),
                    1 => Ok(Self::AllCaps),
                    2 => Ok(Self::SmallCaps),
                    _ => Err(s.err_range("0..2")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for TextCaps {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Regular => 0.ser_json(d, s),
            Self::AllCaps => 1.ser_json(d, s),
            Self::SmallCaps => 2.ser_json(d, s),
        }
    }
}

pub trait FromTo<T> {
    fn from(v: T) -> Self;
    fn to(self) -> T;
}

impl FromTo<Value> for f32 {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        v[0]
    }

    fn to(self) -> Value {
        Value::Primitive(self)
    }
}

mod vector_2_d {
    use {
        nanoserde::{DeJson, DeJsonErr, DeJsonState, SerJson, SerJsonState},
        std::str::Chars,
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

impl FromTo<Value> for Vector2D {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        Vector2D::new(v[0], v.get(1).cloned().unwrap_or(0.0))
    }

    fn to(self) -> Value {
        todo!()
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
#[nserde(serialize_none_as_null)]
pub struct Model {
    #[nserde(rename = "nm")]
    pub name: Option<String>,
    #[nserde(rename = "mn", skip)]
    pub match_name: Option<String>,
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
    #[nserde(default)]
    pub assets: Vec<Asset>,
    #[nserde(default)]
    pub fonts: FontList,
}

#[derive(Debug, Clone)]
pub enum Asset {
    Media(Media),
    Precomposition(Precomposition),
}

impl DeJson for Asset {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        match s.tok {
            DeJsonTok::CurlyOpen => {
                let mut _asset = None;
                let mut _name = None;
                let mut _match_name = None;
                let mut _id = None;
                s.next_tok(i)?;
                {
                    while let Some(_) = s.next_str() {
                        match AsRef::<str>::as_ref(&s.strbuf) {
                            "e" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Media(media)) => {
                                        if let Some(match_name) = _match_name {
                                            media.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(name) = _name {
                                            media.name = name;
                                            _name = None;
                                        }
                                        if let Some(id) = _id {
                                            media.id = id;
                                            _id = None;
                                        }
                                        let embedded = DeJson::de_json(s, i)?;
                                        media.embedded = From::<&BoolFromInt>::from(&embedded);
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "fr" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    None => {
                                        let mut precomposition = Precomposition::default();
                                        if let Some(name) = _name {
                                            precomposition.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            precomposition.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            precomposition.id = id;
                                            _id = None;
                                        }
                                        precomposition.frame_rate = DeJson::de_json(s, i)?;
                                        _asset = Some(Asset::Precomposition(precomposition));
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        if let Some(name) = _name {
                                            precomposition.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            precomposition.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            precomposition.id = id;
                                            _id = None;
                                        }
                                        precomposition.frame_rate = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "h" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Media(media)) => {
                                        if let Some(name) = _name {
                                            media.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            media.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            media.id = id;
                                            _id = None;
                                        }
                                        media.height = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        if let Some(name) = _name {
                                            precomposition.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            precomposition.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            precomposition.id = id;
                                            _id = None;
                                        }
                                        precomposition.height = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "id" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    None => {
                                        _id = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Media(media)) => {
                                        media.id = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        precomposition.id = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "layers" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Precomposition(precomposition)) => {
                                        if let Some(name) = _name {
                                            precomposition.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            precomposition.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            precomposition.id = id;
                                            _id = None;
                                        }
                                        precomposition.layers = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "mn" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    None => {
                                        _match_name = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Media(media)) => {
                                        media.match_name = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        precomposition.match_name = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "nm" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    None => {
                                        _name = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Media(media)) => {
                                        media.name = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        precomposition.name = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "p" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Media(media)) => {
                                        if let Some(name) = _name {
                                            media.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            media.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            media.id = id;
                                            _id = None;
                                        }
                                        media.filename = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "u" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Media(media)) => {
                                        if let Some(name) = _name {
                                            media.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            media.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            media.id = id;
                                            _id = None;
                                        }
                                        media.pwd = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            "w" => {
                                s.next_colon(i)?;
                                match _asset.as_mut() {
                                    Some(Asset::Media(media)) => {
                                        if let Some(name) = _name {
                                            media.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            media.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            media.id = id;
                                            _id = None;
                                        }
                                        media.width = DeJson::de_json(s, i)?;
                                    }
                                    Some(Asset::Precomposition(precomposition)) => {
                                        if let Some(name) = _name {
                                            precomposition.name = name;
                                            _name = None;
                                        }
                                        if let Some(match_name) = _match_name {
                                            precomposition.match_name = match_name;
                                            _match_name = None;
                                        }
                                        if let Some(id) = _id {
                                            precomposition.id = id;
                                            _id = None;
                                        }
                                        precomposition.width = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            _ => de_unreachable(s),
                        }
                        s.eat_comma_curly(i)?
                    }
                    s.curly_close(i)?;
                }
                s.eat_comma_block(i)?;
                println!("END de_json for Asset");
                Ok(_asset.expect("Not supported asset type"))
            }
            _ => Err(s.err_token("{")),
        }
    }
}

impl SerJson for Asset {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Media(media) => {
                media.ser_json(d, s);
            }
            Self::Precomposition(precomposition) => {
                precomposition.ser_json(d, s);
            }
            _ => todo!(),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Precomposition {
    pub id: String,
    pub layers: Vec<Layer>,
    #[nserde(rename = "nm")]
    name: Option<String>,
    #[nserde(rename = "mn", skip)]
    pub match_name: Option<String>,
    #[nserde(rename = "w", default, skip)]
    pub width: Option<u32>,
    #[nserde(rename = "h", default, skip)]
    pub height: Option<u32>,
    #[nserde(rename = "fr")]
    pub frame_rate: Option<f32>,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct FontList {
    pub list: Vec<Font>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Font {
    #[nserde(default)]
    ascent: Option<f32>,
    #[nserde(rename = "fFamily")]
    pub family: String,
    #[nserde(rename = "fName")]
    pub name: String,
    #[nserde(rename = "fStyle")]
    style: String,
    #[nserde(rename = "fPath", default)]
    pub path: Option<String>,
    #[nserde(rename = "fWeight")]
    weight: Option<String>,
    #[nserde(default)]
    pub origin: FontPathOrigin,
    #[nserde(rename = "fClass", default)]
    class: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum FontPathOrigin {
    #[default]
    Local = 0,
    CssUrl = 1,
    ScriptUrl = 2,
    FontUrl = 3,
}

impl DeJson for FontPathOrigin {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    0 => Ok(Self::Local),
                    1 => Ok(Self::CssUrl),
                    2 => Ok(Self::ScriptUrl),
                    3 => Ok(Self::FontUrl),
                    _ => Err(s.err_range("0..3")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for FontPathOrigin {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Local => 0.ser_json(d, s),
            Self::CssUrl => 1.ser_json(d, s),
            Self::ScriptUrl => 2.ser_json(d, s),
            Self::FontUrl => 3.ser_json(d, s),
        }
    }
}

#[derive(PartialEq, Debug, DeJson, SerJson)]
#[nserde(transparent)]
// deserialize_with = "bool_from_int"
pub struct BoolFromInt(u32);

impl From<&bool> for BoolFromInt {
    fn from(e: &bool) -> BoolFromInt {
        BoolFromInt(*e as u32)
    }
}
impl From<&BoolFromInt> for bool {
    fn from(n: &BoolFromInt) -> bool {
        match n.0 {
            0 => false,
            1 => true,
            _ => panic!("wrong number for boolean representation"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    is_3d: bool,
    pub hidden: bool,
    pub index: Option<u32>,
    pub parent_index: Option<u32>,
    pub id: u32,
    pub auto_orient: bool,
    pub start_frame: f32,
    pub end_frame: f32,
    pub start_time: f32,
    pub name: Option<String>,
    pub match_name: Option<String>,
    pub transform: Option<Transform>,
    pub content: LayerContent,
    pub time_stretch: Option<f32>,
    pub matte_mode: Option<MatteMode>,
    pub blend_mode: Option<BlendMode>,
    pub has_mask: bool,
    pub masks_properties: Vec<Mask>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MatteMode {
    Normal = 0,
    Alpha = 1,
    InvertedAlpha = 2,
    Luma = 3,
    InvertedLuma = 4,
}

impl DeJson for MatteMode {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    0 => Ok(Self::Normal),
                    1 => Ok(Self::Alpha),
                    2 => Ok(Self::InvertedAlpha),
                    3 => Ok(Self::Luma),
                    4 => Ok(Self::InvertedLuma),
                    _ => Err(s.err_range("0..4")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for MatteMode {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Normal => 0.ser_json(d, s),
            Self::Alpha => 1.ser_json(d, s),
            Self::InvertedAlpha => 2.ser_json(d, s),
            Self::Luma => 3.ser_json(d, s),
            Self::InvertedLuma => 4.ser_json(d, s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BlendMode {
    Normal = 0,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HighLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Add,
    HardMix,
}

impl DeJson for BlendMode {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    0 => Ok(Self::Normal),
                    1 => Ok(Self::Multiply),
                    2 => Ok(Self::Screen),
                    3 => Ok(Self::Overlay),
                    4 => Ok(Self::Darken),
                    5 => Ok(Self::Lighten),
                    6 => Ok(Self::ColorDodge),
                    7 => Ok(Self::ColorBurn),
                    8 => Ok(Self::HighLight),
                    9 => Ok(Self::SoftLight),
                    10 => Ok(Self::Difference),
                    11 => Ok(Self::Exclusion),
                    12 => Ok(Self::Hue),
                    13 => Ok(Self::Saturation),
                    14 => Ok(Self::Color),
                    15 => Ok(Self::Luminosity),
                    16 => Ok(Self::Add),
                    17 => Ok(Self::HardMix),
                    _ => Err(s.err_range("0..17")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for BlendMode {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Normal => 0.ser_json(d, s),
            Self::Multiply => 1.ser_json(d, s),
            Self::Screen => 2.ser_json(d, s),
            Self::Overlay => 3.ser_json(d, s),
            Self::Darken => 4.ser_json(d, s),
            Self::Lighten => 5.ser_json(d, s),
            Self::ColorDodge => 6.ser_json(d, s),
            Self::ColorBurn => 7.ser_json(d, s),
            Self::HighLight => 8.ser_json(d, s),
            Self::SoftLight => 9.ser_json(d, s),
            Self::Difference => 10.ser_json(d, s),
            Self::Exclusion => 11.ser_json(d, s),
            Self::Hue => 12.ser_json(d, s),
            Self::Saturation => 13.ser_json(d, s),
            Self::Color => 14.ser_json(d, s),
            Self::Luminosity => 15.ser_json(d, s),
            Self::Add => 16.ser_json(d, s),
            Self::HardMix => 17.ser_json(d, s),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Mask {
    #[nserde(rename = "nm", default)]
    pub name: String,
    #[nserde(rename = "inv", default)]
    inverted: bool,
    #[nserde(rename = "pt")]
    pub points: Animated<Vec<Bezier>>,
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    pub mode: MaskMode,
    #[nserde(rename = "e", default)]
    expand: Option<Animated<f32>>,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
pub enum MaskMode {
    #[nserde(rename = "n")]
    None,
    #[nserde(rename = "a")]
    Add,
    #[nserde(rename = "s")]
    Subtract,
    #[nserde(rename = "i")]
    Intersect,
    #[nserde(rename = "l")]
    Lighten,
    #[nserde(rename = "d")]
    Darken,
    #[nserde(rename = "f")]
    Difference,
}

impl DeJson for Layer {
    #[allow(clippy::ignored_unit_patterns)]
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        Ok({
            let mut _is_3d = None;
            let mut _hidden = None;
            let mut _index = None;
            let mut _parent_index = None;
            let mut _auto_orient = None;
            let mut _start_frame = None;
            let mut _end_frame = None;
            let mut _time_stretch = None;
            let mut _matte_mode = None;
            let mut _blend_mode = None;
            let mut _has_mask = None;
            let mut _masks_properties = None;
            let mut _start_time = None;
            let mut _name = None;
            let mut _match_name = None;
            let mut _transform = None;
            let mut _content = None;

            s.curly_open(i)?;
            while let Some(_) = s.next_str() {
                match AsRef::<str>::as_ref(&s.strbuf) {
                    "a" => {
                        s.next_colon(i)?;
                        // assert!(matches!(
                        //     _content,
                        //     None | Some(LayerContent::Text(_))
                        // ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "ao" => {
                        s.next_colon(i)?;
                        _auto_orient = Some(DeJson::de_json(s, i)?);
                    }
                    "bm" => {
                        s.next_colon(i)?;
                        _blend_mode = Some(DeJson::de_json(s, i)?);
                    }
                    "d" => {
                        s.next_colon(i)?;
                        // assert!(matches!(
                        //     _content,
                        //     None | Some(LayerContent::Text(_))
                        // ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "ddd" => {
                        s.next_colon(i)?;
                        _is_3d = Some(DeJson::de_json(s, i)?);
                    }
                    "e" => {
                        s.next_colon(i)?;
                        assert!(matches!(_content, None | Some(LayerContent::Media(_))));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "h" => {
                        s.next_colon(i)?;
                        match _content.as_mut() {
                            Some(LayerContent::PreCompositionRef(PreCompositionRef {
                                ref mut height,
                                ..
                            })) => {
                                *height = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "hasMask" => {
                        s.next_colon(i)?;
                        _has_mask = Some(DeJson::de_json(s, i)?);
                    }
                    "hd" => {
                        s.next_colon(i)?;
                        _hidden = Some(DeJson::de_json(s, i)?);
                    }
                    "ind" => {
                        s.next_colon(i)?;
                        _index = Some(DeJson::de_json(s, i)?);
                    }
                    "ip" => {
                        s.next_colon(i)?;
                        _start_frame = Some(DeJson::de_json(s, i)?);
                    }
                    "ks" => {
                        s.next_colon(i)?;
                        _transform = Some(DeJson::de_json(s, i)?);
                    }
                    "m" => {
                        s.next_colon(i)?;
                        // assert!(matches!(
                        //     _content,
                        //     None | Some(LayerContent::Text(_))
                        // ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "masksProperties" => {
                        s.next_colon(i)?;
                        _masks_properties = Some(DeJson::de_json(s, i)?);
                    }
                    "nm" => {
                        s.next_colon(i)?;
                        _name = Some(DeJson::de_json(s, i)?);
                    }
                    "mn" => {
                        s.next_colon(i)?;
                        _match_name = Some(DeJson::de_json(s, i)?);
                    }
                    "op" => {
                        s.next_colon(i)?;
                        _end_frame = Some(DeJson::de_json(s, i)?);
                    }
                    "p" => {
                        s.next_colon(i)?;
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "parent" => {
                        s.next_colon(i)?;
                        _parent_index = Some(DeJson::de_json(s, i)?);
                    }
                    "refId" => {
                        s.next_colon(i)?;
                        match _content.as_mut() {
                            Some(LayerContent::PreCompositionRef(PreCompositionRef {
                                ref mut ref_id,
                                ..
                            })) => {
                                *ref_id = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "sc" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::SolidColor { .. })
                        ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "sh" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::SolidColor { .. })
                        ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "shapes" => {
                        s.next_colon(i)?;
                        let parsed_shapes = DeJson::de_json(s, i)?;
                        match _content.as_mut() {
                            None => {
                                _content = Some(LayerContent::Shape(ShapeGroup {
                                    shapes: parsed_shapes,
                                }));
                            }
                            Some(LayerContent::Shape(ShapeGroup { ref mut shapes })) => {
                                *shapes = parsed_shapes
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "sr" => {
                        s.next_colon(i)?;
                        _time_stretch = Some(DeJson::de_json(s, i)?);
                    }
                    "st" => {
                        s.next_colon(i)?;
                        _start_time = Some(DeJson::de_json(s, i)?);
                    }
                    "sw" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::SolidColor { .. })
                        ));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "tm" => {
                        s.next_colon(i)?;
                        let time_remapping = DeJson::de_json(s, i)?;
                        match _content.as_mut() {
                            None => {
                                _content =
                                    Some(LayerContent::PreCompositionRef(PreCompositionRef {
                                        time_remapping,
                                        ..Default::default()
                                    }));
                            }
                            Some(LayerContent::PreCompositionRef(value)) => {
                                value.time_remapping = time_remapping;
                            }
                            _ => unreachable!(),
                        }
                    }
                    "tt" => {
                        s.next_colon(i)?;
                        _matte_mode = Some(DeJson::de_json(s, i)?);
                    }
                    "ty" => {
                        s.next_colon(i)?;
                        match s.tok {
                            DeJsonTok::U64(v) => match v {
                                0 => {
                                    _content =
                                        Some(LayerContent::PreCompositionRef(Default::default()))
                                }
                                1 => {
                                    _content = Some(LayerContent::SolidColor {
                                        color: Default::default(),
                                        width: Default::default(),
                                        height: Default::default(),
                                    })
                                }
                                2 => _content = Some(LayerContent::MediaRef(Default::default())),
                                3 => _content = Some(LayerContent::Empty),
                                4 => _content = Some(LayerContent::Shape(Default::default())),
                                6 => _content = Some(LayerContent::MediaRef(Default::default())),
                                _ => de_unreachable(s),
                            },
                            _ => de_unreachable(s),
                        }
                        s.next_tok(i)?;
                    }
                    "u" => {
                        s.next_colon(i)?;
                        assert!(matches!(_content, None | Some(LayerContent::Media(_))));
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "w" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::PreCompositionRef(_))
                                | Some(LayerContent::MediaRef(_))
                        ));
                        match _content.as_mut() {
                            Some(LayerContent::PreCompositionRef(PreCompositionRef {
                                ref mut width,
                                ..
                            })) => {
                                *width = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    _ => de_unreachable(s),
                }
                s.eat_comma_curly(i)?
            }
            s.curly_close(i)?;
            Layer {
                is_3d: {
                    if let Some(t) = _is_3d {
                        From::<&BoolFromInt>::from(&t)
                    } else {
                        Default::default()
                    }
                },
                hidden: {
                    if let Some(t) = _hidden {
                        t
                    } else {
                        Default::default()
                    }
                },
                index: {
                    if let Some(t) = _index {
                        t
                    } else {
                        None
                    }
                },
                parent_index: {
                    if let Some(t) = _parent_index {
                        t
                    } else {
                        None
                    }
                },
                id: Default::default(),
                auto_orient: {
                    if let Some(t) = _auto_orient {
                        From::<&BoolFromInt>::from(&t)
                    } else {
                        Default::default()
                    }
                },
                start_frame: {
                    if let Some(t) = _start_frame {
                        t
                    } else {
                        return Err(s.err_nf("start_frame"));
                    }
                },
                end_frame: {
                    if let Some(t) = _end_frame {
                        t
                    } else {
                        return Err(s.err_nf("end_frame"));
                    }
                },
                time_stretch: {
                    if let Some(t) = _time_stretch {
                        t
                    } else {
                        None
                    }
                },
                matte_mode: {
                    if let Some(t) = _matte_mode {
                        t
                    } else {
                        None
                    }
                },
                blend_mode: {
                    if let Some(t) = _blend_mode {
                        t
                    } else {
                        None
                    }
                },
                has_mask: {
                    if let Some(t) = _has_mask {
                        t
                    } else {
                        Default::default()
                    }
                },
                masks_properties: {
                    if let Some(t) = _masks_properties {
                        t
                    } else {
                        Default::default()
                    }
                },
                start_time: {
                    if let Some(t) = _start_time {
                        t
                    } else {
                        return Err(s.err_nf("start_time"));
                    }
                },
                name: {
                    if let Some(t) = _name {
                        t
                    } else {
                        None
                    }
                },
                match_name: {
                    if let Some(t) = _match_name {
                        t
                    } else {
                        None
                    }
                },
                transform: {
                    if let Some(t) = _transform {
                        t
                    } else {
                        None
                    }
                },
                content: {
                    if let Some(t) = _content {
                        t
                    } else {
                        return Err(s.err_nf("content"));
                    }
                },
            }
        })
    }
}

impl SerJson for Layer {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.st_pre();
        let mut first_field_was_serialized = false;
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ddd");
        {
            let proxy: BoolFromInt = Into::into(&self.is_3d);
            proxy
        }
        .ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "hd");
        self.hidden.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ind");
        if let Some(t) = &self.index {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "parent");
        if let Some(t) = &self.parent_index {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ao");
        {
            let proxy: BoolFromInt = Into::into(&self.auto_orient);
            proxy
        }
        .ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ip");
        self.start_frame.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "op");
        self.end_frame.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "st");
        self.start_time.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "nm");
        if let Some(t) = &self.name {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ks");
        if let Some(t) = &self.transform {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ty");
        match &self.content {
            LayerContent::Shape(shape_group) => {
                i32::ser_json(&4, d + 1, s);
                s.conl();
                s.field(d + 1, "shapes");
                shape_group.shapes.ser_json(d + 1, s);
            }
            LayerContent::PreCompositionRef(pre_composition_ref) => {
                i32::ser_json(&0, d + 1, s);
                s.conl();
                s.field(d + 1, "refId");
                pre_composition_ref.ref_id.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "w");
                pre_composition_ref.width.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "h");
                pre_composition_ref.height.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "tm");
                pre_composition_ref.time_remapping.ser_json(d + 1, s);
            }
            _ => unreachable!(),
        }
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "tt");
        if let Some(t) = &self.matte_mode {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "bm");
        if let Some(t) = &self.blend_mode {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "hasMask");
        self.has_mask.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "masksProperties");
        self.masks_properties.ser_json(d + 1, s);
        s.st_post(d);
    }
}

#[derive(Debug, Clone)]
pub enum LayerContent {
    PreCompositionRef(PreCompositionRef),
    SolidColor {
        color: Rgba,
        height: f32,
        width: f32,
    },
    MediaRef(MediaRef),
    Empty,
    Shape(ShapeGroup),
    // Text(TextAnimationData),
    Media(Media),
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct PreCompositionRef {
    #[nserde(rename = "refId")]
    pub ref_id: String,
    #[nserde(rename = "w")]
    width: u32,
    #[nserde(rename = "h")]
    height: u32,
    #[nserde(rename = "tm")]
    pub time_remapping: Option<Animated<f32>>,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new_f32(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }

    pub fn new_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct MediaRef {
    #[nserde(rename = "refId")]
    pub ref_id: String,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct ShapeGroup {
    pub shapes: Vec<ShapeLayer>,
}

#[derive(Debug, Clone)]
pub struct ShapeLayer {
    pub name: Option<String>,
    pub hidden: bool,
    pub shape: Shape,
}

impl SerJson for ShapeLayer {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.st_pre();
        let mut first_field_was_serialized = false;
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "nm");
        if let Some(t) = &self.name {
            t.ser_json(d + 1, s);
        } else {
            Option::<i32>::ser_json(&None, d + 1, s);
        };
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "hd");
        self.hidden.ser_json(d + 1, s);
        if first_field_was_serialized {
            s.conl();
        }
        first_field_was_serialized = true;
        s.field(d + 1, "ty");
        match &self.shape {
            Shape::Rectangle(rectangle) => {
                String::ser_json(&"rc".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "d");
                rectangle.direction.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "p");
                rectangle.position.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "s");
                rectangle.size.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "r");
                rectangle.radius.ser_json(d + 1, s);
            }
            Shape::Ellipse(ellipse) => {
                String::ser_json(&"el".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "d");
                ellipse.direction.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "p");
                ellipse.position.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "s");
                ellipse.size.ser_json(d + 1, s);
            }
            Shape::Group { shapes } => {
                String::ser_json(&"gr".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "it");
                shapes.ser_json(d + 1, s);
            }
            Shape::Fill(fill) => {
                String::ser_json(&"fl".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "o");
                fill.opacity.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "c");
                fill.color.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "r");
                fill.fill_rule.ser_json(d + 1, s);
            }
            Shape::Transform(transform) => {
                String::ser_json(&"tr".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "a");
                transform.anchor.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "p");
                transform.position.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "s");
                transform.scale.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "r");
                transform.rotation.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "o");
                transform.opacity.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "sk");
                transform.skew.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "sa");
                transform.skew_axis.ser_json(d + 1, s);
            }
            Shape::Path { data, .. } => {
                String::ser_json(&"sh".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "ks");
                data.ser_json(d + 1, s);
            }
            Shape::Stroke(stroke) => {
                String::ser_json(&"st".into(), d + 1, s);
                s.conl();
                s.field(d + 1, "lc");
                stroke.line_cap.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "lj");
                stroke.line_join.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "ml");
                stroke.miter_limit.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "o");
                stroke.opacity.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "w");
                stroke.width.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "d");
                stroke.dashes.ser_json(d + 1, s);
                s.conl();
                s.field(d + 1, "c");
                stroke.color.ser_json(d + 1, s);
            }
            _ => unreachable!(),
        }
        s.st_post(d);
    }
}

fn de_unreachable(s: &mut DeJsonState) {
    panic!("Should not be here!: {}:{}", s.line, s.col);
}

impl DeJson for ShapeLayer {
    #[allow(clippy::ignored_unit_patterns)]
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        Ok({
            let mut _name = None;
            let mut _hidden = None;
            let mut _shape = None;
            let mut _direction = None;
            let mut _dashes = None;
            s.curly_open(i)?;
            while let Some(_) = s.next_str() {
                match AsRef::<str>::as_ref(&s.strbuf) {
                    "nm" => {
                        s.next_colon(i)?;
                        _name = Some(DeJson::de_json(s, i)?);
                    }
                    "hd" => {
                        s.next_colon(i)?;
                        _hidden = Some(DeJson::de_json(s, i)?);
                    }
                    "ty" => {
                        s.next_colon(i)?;
                        if let Some(_) = s.next_str() {
                            match AsRef::<str>::as_ref(&s.strbuf) {
                                "rc" => {
                                    match _shape.as_mut() {
                                        None => {
                                            let mut rect = Rectangle::default();
                                            if let Some(direction) = _direction {
                                                rect.direction = direction;
                                                _direction = None;
                                            }
                                            _shape = Some(Shape::Rectangle(rect));
                                        }
                                        Some(Shape::Rectangle(rect)) => {
                                            if let Some(direction) = _direction {
                                                rect.direction = direction;
                                                _direction = None;
                                            }
                                        }
                                        _ => return Err(s.err_nf("start_time")),
                                    }
                                }
                                "el" => {
                                    match _shape.as_mut() {
                                        None => {
                                            let mut ellipse = Ellipse::default();
                                            if let Some(direction) = _direction {
                                                ellipse.direction = direction;
                                                _direction = None;
                                            }
                                            _shape = Some(Shape::Ellipse(ellipse));
                                        }
                                        Some(Shape::Ellipse(ellipse)) => {
                                            if let Some(direction) = _direction {
                                                ellipse.direction = direction;
                                                _direction = None;
                                            }
                                        }
                                        _ => de_unreachable(s),
                                    }
                                }
                                "gr" => {
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Group {
                                                shapes: Default::default(),
                                            });
                                        }
                                        Some(Shape::Group { .. }) => (),
                                        _ => de_unreachable(s),
                                    }
                                }
                                "fl" => {
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Fill(Default::default()));
                                        }
                                        Some(Shape::Fill { .. }) => (),
                                        _ => de_unreachable(s),
                                    }
                                }
                                "tr" => {
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Transform(Default::default()));
                                        }
                                        Some(Shape::Transform { .. }) => (),
                                        _ => de_unreachable(s),
                                    }
                                }
                                "sh" => {
                                    match _shape.as_mut() {
                                        None => {
                                            let path = Shape::Path {
                                                data: Default::default(),
                                                direction: if let Some(direction) = _direction {
                                                    _direction = None;
                                                    direction
                                                } else {
                                                    Default::default()
                                                },
                                                text_range: Default::default(),
                                            };
                                            _shape = Some(path);
                                        }
                                        Some(Shape::Path {
                                            ref mut direction, ..
                                        }) => {
                                            if let Some(saved_direction) = _direction {
                                                *direction = saved_direction;
                                                _direction = None;
                                            }
                                        }
                                        _ => de_unreachable(s),
                                    }
                                }
                                "st" => {
                                    match _shape.as_mut() {
                                        None => {
                                            let mut stroke = Stroke::default();
                                            if let Some(dashes) = _dashes {
                                                stroke.dashes = dashes;
                                                _dashes = None;
                                            }
                                            _shape = Some(Shape::Stroke(stroke));
                                        }
                                        Some(Shape::Stroke(stroke)) => {
                                            if let Some(dashes) = _dashes {
                                                stroke.dashes = dashes;
                                                _dashes = None;
                                            }
                                        }
                                        _ => de_unreachable(s),
                                    }
                                }
                                "gf" => {
                                    match _shape.as_mut() {
                                        None => {
                                            _shape =
                                                Some(Shape::GradientFill(GradientFill::default()));
                                        }
                                        Some(Shape::GradientFill(_)) => (),
                                        _ => de_unreachable(s),
                                    }
                                }
                                _ => de_unreachable(s),
                            }
                            s.next_tok(i)?;
                        }
                    }
                    "it" => {
                        s.next_colon(i)?;
                        let shapes = DeJson::de_json(s, i)?;
                        match _shape.as_mut() {
                            None => {
                                _shape = Some(Shape::Group { shapes });
                            }
                            Some(Shape::Group {
                                shapes: shapes_field,
                            }) => {
                                *shapes_field = shapes;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "a" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.anchor = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "c" => {
                        s.next_colon(i)?;
                        let color = DeJson::de_json(s, i)?;
                        match _shape.as_mut() {
                            Some(Shape::Fill(fill)) => {
                                fill.color = color;
                            }
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.color = color;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "d" => {
                        s.next_colon(i)?;

                        match _shape.as_mut() {
                            None => {
                                match s.tok {
                                    DeJsonTok::U64(_) => {
                                        _direction = DeJson::de_json(s, i)?;
                                    }
                                    DeJsonTok::BlockOpen => {
                                        _dashes = DeJson::de_json(s, i)?;
                                    }
                                    _ => de_unreachable(s),
                                }
                            }
                            Some(Shape::Rectangle(rectangle)) => {
                                rectangle.direction = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Ellipse(ellipse)) => {
                                ellipse.direction = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Path {
                                ref mut direction, ..
                            }) => {
                                *direction = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.dashes = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "e" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::GradientFill(gradient_fill)) => {
                                gradient_fill.gradient.end = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "g" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::GradientFill(gradient_fill)) => {
                                let colors = DeJson::de_json(s, i)?;
                                gradient_fill.gradient.colors =
                                    From::<&ColorListHelper>::from(&colors);
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "ks" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Path {
                                ref mut data,
                                ref mut direction,
                                ..
                            }) => {
                                if let Some(saved_direction) = _direction {
                                    *direction = saved_direction;
                                    _direction = None;
                                }
                                *data = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "lc" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.line_cap = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "lj" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.line_join = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "ml" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.miter_limit = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "o" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::GradientFill(gradient_fill)) => {
                                gradient_fill.opacity = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Fill(fill)) => {
                                fill.opacity = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Transform(transform)) => {
                                transform.opacity = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.opacity = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "p" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Rectangle(rectangle)) => {
                                if let Some(direction) = _direction {
                                    rectangle.direction = direction;
                                    _direction = None;
                                }
                                rectangle.position = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Ellipse(ellipse)) => {
                                if let Some(direction) = _direction {
                                    ellipse.direction = direction;
                                    _direction = None;
                                }
                                ellipse.position = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Transform(transform)) => {
                                transform.position = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "r" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Fill(fill)) => {
                                fill.fill_rule = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Transform(transform)) => {
                                transform.rotation = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Rectangle(rectangle)) => {
                                if let Some(direction) = _direction {
                                    rectangle.direction = direction;
                                    _direction = None;
                                }
                                rectangle.radius = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::RoundedCorners { radius }) => {
                                *radius = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::GradientFill(gradient_fill)) => {
                                gradient_fill.fill_rule = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "s" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Rectangle(rectangle)) => {
                                if let Some(direction) = _direction {
                                    rectangle.direction = direction;
                                    _direction = None;
                                }
                                rectangle.size = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Ellipse(ellipse)) => {
                                if let Some(direction) = _direction {
                                    ellipse.direction = direction;
                                    _direction = None;
                                }
                                ellipse.size = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::Transform(transform)) => {
                                transform.scale = DeJson::de_json(s, i)?;
                            }
                            Some(Shape::GradientFill(gradient_fill)) => {
                                gradient_fill.gradient.start = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "sk" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.skew = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "sa" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.skew_axis = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "t" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::GradientFill(gradient_fill)) => {
                                gradient_fill.gradient.gradient_ty = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    "w" => {
                        s.next_colon(i)?;
                        match _shape.as_mut() {
                            Some(Shape::Stroke(stroke)) => {
                                if let Some(dashes) = _dashes {
                                    stroke.dashes = dashes;
                                    _dashes = None;
                                }
                                stroke.width = DeJson::de_json(s, i)?;
                            }
                            _ => de_unreachable(s),
                        }
                    }
                    _ => {
                        s.next_colon(i)?;
                        s.whole_field(i)?;
                    }
                }
                s.eat_comma_curly(i)?
            }
            s.curly_close(i)?;
            ShapeLayer {
                name: {
                    if let Some(t) = _name {
                        t
                    } else {
                        None
                    }
                },
                hidden: {
                    if let Some(t) = _hidden {
                        t
                    } else {
                        Default::default()
                    }
                },
                shape: {
                    if let Some(t) = _shape {
                        t
                    } else {
                        Default::default()
                    }
                },
            }
        })
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
#[nserde(tag = "ty")]
pub enum Shape {
    #[nserde(rename = "rc")]
    Rectangle(Rectangle),
    #[nserde(rename = "el")]
    Ellipse(Ellipse),
    // #[nserde(rename = "sr")]
    // PolyStar(PolyStar),
    #[nserde(rename = "sh")]
    Path {
        #[nserde(rename = "ks")]
        data: Animated<Vec<Bezier>>,
        #[nserde(rename = "d", default)]
        direction: ShapeDirection,
        #[nserde(skip)]
        text_range: Option<TextRangeInfo>,
    },
    #[nserde(rename = "fl")]
    Fill(Fill),
    #[nserde(rename = "st")]
    Stroke(Stroke),
    #[nserde(rename = "gf")]
    GradientFill(GradientFill),
    // #[nserde(rename = "gs")]
    // GradientStroke(GradientStroke),
    #[nserde(rename = "gr")]
    Group {
        // TODO: add np property
        #[nserde(rename = "it")]
        shapes: Vec<ShapeLayer>,
    },
    #[nserde(rename = "tr")]
    Transform(Transform),
    // #[nserde(rename = "rp")]
    // Repeater {
    //     #[nserde(rename = "c")]
    //     copies: Animated<f32>,
    //     #[nserde(rename = "o")]
    //     offset: Animated<f32>,
    //     #[nserde(rename = "m")]
    //     composite: Composite,
    //     #[nserde(rename = "tr")]
    //     transform: RepeaterTransform,
    // },
    // #[nserde(rename = "tm")]
    // Trim(Trim),
    #[nserde(rename = "rd")]
    RoundedCorners {
        #[nserde(rename = "r")]
        radius: Animated<f32>,
    },
    #[nserde(rename = "pb")]
    PuckerBloat {
        #[nserde(rename = "a")]
        amount: Animated<f32>,
    },
    #[nserde(rename = "tw")]
    Twist {
        #[nserde(rename = "a")]
        angle: Animated<f32>,
        #[nserde(rename = "c")]
        center: Animated<Vector2D>,
    },
    // #[nserde(rename = "mm")]
    // Merge {
    //     #[nserde(rename = "mm")]
    //     mode: MergeMode,
    // },
    // #[nserde(rename = "op")]
    // OffsetPath {
    //     #[nserde(rename = "a")]
    //     amount: Animated<f32>,
    //     #[nserde(rename = "lj")]
    //     line_join: LineJoin,
    //     #[nserde(rename = "ml")]
    //     miter_limit: f32,
    // },
    // #[nserde(rename = "zz")]
    // ZigZag {
    //     #[nserde(rename = "r")]
    //     radius: Animated<f32>,
    //     #[nserde(rename = "s")]
    //     distance: Animated<f32>,
    //     #[nserde(rename = "pt")]
    //     ridges: Animated<f32>,
    // },
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Ellipse(Ellipse {
            direction: Default::default(),
            position: Default::default(),
            size: Default::default(),
        })
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Bezier {
    #[nserde(rename = "c", default)]
    pub closed: bool,
    #[nserde(rename = "v")]
    pub verticies: Vec<Vector2D>,
    #[nserde(rename = "i")]
    pub in_tangent: Vec<Vector2D>,
    #[nserde(rename = "o")]
    pub out_tangent: Vec<Vector2D>,
}

impl FromTo<Value> for Vec<Bezier> {
    fn from(v: Value) -> Self {
        match v {
            Value::ComplexBezier(b) => b,
            Value::Bezier(b) => vec![b],
            _ => todo!(),
        }
    }

    fn to(self) -> Value {
        Value::ComplexBezier(self)
    }
}

impl FromTo<Value> for Vec<f32> {
    fn from(v: Value) -> Self {
        match v {
            Value::Primitive(f) => vec![f],
            Value::List(l) => l,
            _ => todo!(),
        }
    }

    fn to(self) -> Value {
        Value::List(self)
    }
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextRangeInfo {
    #[nserde(skip)]
    pub value: Vec<Vec<char>>,
    pub index: (usize, usize), // line, char
    pub ranges: Vec<TextRange>,
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextRange {
    #[nserde(rename = "nm", default)]
    name: Option<String>,
    #[nserde(rename = "a", default)]
    pub style: Option<TextStyle>,
    #[nserde(rename = "s")]
    pub selector: TextRangeSelector,
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextRangeSelector {
    #[nserde(rename = "t", proxy = "BoolFromInt")]
    expressible: bool,
    #[nserde(rename = "xe")]
    max_ease: Animated<f32>,
    #[nserde(rename = "ne")]
    min_ease: Animated<f32>,
    #[nserde(rename = "a")]
    max_amount: Animated<f32>,
    #[nserde(rename = "b")]
    based_on: TextBased,
    #[nserde(rename = "rn", proxy = "BoolFromInt")]
    randomize: bool,
    #[nserde(rename = "sh")]
    shape: TextShape,
    #[nserde(rename = "o", default)]
    offset: Option<Animated<f32>>,
    #[nserde(rename = "r")]
    pub range_units: TextBased,
    #[nserde(rename = "sm", default)]
    selector_smoothness: Option<Animated<f32>>,
    #[nserde(rename = "s", default)]
    pub start: Option<Animated<f32>>,
    #[nserde(rename = "e", default)]
    pub end: Option<Animated<f32>>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextBased {
    Characters = 1,
    CharactersExcludingSpaces = 2,
    Words = 3,
    Lines = 4,
}

impl DeJson for TextBased {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Characters),
                    2 => Ok(Self::CharactersExcludingSpaces),
                    3 => Ok(Self::Words),
                    4 => Ok(Self::Lines),
                    _ => Err(s.err_range("1..4")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for TextBased {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Characters => 1.ser_json(d, s),
            Self::CharactersExcludingSpaces => 2.ser_json(d, s),
            Self::Words => 3.ser_json(d, s),
            Self::Lines => 4.ser_json(d, s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextShape {
    Square = 1,
    RampUp = 2,
    RampDown = 3,
    Triangle = 4,
    Round = 5,
    Smooth = 6,
}

impl DeJson for TextShape {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Square),
                    2 => Ok(Self::RampUp),
                    3 => Ok(Self::RampDown),
                    4 => Ok(Self::Triangle),
                    5 => Ok(Self::Round),
                    6 => Ok(Self::Smooth),
                    _ => Err(s.err_range("1..6")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for TextShape {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Square => 1.ser_json(d, s),
            Self::RampUp => 2.ser_json(d, s),
            Self::RampDown => 3.ser_json(d, s),
            Self::Triangle => 4.ser_json(d, s),
            Self::Round => 5.ser_json(d, s),
            Self::Smooth => 6.ser_json(d, s),
        }
    }
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextStyle {
    #[nserde(rename = "sw", default)]
    stroke_width: Option<Animated<f32>>,
    #[nserde(rename = "sc", default)]
    stroke_color: Option<Animated<Rgb>>,
    #[nserde(rename = "sh", default)]
    stroke_hue: Option<Animated<f32>>,
    #[nserde(rename = "ss", default)]
    stroke_saturation: Option<Animated<f32>>,
    #[nserde(rename = "sb", default)]
    stroke_brightness: Option<Animated<f32>>,
    #[nserde(rename = "so", default)]
    stroke_opacity: Option<Animated<f32>>,
    #[nserde(rename = "fc", default)]
    fill_color: Option<Animated<Rgb>>,
    #[nserde(rename = "fh", default)]
    fill_hue: Option<Animated<f32>>,
    #[nserde(rename = "fs", default)]
    fill_saturation: Option<Animated<f32>>,
    #[nserde(rename = "fb", default)]
    fill_brightness: Option<Animated<f32>>,
    #[nserde(rename = "t", default)]
    pub letter_spacing: Option<Animated<f32>>,
    #[nserde(rename = "bl", default)]
    blur: Option<Animated<f32>>,
    #[nserde(rename = "ls", default)]
    pub line_spacing: Option<Animated<f32>>,
    // #[nserde(flatten)]
    transform: Option<Transform>,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Rectangle {
    #[nserde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[nserde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[nserde(rename = "s")]
    pub size: Animated<Vector2D>,
    #[nserde(rename = "r")]
    pub radius: Animated<f32>,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Ellipse {
    #[nserde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[nserde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[nserde(rename = "s")]
    pub size: Animated<Vector2D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum ShapeDirection {
    #[default]
    Clockwise = 1,
    CounterClockwise = 2,
}

impl DeJson for ShapeDirection {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Clockwise),
                    2 => Ok(Self::CounterClockwise),
                    _ => Err(s.err_range("1..2")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for ShapeDirection {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Clockwise => 1.ser_json(d, s),
            Self::CounterClockwise => 2.ser_json(d, s),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Fill {
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "c")]
    pub color: Animated<Rgb>,
    #[nserde(rename = "r", default)]
    pub fill_rule: FillRule,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new_f32(r: f32, g: f32, b: f32) -> Rgb {
        Rgb {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }

    pub fn new_u8(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }
}

impl FromTo<Value> for Rgb {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        if v[0] > 1.0 && v[0] <= 255.0 {
            Rgb::new_u8(v[0] as u8, v[1] as u8, v[2] as u8)
        } else {
            Rgb::new_f32(v[0], v[1], v[2])
        }
    }

    fn to(self) -> Value {
        Value::List(vec![
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ])
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum FillRule {
    #[default]
    NonZero = 1,
    EvenOdd = 2,
}

impl DeJson for FillRule {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::NonZero),
                    2 => Ok(Self::EvenOdd),
                    _ => Err(s.err_range("1..2")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for FillRule {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::NonZero => 1.ser_json(d, s),
            Self::EvenOdd => 2.ser_json(d, s),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Stroke {
    #[nserde(rename = "lc")]
    pub line_cap: LineCap,
    #[nserde(rename = "lj")]
    pub line_join: LineJoin,
    #[nserde(rename = "ml", default)]
    miter_limit: f32,
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "w")]
    pub width: Animated<f32>,
    #[nserde(rename = "d", default)]
    dashes: Vec<StrokeDash>,
    #[nserde(rename = "c")]
    pub color: Animated<Rgb>,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum LineCap {
    #[default]
    Butt = 1,
    Round = 2,
    Square = 3,
}

impl DeJson for LineCap {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Butt),
                    2 => Ok(Self::Round),
                    3 => Ok(Self::Square),
                    _ => Err(s.err_range("0..6")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for LineCap {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Butt => 1.ser_json(d, s),
            Self::Round => 2.ser_json(d, s),
            Self::Square => 3.ser_json(d, s),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum LineJoin {
    #[default]
    Miter = 1,
    Round = 2,
    Bevel = 3,
}

impl DeJson for LineJoin {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Miter),
                    2 => Ok(Self::Round),
                    3 => Ok(Self::Bevel),
                    _ => Err(s.err_range("0..6")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for LineJoin {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Miter => 1.ser_json(d, s),
            Self::Round => 2.ser_json(d, s),
            Self::Bevel => 3.ser_json(d, s),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct StrokeDash {
    #[nserde(rename = "v")]
    length: Animated<f32>,
    #[nserde(rename = "n")]
    ty: StrokeDashType,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
pub enum StrokeDashType {
    #[nserde(rename = "d")]
    Dash,
    #[nserde(rename = "g")]
    Gap,
    #[nserde(rename = "o")]
    Offset,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct GradientFill {
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "r")]
    pub fill_rule: FillRule,
    #[nserde(flatten)]
    pub gradient: Gradient,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Gradient {
    #[nserde(rename = "s")]
    pub start: Animated<Vector2D>,
    #[nserde(rename = "e")]
    pub end: Animated<Vector2D>,
    #[nserde(rename = "t")]
    pub gradient_ty: GradientType,
    #[nserde(rename = "g", proxy = "ColorListHelper")]
    pub colors: ColorList,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum GradientType {
    #[default]
    Linear = 1,
    Radial = 2,
}

impl DeJson for GradientType {
    fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> Result<Self, DeJsonErr> {

        match s.tok {
            DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::Linear),
                    2 => Ok(Self::Radial),
                    _ => Err(s.err_range("1..2")),
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for GradientType {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Linear => 1.ser_json(d, s),
            Self::Radial => 2.ser_json(d, s),
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct ColorList {
    color_count: usize,
    pub colors: Animated<Vec<GradientColor>>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct GradientColor {
    pub offset: f32,
    pub color: Rgba,
}

impl FromTo<Value> for Vec<GradientColor> {
    fn from(v: Value) -> Self {
        todo!()
    }

    fn to(self) -> Value {
        todo!()
    }
}

#[derive(SerJson, DeJson)]
struct ColorListHelper {
    #[nserde(rename = "p")]
    color_count: usize,
    #[nserde(rename = "k")]
    colors: Animated<Vec<f32>>,
}

impl From<&ColorListHelper> for ColorList {
    fn from(helper: &ColorListHelper) -> Self {
        let color_count = helper.color_count;
        ColorList {
            color_count,
            colors: Animated {
                animated: helper.colors.animated,
                keyframes: helper
                    .colors
                    .keyframes
                    .clone()
                    .into_iter()
                    .map(|keyframe| {
                        let start = f32_to_gradient_colors(&keyframe.start_value, color_count);
                        let end = f32_to_gradient_colors(&keyframe.end_value, color_count);
                        keyframe.alter_value(start, end)
                    })
                    .collect(),
            },
        }
    }
}

fn f32_to_gradient_colors(data: &Vec<f32>, color_count: usize) -> Vec<GradientColor> {
    if data.len() == color_count * 4 {
        // Rgb color
        data.chunks(4)
            .map(|chunk| GradientColor {
                offset: chunk[0],
                color: Rgba::new_f32(chunk[1], chunk[2], chunk[3], 1.0),
            })
            .collect()
    } else if data.len() == color_count * 4 + color_count * 2 {
        // Rgba color
        (&data[0..(color_count * 4)])
            .chunks(4)
            .zip((&data[(color_count * 4)..]).chunks(2))
            .map(|(chunk, opacity)| GradientColor {
                offset: chunk[0],
                color: Rgba::new_f32(chunk[1], chunk[2], chunk[3], opacity[1]),
            })
            .collect()
    } else {
        unimplemented!()
    }
}

impl From<&ColorList> for ColorListHelper {
    fn from(list: &ColorList) -> Self {
        ColorListHelper {
            color_count: list.color_count,
            colors: Animated {
                animated: list.colors.animated,
                keyframes: list
                    .colors
                    .keyframes
                    .clone()
                    .into_iter()
                    .map(|keyframe| {
                        let start = gradient_colors_to_f32(&keyframe.start_value);
                        let end = gradient_colors_to_f32(&keyframe.end_value);
                        keyframe.alter_value(start, end)
                    })
                    .collect(),
            },
        }
    }
}

fn gradient_colors_to_f32(data: &Vec<GradientColor>) -> Vec<f32> {
    let mut start = data
        .iter()
        .flat_map(|color| {
            vec![
                color.offset,
                color.color.r as f32 / 255.0,
                color.color.g as f32 / 255.0,
                color.color.b as f32 / 255.0,
            ]
        })
        .collect::<Vec<_>>();
    let start_has_opacity = data.iter().any(|color| color.color.a < 255);
    if start_has_opacity {
        start.extend(
            data.iter()
                .flat_map(|color| vec![color.offset, color.color.a as f32 / 255.0]),
        );
    }
    start
}

// #[derive(SerJson, DeJson, Debug, Clone)]
// pub struct TextAnimationData {
//     #[nserde(rename = "a")]
//     pub ranges: Vec<TextRange>,
//     #[nserde(rename = "d")]
//     pub document: TextData,
//     #[nserde(rename = "m")]
//     options: TextAlignmentOptions,
//     #[nserde(rename = "p")]
//     follow_path: TextFollowPath,
// }

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Media {
    #[nserde(rename = "u", default)]
    pub pwd: String,
    #[nserde(rename = "p")]
    pub filename: String,
    #[nserde(rename = "e", proxy = "BoolFromInt", default)]
    pub embedded: bool,
    id: String,
    #[nserde(rename = "nm", default)]
    name: Option<String>,
    #[nserde(rename = "mn", default, skip)]
    match_name: Option<String>,
    #[nserde(rename = "w", default)]
    pub width: Option<u32>,
    #[nserde(rename = "h", default)]
    pub height: Option<u32>,
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
#[nserde(serialize_none_as_null)]
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

impl Default for Transform {
    fn default() -> Self {
        println!("removed 3");
        Self {
            anchor: Default::default(),
            position: Default::default(),
            scale: default_vec2_100(),
            rotation: Default::default(),
            opacity: default_number_100(),
            skew: Default::default(),
            skew_axis: Default::default(),
            auto_orient: false,
        }
    }
}

#[derive(Debug)]
// deserialize_with = "keyframes_from_array"
pub enum KeyFramesFromArray {
    Plain(Value),
    List(Value),
    LegacyKeyFrames(Vec<LegacyKeyFrame<Value>>),
}

fn default_none<T>() -> Option<T> {
    None
}

#[derive(DeJson, SerJson, Default, Debug, Clone, PartialEq)]
pub struct LegacyKeyFrame<T> {
    #[nserde(rename = "s")]
    start_value: T,
    #[nserde(rename = "e", default_with = "default_none")]
    end_value: Option<T>,
    #[nserde(rename = "t", default)]
    start_frame: f32,
    #[nserde(skip)]
    end_frame: f32,
    #[nserde(rename = "o", default)]
    easing_out: Option<Easing>,
    #[nserde(rename = "i", default)]
    easing_in: Option<Easing>,
    #[nserde(rename = "h", default, proxy = "BoolFromInt")]
    hold: bool,
}

impl DeJson for KeyFramesFromArray {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        match s.tok {
            DeJsonTok::U64(v) => {
                s.next_tok(i)?;
                Ok(Self::Plain(Value::Primitive(v as f32)))
            }
            DeJsonTok::I64(v) => {
                s.next_tok(i)?;
                Ok(Self::Plain(Value::Primitive(v as f32)))
            }
            DeJsonTok::BlockOpen => {
                s.next_tok(i)?;
                match s.tok {
                    DeJsonTok::F64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::List(Value::List(res)))
                    }
                    DeJsonTok::U64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::List(Value::List(res)))
                    }
                    DeJsonTok::I64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::List(Value::List(res)))
                    }
                    DeJsonTok::CurlyOpen => {
                        let mut res = vec![];
                        s.next_tok(i)?;
                        {
                            let mut _start_value = None;
                            let mut _end_value = None;
                            let mut _start_frame = None;
                            let mut _easing_out = None;
                            let mut _easing_in = None;
                            let mut _hold = None;
                            while let Some(_) = s.next_str() {
                                match AsRef::<str>::as_ref(&s.strbuf) {
                                    "s" => {
                                        s.next_colon(i)?;
                                        _start_value = Some(DeJson::de_json(s, i)?);
                                    }
                                    "e" => {
                                        s.next_colon(i)?;
                                        _end_value = Some(DeJson::de_json(s, i)?);
                                    }
                                    "t" => {
                                        s.next_colon(i)?;
                                        _start_frame = Some(DeJson::de_json(s, i)?);
                                    }
                                    "o" => {
                                        s.next_colon(i)?;
                                        _easing_out = Some(DeJson::de_json(s, i)?);
                                    }
                                    "i" => {
                                        s.next_colon(i)?;
                                        _easing_in = Some(DeJson::de_json(s, i)?);
                                    }
                                    "h" => {
                                        s.next_colon(i)?;
                                        _hold = Some(DeJson::de_json(s, i)?);
                                    }
                                    _ => {
                                        s.next_colon(i)?;
                                        s.whole_field(i)?;
                                    }
                                }
                                s.eat_comma_curly(i)?
                            }
                            s.curly_close(i)?;
                            res.push(LegacyKeyFrame {
                                start_value: {
                                    if let Some(t) = _start_value {
                                        t
                                    } else {
                                        return Err(s.err_nf("start_value"));
                                    }
                                },
                                end_value: {
                                    if let Some(t) = _end_value {
                                        t
                                    } else {
                                        default_none()
                                    }
                                },
                                start_frame: {
                                    if let Some(t) = _start_frame {
                                        t
                                    } else {
                                        Default::default()
                                    }
                                },
                                end_frame: Default::default(),
                                easing_out: {
                                    if let Some(t) = _easing_out {
                                        t
                                    } else {
                                        None
                                    }
                                },
                                easing_in: {
                                    if let Some(t) = _easing_in {
                                        t
                                    } else {
                                        None
                                    }
                                },
                                hold: {
                                    if let Some(t) = _hold {
                                        From::<&BoolFromInt>::from(&t)
                                    } else {
                                        Default::default()
                                    }
                                },
                            });
                        }
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        Ok(Self::LegacyKeyFrames(res))
                    }
                    _ => Err(s.err_token("U64 or {")),
                }
            }
            DeJsonTok::CurlyOpen => {
                let res = DeJson::de_json(s, i)?;
                Ok(Self::Plain(Value::Bezier(res)))
            }
            _ => Err(s.err_token("U64 or [")),
        }
    }
}

impl SerJson for KeyFramesFromArray {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        match self {
            Self::Plain(f0) => {
                f0.ser_json(d, s);
            }
            Self::List(f0) => {
                f0.ser_json(d, s);
            }
            Self::LegacyKeyFrames(_) => todo!(),
        }
    }
}

impl<T: Default + Debug> From<&Vec<KeyFrame<T>>> for KeyFramesFromArray {
    fn from(value: &Vec<KeyFrame<T>>) -> KeyFramesFromArray {
        KeyFramesFromArray::List(Value::List(value.iter().map(|v| v.start_frame).collect()))
    }
}

impl<T: Clone + Default + FromTo<Value> + Debug> From<&KeyFramesFromArray> for Vec<KeyFrame<T>> {
    fn from(val: &KeyFramesFromArray) -> Vec<KeyFrame<T>> {
        match val {
            KeyFramesFromArray::Plain(v) => {
                vec![KeyFrame {
                    start_value: T::from(v.clone()),
                    end_value: T::from(v.clone()),
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
            KeyFramesFromArray::LegacyKeyFrames(v) => {
                let mut result: Vec<LegacyKeyFrame<Value>> = vec![];
                // Sometimes keyframes especially from TextData do not have an ending frame, so
                // we double check here to avoid removing them.
                for k in v {
                    let mut res_k = k.clone();
                    if let Some(prev) = result.last_mut() {
                        prev.end_frame = k.start_frame;
                    }
                    if k.hold {
                        res_k = k.clone();
                        res_k.end_value = Some(k.start_value.clone());
                    }
                    result.push(res_k)
                }
                if result.len() > 1 {
                    for i in 0..(result.len() - 1) {
                        if result[i].end_value.is_none() {
                            result[i].end_value = Some(result[i + 1].start_value.clone());
                        }
                    }
                }
                let res = result
                    .into_iter()
                    .map(|keyframe| {
                        let end_value = T::from(
                            keyframe
                                .end_value
                                .unwrap_or_else(|| keyframe.start_value.clone()),
                        );
                        KeyFrame {
                            end_value,
                            start_value: T::from(keyframe.start_value),
                            start_frame: keyframe.start_frame,
                            end_frame: keyframe.end_frame.max(keyframe.start_frame),
                            easing_in: keyframe.easing_in,
                            easing_out: keyframe.easing_out,
                        }
                    })
                    .collect();
                res
            }
        }
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Animated<T: Debug + Default + Clone + FromTo<Value>> {
    #[nserde(proxy = "BoolFromInt", rename = "a", default)]
    pub animated: bool,
    #[nserde(proxy = "KeyFramesFromArray", rename = "k")]
    pub keyframes: Vec<KeyFrame<T>>,
}

#[derive(SerJson, DeJson, Default, Debug, Clone, PartialEq)]
pub struct KeyFrame<T: Default + Debug> {
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

impl<T: Clone + Default + Debug> KeyFrame<T> {
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

    pub fn alter_value<U: Default + Debug>(&self, start: U, end: U) -> KeyFrame<U> {
        KeyFrame {
            start_value: start,
            end_value: end,
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            easing_out: self.easing_out.clone(),
            easing_in: self.easing_in.clone(),
        }
    }
}

#[derive(PartialEq, Debug, SerJson)]
// deserialize_with = "array_from_array_or_number"
enum ArrayFromArrayOfNumber {
    List(Vec<f32>),
    Primitive(f32),
}

impl DeJson for ArrayFromArrayOfNumber {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        match s.tok {
            DeJsonTok::U64(_) => {
                let r = Self::Primitive(s.as_f64()? as f32);
                s.next_tok(i)?;
                Ok(r)
            }
            DeJsonTok::F64(_) => {
                let r = Self::Primitive(s.as_f64()? as f32);
                s.next_tok(i)?;
                Ok(r)
            }
            DeJsonTok::BlockOpen => {
                let r: Vec<f32> = DeJson::de_json(s, i)?;
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
