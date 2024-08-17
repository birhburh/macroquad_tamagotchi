// Based on https://github.com/zimond/lottie-rs/

use {
    nanoserde::{DeJson, DeJsonErr, DeJsonState, SerJson, SerJsonState},
    std::str::Chars,
};


#[derive(Debug, Clone, Default)]
pub struct Vector2D(euclid::default::Vector2D<f32>);

impl Vector2D {
    fn new(x: f32, y: f32) -> Vector2D {
        Vector2D(euclid::default::Vector2D::new(x, y))
    }
}

impl SerJson for Vector2D {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        self.0.x.ser_json(d, s);
        s.out.push(',');
        self.0.y.ser_json(d, s);
        s.out.push(']');
    }
}
impl DeJson for Vector2D {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        s.block_open(i)?;
        let x = DeJson::de_json(s, i)?;
        s.eat_comma_block(i)?;
        let y = DeJson::de_json(s, i)?;
        let r = Vector2D::new(x, y);
        s.block_close(i)?;
        Ok(r)
    }
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Animated<T: Default> {
    #[nserde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a",
        default
    )]
    pub animated: bool,
    #[nserde(
        deserialize_with = "keyframes_from_array",
        serialize_with = "array_from_keyframes",
        bound = "T: FromTo<helpers::Value>",
        rename = "k"
    )]
    pub keyframes: Vec<KeyFrame<T>>,
}

#[derive(SerJson, DeJson, Default, Debug, Clone)]
pub struct KeyFrame<T: Default> {
    #[nserde(rename = "s")]
    pub start_value: T,
    #[nserde(skip)]
    pub end_value: Option<T>,
    #[nserde(rename = "t", default)]
    pub start_frame: f32,
    // TODO: could end_frame & next start_frame create a gap?
    #[nserde(skip)]
    pub end_frame: Option<f32>,
    #[nserde(rename = "o", default)]
    pub easing_out: Option<Easing>,
    #[nserde(rename = "i", default)]
    pub easing_in: Option<Easing>,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Easing {
    #[nserde(deserialize_with = "array_from_array_or_number")]
    pub x: Vec<f32>,
    #[nserde(deserialize_with = "array_from_array_or_number")]
    pub y: Vec<f32>,
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
    #[nserde(default)]
    pub assets: Vec<Asset>,
    #[nserde(default)]
    pub fonts: FontList,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Layer {
    #[nserde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "ddd",
        default
    )]
    is_3d: bool,
    #[nserde(rename = "hd", default)]
    pub hidden: bool,
    #[nserde(rename = "ind", default)]
    pub index: Option<u32>,
    #[nserde(rename = "parent", default)]
    pub parent_index: Option<u32>,
    #[nserde(skip)]
    pub id: Option<u32>,
    #[nserde(
        rename = "ao",
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
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
    #[nserde(flatten)]
    pub content: LayerContent,
    #[nserde(rename = "tt", default)]
    pub matte_mode: Option<MatteMode>,
    #[nserde(rename = "bm", default)]
    pub blend_mode: Option<BlendMode>,
    #[nserde(default, rename = "hasMask")]
    pub has_mask: bool,
    #[nserde(default, rename = "masksProperties")]
    pub masks_properties: Vec<Mask>,
}

#[derive(DeJson, SerJson, Debug, Clone, Copy, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(DeJson, SerJson, Debug, Clone, Copy, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(DeJson, SerJson, Debug, Clone)]
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
    Text(TextAnimationData),
    Media(Media),
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct MediaRef {
    #[nserde(rename = "refId")]
    pub ref_id: String,
}

#[derive(SerJson, DeJson, Debug, Clone)]
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

impl<T: Clone + Default> KeyFrame<T> {
    pub fn from_value(value: T) -> Self {
        KeyFrame {
            start_value: value.clone(),
            end_value: Some(value),
            start_frame: 0.0,
            end_frame: Some(0.0),
            easing_out: None,
            easing_in: None,
        }
    }
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
    pub auto_orient: Option<bool>,
    #[nserde(rename = "o", default_with = "default_number_100")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "sk", default)]
    pub skew: Option<Animated<f32>>,
    #[nserde(rename = "sa", default)]
    pub skew_axis: Option<Animated<f32>>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct RepeaterTransform {
    #[nserde(rename = "a", default)]
    anchor: Animated<Vector2D>,
    #[nserde(rename = "p")]
    position: Animated<Vector2D>,
    #[nserde(rename = "s")]
    scale: Animated<Vector2D>,
    #[nserde(rename = "r")]
    rotation: Animated<f32>,
    #[nserde(rename = "so")]
    start_opacity: Animated<f32>,
    #[nserde(rename = "eo")]
    end_opacity: Animated<f32>,
    #[nserde(rename = "sk", default)]
    skew: Option<Animated<Vector2D>>,
    #[nserde(rename = "sa", default)]
    skew_axis: Option<Animated<Vector2D>>,
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

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct ShapeLayer {
    #[nserde(rename = "nm", default)]
    pub name: Option<String>,
    #[nserde(rename = "hd", default)]
    pub hidden: bool,
    #[nserde(flatten)]
    pub shape: Shape,
}

#[derive(SerJson, Debug, Clone, Default)]
pub struct TextRangeInfo {
    #[nserde(skip)]
    pub value: Vec<Vec<char>>,
    pub index: (usize, usize), // line, char
    pub ranges: Vec<TextRange>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
#[nserde(tag = "ty")]
pub enum Shape {
    #[nserde(rename = "rc")]
    Rectangle(Rectangle),
    #[nserde(rename = "el")]
    Ellipse(Ellipse),
    #[nserde(rename = "sr")]
    PolyStar(PolyStar),
    #[nserde(rename = "sh")]
    Path {
        #[nserde(rename = "ks")]
        d1: Animated<Vec<Bezier>>, // renamed it but probably should add PR to nanoserde to be able to use field named `d`
        #[nserde(skip)]
        text_range: Option<TextRangeInfo>,
    },
    #[nserde(rename = "fl")]
    Fill(Fill),
    #[nserde(rename = "st")]
    Stroke(Stroke),
    #[nserde(rename = "gf")]
    GradientFill(GradientFill),
    #[nserde(rename = "gs")]
    GradientStroke(GradientStroke),
    #[nserde(rename = "gr")]
    Group {
        // TODO: add np property
        #[nserde(rename = "it")]
        shapes: Vec<ShapeLayer>,
    },
    #[nserde(rename = "tr")]
    Transform(Transform),
    #[nserde(rename = "rp")]
    Repeater {
        #[nserde(rename = "c")]
        copies: Animated<f32>,
        #[nserde(rename = "o")]
        offset: Animated<f32>,
        #[nserde(rename = "m")]
        composite: Composite,
        #[nserde(rename = "tr")]
        transform: RepeaterTransform,
    },
    #[nserde(rename = "tm")]
    Trim(Trim),
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
    #[nserde(rename = "mm")]
    Merge {
        #[nserde(rename = "mm")]
        mode: MergeMode,
    },
    #[nserde(rename = "op")]
    OffsetPath {
        #[nserde(rename = "a")]
        amount: Animated<f32>,
        #[nserde(rename = "lj")]
        line_join: LineJoin,
        #[nserde(rename = "ml")]
        miter_limit: f32,
    },
    #[nserde(rename = "zz")]
    ZigZag {
        #[nserde(rename = "r")]
        radius: Animated<f32>,
        #[nserde(rename = "s")]
        distance: Animated<f32>,
        #[nserde(rename = "pt")]
        ridges: Animated<f32>,
    },
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PolyStarType {
    Star = 1,
    Polygon = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum FillRule {
    #[default]
    NonZero = 1,
    EvenOdd = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineCap {
    Butt = 1,
    Round = 2,
    Square = 3,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineJoin {
    Miter = 1,
    Round = 2,
    Bevel = 3,
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

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum GradientType {
    Linear = 1,
    Radial = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Composite {
    Above = 1,
    Below = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TrimMultipleShape {
    Individually = 1,
    Simultaneously = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum ShapeDirection {
    #[default]
    Clockwise = 1,
    CounterClockwise = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy)]
#[repr(u8)]
pub enum MergeMode {
    #[nserde(other)]
    Unsupported = 1,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum FontPathOrigin {
    #[default]
    Local = 0,
    CssUrl = 1,
    ScriptUrl = 2,
    FontUrl = 3,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum TextJustify {
    #[default]
    Left = 0,
    Right = 1,
    Center = 2,
    LastLineLeft = 3,
    LastLineRight = 4,
    LastLineCenter = 5,
    LastLineFull = 6,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum TextCaps {
    #[default]
    Regular = 0,
    AllCaps = 1,
    SmallCaps = 2,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum TextBased {
    #[default]
    Characters = 1,
    CharactersExcludingSpaces = 2,
    Words = 3,
    Lines = 4,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum TextShape {
    #[default]
    Square = 1,
    RampUp = 2,
    RampDown = 3,
    Triangle = 4,
    Round = 5,
    Smooth = 6,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MatteMode {
    Normal = 0,
    Alpha = 1,
    InvertedAlpha = 2,
    Luma = 3,
    InvertedLuma = 4,
}

#[derive(SerJson, DeJson, Debug, Clone, Copy, PartialEq)]
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

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Trim {
    #[nserde(rename = "s")]
    pub start: Animated<f32>,
    #[nserde(rename = "e")]
    pub end: Animated<f32>,
    #[nserde(rename = "o")]
    pub offset: Animated<f32>,
    #[nserde(rename = "m")]
    pub multiple_shape: TrimMultipleShape,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Fill {
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "c")]
    pub color: Animated<Rgb>,
    #[nserde(rename = "r", default)]
    pub fill_rule: FillRule,
}

#[derive(SerJson, DeJson, Debug, Clone)]
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

#[derive(SerJson, DeJson, Debug, Clone)]
#[nserde(from = "ColorListHelper", into = "ColorListHelper")]
pub struct ColorList {
    color_count: usize,
    pub colors: Animated<Vec<GradientColor>>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct GradientColor {
    pub offset: f32,
    pub color: Rgba,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Gradient {
    #[nserde(rename = "s")]
    pub start: Animated<Vector2D>,
    #[nserde(rename = "e")]
    pub end: Animated<Vector2D>,
    #[nserde(rename = "t")]
    pub gradient_ty: GradientType,
    #[nserde(rename = "g")]
    pub colors: ColorList,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct GradientFill {
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "r")]
    pub fill_rule: FillRule,
    #[nserde(flatten)]
    pub gradient: Gradient,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct GradientStroke {
    #[nserde(rename = "lc")]
    pub line_cap: LineCap,
    #[nserde(rename = "lj")]
    pub line_join: LineJoin,
    #[nserde(rename = "ml")]
    miter_limit: f32,
    #[nserde(rename = "o")]
    pub opacity: Animated<f32>,
    #[nserde(rename = "w")]
    pub width: Animated<f32>,
    #[nserde(rename = "d", default)]
    dashes: Vec<StrokeDash>,
    #[nserde(flatten)]
    pub gradient: Gradient,
}

#[derive(SerJson, DeJson, Debug, Clone)]
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

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Ellipse {
    #[nserde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[nserde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[nserde(rename = "s")]
    pub size: Animated<Vector2D>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct PolyStar {
    #[nserde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[nserde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[nserde(rename = "or")]
    pub outer_radius: Animated<f32>,
    #[nserde(rename = "os")]
    pub outer_roundness: Animated<f32>,
    #[nserde(rename = "ir", default)]
    pub inner_radius: Option<Animated<f32>>,
    #[nserde(rename = "is")]
    pub inner_roundness: Option<Animated<f32>>,
    #[nserde(rename = "r")]
    pub rotation: Animated<f32>,
    #[nserde(rename = "pt")]
    pub points: Animated<f32>,
    #[nserde(rename = "sy")]
    pub star_type: PolyStarType,
}

#[derive(SerJson, DeJson, Debug, Clone)]
#[nserde(untagged)]
pub enum Asset {
    Media(Media),
    Sound,
    Precomposition(Precomposition),
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Media {
    #[nserde(rename = "u", default)]
    pub pwd: String,
    #[nserde(rename = "p")]
    pub filename: String,
    #[nserde(
        rename = "e",
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub embedded: bool,
    id: String,
    #[nserde(rename = "nm", default)]
    name: Option<String>,
    #[nserde(rename = "w", default)]
    pub width: Option<u32>,
    #[nserde(rename = "h", default)]
    pub height: Option<u32>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Precomposition {
    pub id: String,
    pub layers: Vec<Layer>,
    #[nserde(rename = "nm")]
    name: Option<String>,
    #[nserde(rename = "fr")]
    pub frame_rate: Option<f32>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct ShapeGroup {
    pub shapes: Vec<ShapeLayer>,
}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
pub struct Bezier {
    #[nserde(rename = "c", default)]
    pub closed: bool,
    #[nserde(
        rename = "v",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub verticies: Vec<Vector2D>,
    #[nserde(
        rename = "i",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub in_tangent: Vec<Vector2D>,
    #[nserde(
        rename = "o",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub out_tangent: Vec<Vector2D>,
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextAnimationData {
    #[nserde(rename = "a")]
    pub ranges: Vec<TextRange>,
    #[nserde(rename = "d")]
    pub document: TextData,
    #[nserde(rename = "m")]
    options: TextAlignmentOptions,
    #[nserde(rename = "p")]
    follow_path: TextFollowPath,
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
    #[nserde(flatten)]
    transform: Option<Transform>,
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
    #[nserde(rename = "t", deserialize_with = "bool_from_int")]
    expressible: bool,
    #[nserde(rename = "xe")]
    max_ease: Animated<f32>,
    #[nserde(rename = "ne")]
    min_ease: Animated<f32>,
    #[nserde(rename = "a")]
    max_amount: Animated<f32>,
    #[nserde(rename = "b")]
    based_on: TextBased,
    #[nserde(rename = "rn", deserialize_with = "bool_from_int")]
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

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct TextData {
    #[nserde(rename = "x", default)]
    expression: Option<String>,
    #[nserde(
        deserialize_with = "keyframes_from_array",
        serialize_with = "array_from_keyframes",
        rename = "k"
    )]
    pub keyframes: Vec<KeyFrame<TextDocument>>,
}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct TextAlignmentOptions {}

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct TextFollowPath {}

#[derive(SerJson, DeJson, Debug, Clone, Default)]
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

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct Mask {
    #[nserde(rename = "nm", default)]
    pub name: String,
    #[nserde(rename = "mn", default)]
    match_name: String,
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
