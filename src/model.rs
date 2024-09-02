// Based on https://github.com/zimond/lottie-rs/

// nanoserde TODO:
// - implement ((serialize|deserialize)_)?with
// - implement untagged for enum
// - implement flatten for attribute
// - imlemnet tag attribute for enum

use {
    nanoserde::{DeJson, DeJsonTok, SerJson}, std::{fmt::Debug, vec}
};

pub trait FromTo<T> {
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

    fn to(self) -> Vec<f32> {
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
    pub transform: Option<Transform>,
    pub content: LayerContent,
    // #[nserde(rename = "tt", default)]
    // pub matte_mode: Option<MatteMode>,
    // #[nserde(rename = "bm", default)]
    // pub blend_mode: Option<BlendMode>,
    // #[nserde(default, rename = "hasMask")]
    // pub has_mask: bool,
    // #[nserde(default, rename = "masksProperties")]
    // pub masks_properties: Vec<Mask>,
}

impl DeJson for Layer {
    #[allow(clippy::ignored_unit_patterns)]
    fn de_json(
        s: &mut nanoserde::DeJsonState,
        i: &mut core::str::Chars,
    ) -> ::core::result::Result<Self, nanoserde::DeJsonErr> {
        println!("de_json for Layer\n");
        ::core::result::Result::Ok({
            let mut _is_3d = None;
            let mut _hidden = None;
            let mut _index = None;
            let mut _parent_index = None;
            let mut _auto_orient = None;
            let mut _start_frame = None;
            let mut _end_frame = None;
            let mut _start_time = None;
            let mut _name = None;
            let mut _transform = None;
            let mut _content = None;
            s.curly_open(i)?;
            while let Some(_) = s.next_str() {
                match AsRef::<str>::as_ref(&s.strbuf) {
                    "ddd" => {
                        s.next_colon(i)?;
                        _is_3d = Some(DeJson::de_json(s, i)?);
                    }
                    "hd" => {
                        s.next_colon(i)?;
                        _hidden = Some(DeJson::de_json(s, i)?);
                    }
                    "ind" => {
                        s.next_colon(i)?;
                        _index = Some(DeJson::de_json(s, i)?);
                    }
                    "parent" => {
                        s.next_colon(i)?;
                        _parent_index = Some(DeJson::de_json(s, i)?);
                    }
                    "ao" => {
                        s.next_colon(i)?;
                        _auto_orient = Some(DeJson::de_json(s, i)?);
                    }
                    "ip" => {
                        s.next_colon(i)?;
                        _start_frame = Some(DeJson::de_json(s, i)?);
                    }
                    "op" => {
                        s.next_colon(i)?;
                        _end_frame = Some(DeJson::de_json(s, i)?);
                    }
                    "st" => {
                        s.next_colon(i)?;
                        _start_time = Some(DeJson::de_json(s, i)?);
                    }
                    "ks" => {
                        s.next_colon(i)?;
                        _transform = Some(DeJson::de_json(s, i)?);
                    }
                    "refId" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::PreCompositionRef(_))
                                | Some(LayerContent::MediaRef(_))
                        ));
                        println!("sub de_json for PreCompositionRef");
                        println!("sub de_json for MediaRef");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "w" | "h" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::PreCompositionRef(_))
                                | Some(LayerContent::MediaRef(_))
                        ));
                        println!("sub de_json for PreCompositionRef");
                        println!("sub de_json for Media");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "tm" => {
                        s.next_colon(i)?;
                        println!("sub de_json for PreCompositionRef");
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
                    "shapes" => {
                        s.next_colon(i)?;
                        assert!(matches!(_content, None));
                        println!("sub de_json for Shape(ShapeGroup)");
                        let shapes = DeJson::de_json(s, i)?;
                        match _content.as_mut() {
                            None => {
                                _content = Some(LayerContent::Shape(ShapeGroup { shapes }));
                            }
                            _ => unreachable!(),
                        }
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "a" | "d" | "m" => {
                        s.next_colon(i)?;
                        // assert!(matches!(
                        //     _content,
                        //     None | Some(LayerContent::Text(_))
                        // ));
                        println!("sub de_json for Text(TextAnimationData)");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "p" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Text(TextAnimationData)");
                        println!("sub de_json for Media");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "u" | "e" => {
                        s.next_colon(i)?;
                        assert!(matches!(_content, None | Some(LayerContent::Media(_))));
                        println!("sub de_json for Media");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    "nm" => {
                        s.next_colon(i)?;
                        assert!(matches!(_content, None | Some(LayerContent::Media(_))));
                        println!("sub de_json for Media");
                        _name = Some(DeJson::de_json(s, i)?);
                    }
                    "sc" | "sh" | "sw" => {
                        s.next_colon(i)?;
                        assert!(matches!(
                            _content,
                            None | Some(LayerContent::SolidColor { .. })
                        ));
                        println!("sub de_json for SolidColor");
                        // _content = Some(DeJson::de_json(s, i)?);
                    }
                    _ => {
                        s.next_colon(i)?;
                        s.whole_field(i)?;
                    }
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
    fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {
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
            _ => unreachable!(),
        }
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

#[derive(SerJson, DeJson, Debug, Clone)]
pub struct MediaRef {
    #[nserde(rename = "refId")]
    pub ref_id: String,
}

#[derive(SerJson, DeJson, Debug, Clone)]
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
    fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {
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
            Shape::Rectangle(_) => String::ser_json(&"rc".into(), d + 1, s),
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
            _ => unreachable!(),
        }
        s.st_post(d);
    }
}

impl DeJson for ShapeLayer {
    #[allow(clippy::ignored_unit_patterns)]
    fn de_json(
        s: &mut nanoserde::DeJsonState,
        i: &mut core::str::Chars,
    ) -> ::core::result::Result<Self, nanoserde::DeJsonErr> {
        println!("de_json for ShapeLayer");
        ::core::result::Result::Ok({
            let mut _name = None;
            let mut _hidden = None;
            let mut _shape = None;
            s.curly_open(i)?;
            while let Some(_) = s.next_str() {
                dbg!(&s.strbuf);
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
                        println!("sub de_json for Shape");
                        s.next_colon(i)?;
                        if let Some(_) = s.next_str() {
                            dbg!(&s.strbuf);
                            match AsRef::<str>::as_ref(&s.strbuf) {
                                "rc" => {
                                    println!("sub de_json for Rectangle");
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Rectangle(Default::default()));
                                        }
                                        Some(Shape::Rectangle(_)) => (),
                                        _ => unreachable!(),
                                    }
                                }
                                "el" => {
                                    println!("sub de_json for Ellipse");
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Ellipse(Default::default()));
                                        }
                                        Some(Shape::Ellipse(_)) => (),
                                        _ => unreachable!(),
                                    }
                                }
                                "gr" => {
                                    println!("sub de_json for Group");
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Group {
                                                shapes: Default::default(),
                                            });
                                        }
                                        Some(Shape::Group {..}) => (),
                                        _ => unreachable!(),
                                    }
                                }
                                "fl" => {
                                    println!("sub de_json for Fill");
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Fill(Default::default()));
                                        }
                                        Some(Shape::Fill {..}) => (),
                                        _ => unreachable!(),
                                    }
                                }
                                "tr" => {
                                    println!("sub de_json for Transform");
                                    match _shape.as_ref() {
                                        None => {
                                            _shape = Some(Shape::Transform(Default::default()));
                                        }
                                        Some(Shape::Transform {..}) => (),
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                            s.next_tok(i)?;
                        }
                    }
                    "it" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Group");
                        let shapes = DeJson::de_json(s, i)?;
                        match _shape.as_mut() {
                            None => {
                                _shape = Some(Shape::Group {
                                    shapes,
                                });
                            }
                            Some(Shape::Group {shapes: shapes_field}) => {
                                *shapes_field = shapes;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "a" => {
                        s.next_colon(i)?;
                        println!("sub de_json for PuckerBloat");
                        println!("sub de_json for Twist");
                        println!("sub de_json for OffsetPath");
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.anchor = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "c" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Fill");
                        let color = DeJson::de_json(s, i)?;
                        match _shape.as_mut() {
                            None => {
                                _shape = Some(Shape::Fill(Fill { color, ..Default::default()}));
                            }
                            Some(Shape::Fill(fill)) => {
                                fill.color = color;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "d" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Rectangle");
                        println!("sub de_json for Ellipse");
                        let direction = DeJson::de_json(s, i)?;
                        match _shape.as_mut() {
                            Some(Shape::Rectangle(rectangle)) => {
                                rectangle.direction = direction;
                            },
                            Some(Shape::Ellipse(ellipse)) => {
                                ellipse.direction = direction;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "o" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Repeater");
                        println!("sub de_json for Fill");
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Fill(fill)) => {
                                fill.opacity = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Transform(transform)) => {
                                transform.opacity = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "p" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Transform");
                        println!("sub de_json for Rectangle");
                        println!("sub de_json for Ellipse");
                        match _shape.as_mut() {
                            Some(Shape::Rectangle(rectangle)) => {
                                rectangle.position = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Ellipse(ellipse)) => {
                                ellipse.position = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Transform(transform)) => {
                                transform.position = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "r" => {
                        s.next_colon(i)?;
                        println!("sub de_json for RoundedCorners");
                        println!("sub de_json for ZigZag");
                        println!("sub de_json for Rectangle");
                        println!("sub de_json for Fill");
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Fill(fill)) => {
                                fill.fill_rule = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Transform(transform)) => {
                                transform.rotation = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Rectangle(rectangle)) => {
                                rectangle.radius = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::RoundedCorners{radius}) => {
                                *radius = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "s" => {
                        s.next_colon(i)?;
                        println!("sub de_json for ZigZag");
                        println!("sub de_json for Rectangle");
                        println!("sub de_json for Ellipse");
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Rectangle(rectangle)) => {
                                rectangle.size = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Ellipse(ellipse)) => {
                                ellipse.size = DeJson::de_json(s, i)?;
                            },
                            Some(Shape::Transform(transform)) => {
                                transform.scale = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "sk" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.skew = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
                        }
                    }
                    "sa" => {
                        s.next_colon(i)?;
                        println!("sub de_json for Transform");
                        match _shape.as_mut() {
                            Some(Shape::Transform(transform)) => {
                                transform.skew_axis = DeJson::de_json(s, i)?;
                            },
                            _ => unreachable!(),
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
    // #[nserde(rename = "sh")]
    // Path {
    //     #[nserde(rename = "ks")]
    //     d: Animated<Vec<Bezier>>,
    //     #[nserde(skip)]
    //     text_range: Option<TextRangeInfo>,
    // },
    #[nserde(rename = "fl")]
    Fill(Fill),
    // #[nserde(rename = "st")]
    // Stroke(Stroke),
    // #[nserde(rename = "gf")]
    // GradientFill(GradientFill),
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

#[derive(DeJson, Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum ShapeDirection {
    #[default]
    Clockwise = 1,
    CounterClockwise = 2,
}

impl SerJson for ShapeDirection {
    fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {
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

impl FromTo<Vec<f32>> for Rgb {
    fn from(v: Vec<f32>) -> Self {
        if v[0] > 1.0 && v[0] <= 255.0 {
            Rgb::new_u8(v[0] as u8, v[1] as u8, v[2] as u8)
        } else {
            Rgb::new_f32(v[0], v[1], v[2])
        }
    }

    fn to(self) -> Vec<f32> {
        vec![
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
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
    fn de_json(s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> Result<Self, nanoserde::DeJsonErr> {
        println!("de_json for FillRule");

        match s.tok {
            nanoserde::DeJsonTok::U64(_) => {
                let r = s.as_f64()? as u8;
                s.next_tok(i)?;
                match r {
                    1 => Ok(Self::NonZero),
                    2 => Ok(Self::EvenOdd),
                    _ => Err(s.err_range("1..2"))
                }
            }
            _ => Err(s.err_token("F64")),
        }
    }
}

impl SerJson for FillRule {
    fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {
        match self {
            Self::NonZero => 1.ser_json(d, s),
            Self::EvenOdd => 2.ser_json(d, s),
        }
    }
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

// #[derive(SerJson, DeJson, Debug, Clone)]
// pub struct TextRange {
//     #[nserde(rename = "nm", default)]
//     name: Option<String>,
//     #[nserde(rename = "a", default)]
//     pub style: Option<TextStyle>,
//     #[nserde(rename = "s")]
//     pub selector: TextRangeSelector,
// }

#[derive(SerJson, DeJson, Debug, Clone)]
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

#[derive(PartialEq, Debug)]
// deserialize_with = "keyframes_from_array"
pub enum KeyFramesFromArray {
    Plain(f32),
    List(Vec<f32>),
    LegacyKeyFrames(Vec<LegacyKeyFrame<Vec<f32>>>),
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
                s.next_tok(i)?;
                dbg!(&s.tok);
                match s.tok {
                    nanoserde::DeJsonTok::F64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        dbg!(&res);
                        Ok(Self::List(res))
                    }
                    nanoserde::DeJsonTok::U64(v) => {
                        let mut res = vec![v as f32];
                        s.next_tok(i)?;
                        s.eat_comma_block(i)?;

                        while s.tok != DeJsonTok::BlockClose {
                            res.push(DeJson::de_json(s, i)?);
                            s.eat_comma_block(i)?;
                        }
                        s.block_close(i)?;
                        dbg!(&res);
                        Ok(Self::List(res))
                    }
                    nanoserde::DeJsonTok::CurlyOpen => {
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
                                dbg!(&s.strbuf);
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
                                        return Err(s.err_nf("start_value"))
                                    }
                                },
                                end_value: {
                                    if let Some(t) = _end_value { t } else { default_none() }
                                },
                                start_frame: {
                                    if let Some(t) = _start_frame { t } else { Default::default() }
                                },
                                end_frame: Default::default(),
                                easing_out: { if let Some(t) = _easing_out { t } else { None } },
                                easing_in: { if let Some(t) = _easing_in { t } else { None } },
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
                        dbg!(&res);
                        Ok(Self::LegacyKeyFrames(res))
                    }
                    _ => Err(s.err_token("U64 or {")),
                }
            }
            _ => Err(s.err_token("U64 or [")),
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
            Self::LegacyKeyFrames(_) => todo!()
        }
    }
}

impl<T: Default + Debug> From<&Vec<KeyFrame<T>>> for KeyFramesFromArray {
    fn from(value: &Vec<KeyFrame<T>>) -> KeyFramesFromArray {
        KeyFramesFromArray::List(value.iter().map(|v| v.start_frame).collect())
    }
}

impl<T: Clone + Default + FromTo<Vec<f32>> + Debug> From<&KeyFramesFromArray> for Vec<KeyFrame<T>> {
    fn from(val: &KeyFramesFromArray) -> Vec<KeyFrame<T>> {
        dbg!(val);
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
            KeyFramesFromArray::LegacyKeyFrames(v) => {
                let mut result: Vec<LegacyKeyFrame<Vec<f32>>> = vec![];
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
                dbg!(&res);
                res
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
}

#[derive(PartialEq, Debug, SerJson)]
// deserialize_with = "array_from_array_or_number"
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
        dbg!((s.cur, s.line, s.col));
        dbg!(&s.tok);
        match s.tok {
            nanoserde::DeJsonTok::F64(_) => {
                let r = Self::Primitive(s.as_f64()? as f32);
                s.next_tok(i)?;
                Ok(r)
            }
            nanoserde::DeJsonTok::BlockOpen => {
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
