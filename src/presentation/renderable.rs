#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::fmt::Debug;

use opengl_graphics::GlGraphics;
use graphics::{ Context, Transformed };
use graphics;

use super::util; use util::{ ExprVector, Alignment, PropertyError };

/// This trait defines shared behaviour for any object of a slide that should be rendered to the
/// screen (referred to in this project as `Renderable objects` or `objects`).
pub trait Renderable: Debug {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics);

    fn get_base_properties(&self) -> &BaseProperties<'_>;

    /// Basically a copy of the [`Clone::clone`] function because this trait wouldn't be object
    /// safe anymore if I'd require the [`Clone`] trait to be implemented
    fn copy<'b>(&self) -> Box<dyn Renderable + 'b>;
}

/// A wrapper for a reference to any object implementing [`Renderable`]
/// 
#[derive(Clone, Copy)]
pub struct RenderableRef<'a> {
    reference: &'a dyn Renderable
}

impl<'a> From<&'a dyn Renderable> for RenderableRef<'a> {
    fn from(reference: &'a dyn Renderable) -> Self {
        Self { reference }
    }
}

impl<'a, R: Renderable> From<&'a R> for RenderableRef<'a> {
    fn from(value: &'a R) -> Self {
        RenderableRef { reference: value as &'a dyn Renderable }
    }
}
impl<'a, R: Renderable> From<&'a once_cell::sync::Lazy<R>> for RenderableRef<'a> {
    fn from(value: &'a once_cell::sync::Lazy<R>) -> Self {
        RenderableRef { reference: &**value as &'a dyn Renderable }
    }
}
impl<'a> Renderable for RenderableRef<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        self.reference.render(time, context, opengl);
    }

    fn get_base_properties(&self) -> &BaseProperties<'_> {
        self.reference.get_base_properties()
    }

    fn copy<'b>(&self) -> Box<dyn Renderable + 'b> {
        let leaked = Box::leak(Box::new(<Self as Clone>::clone(self))) as &mut (dyn Renderable + 'a) as *mut (dyn Renderable + 'a);
        unsafe {
            let result_ptr = std::mem::transmute::<*mut (dyn Renderable + 'a), *mut (dyn Renderable + 'b)>(leaked);
            Box::from_raw(result_ptr)
        }
    }
}
impl<'a> Debug for RenderableRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reference.fmt(f)
    }
}

/// Contains all basic properties that every Renderable object should have.
#[derive(Debug, Clone)]
pub struct BaseProperties<'a> {
    pub pos: ExprVector<'a, 2>,
    pub size: ExprVector<'a, 2>,
    pub color: ExprVector<'a, 4>,
    pub alignment: Alignment
}

impl<'a> BaseProperties<'a> {
    /// Constructs new base properties of a Renderable object from four [`String`]s defining position, size, color and alignment.
    pub fn new<PStr, SStr, CStr, AStr>(pos: PStr, size: SStr, color: CStr, alignment: AStr) -> Result<Self, PropertyError>
    where
        PStr: Into<String>,
        SStr: Into<String>,
        CStr: Into<String>,
        AStr: Into<String>
    {
        let err = |prop: &'static str| move |e: PropertyError|{
            match e {
                PropertyError::SyntaxError(_, _, desc) => PropertyError::SyntaxError("_".to_owned(), prop.to_owned(), desc),
                _ => e
            }
        };

        Ok(BaseProperties {
            pos: util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).map_err((err)("pos"))?.try_into().map_err((err)("pos"))?,
            size: util::parse_expression_list(size, &util::DEFAULT_CONTEXT).map_err((err)("size"))?.try_into().map_err((err)("size"))?,
            color: util::parse_expression_list(color, &util::DEFAULT_CONTEXT).map_err((err)("color"))?.try_into().map_err((err)("color"))?,
            alignment: Alignment::try_from(alignment.into())?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ColoredRect<'a> {
    base: BaseProperties<'a>
}
impl<'a> Renderable for ColoredRect<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        let view_size = context.get_view_size();
        let color_eval = self.base.color.evaluate_arr(view_size[0], view_size[1], time);
        let pos_eval = self.base.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.base.size.evaluate_tuple(view_size[0], view_size[1], time);
        // Convert the alignment to scalar values.
        //   Subtracting the size of the object multiplied by this value from the position of the
        //   object correctly positions it relative to it's pivot.
        let alignment: (f64, f64) = self.base.alignment.into();
        graphics::rectangle(
            [color_eval[0] as f32, color_eval[1] as f32, color_eval[2] as f32, color_eval[3] as f32],
            [pos_eval.0-size_eval.0*alignment.0,pos_eval.1-size_eval.1*alignment.1,size_eval.0,size_eval.1],
            context.transform, opengl);
    }

    fn get_base_properties(&self) -> &BaseProperties<'_> {
        &self.base
    }

    fn copy<'b>(&self) -> Box<dyn Renderable + 'b>
    where Self: Sized {
        let leaked = Box::leak(Box::new(<Self as Clone>::clone(self))) as &mut (dyn Renderable + 'a) as *mut (dyn Renderable + 'a);
        unsafe {
            let result_ptr = std::mem::transmute::<*mut (dyn Renderable + 'a), *mut (dyn Renderable + 'b)>(leaked);
            Box::from_raw(result_ptr)
        }
    }
}
impl<'a> ColoredRect<'a> {
    pub fn new(base: BaseProperties<'a>) -> Self {
        ColoredRect { base }
    }
}

#[derive(Debug, Clone)]
pub struct RoundedRect<'a> {
    base: BaseProperties<'a>,
    corner_rounding: util::ResolutionDependentExpr<'a>,
}

impl<'a> Renderable for RoundedRect<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        use graphics::Graphics;

        let view_size = context.get_view_size();
        let color_eval = self.base.color.evaluate_arr(view_size[0], view_size[1], time);
        let color_arr = [color_eval[0] as f32, color_eval[1] as f32, color_eval[2] as f32, color_eval[3] as f32];
        let mut pos_eval = self.base.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.base.size.evaluate_tuple(view_size[0], view_size[1], time);
        let corner_rounding_eval = self.corner_rounding.evaluate(view_size[0], view_size[1], time);
        let alignment: (f64, f64) = self.base.alignment.into();
        let arc_tri_count: u32 = (corner_rounding_eval as u32 / 2).max(6);
        
        pos_eval = (pos_eval.0 - size_eval.0 * alignment.0, pos_eval.1 - size_eval.1 * alignment.1);

        opengl.tri_list(&context.draw_state, &color_arr, |tri| {
            graphics::triangulation::with_round_rectangle_tri_list(arc_tri_count, context.transform, [pos_eval.0,pos_eval.1,size_eval.0,size_eval.1], corner_rounding_eval, tri);
        });
    }

    fn get_base_properties<'b>(&'b self) -> &'b BaseProperties<'b> {
        &self.base
    }

    fn copy<'b>(&self) -> Box<dyn Renderable + 'b> {
        let leaked = Box::leak(Box::new(<Self as Clone>::clone(self))) as &mut (dyn Renderable + 'a) as *mut (dyn Renderable + 'a);
        unsafe {
            let result_ptr = std::mem::transmute::<*mut (dyn Renderable + 'a), *mut (dyn Renderable + 'b)>(leaked);
            Box::from_raw(result_ptr)
        }
    }
}
impl<'a> RoundedRect<'a> {
    pub fn new<RoundingStr>(base: BaseProperties<'a>, corner_rounding: RoundingStr) -> Result<Self, PropertyError>
    where RoundingStr: Into<String> {
        Ok(RoundedRect {
            base,
            corner_rounding: util::res_dependent_expr(corner_rounding, &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased)?,
        })
    }
}

use crate::render::font;

pub struct TextFont {
    pub base_font: font::Font,
    pub bold_font: font::Font
}
impl TextFont {
    /// Creates a new [`TextFont`]
    pub fn new<BaseStr, BoldStr>(base_font_path: BaseStr, bold_font_path: BoldStr) -> TextFont
    where BaseStr: Into<String>, BoldStr: Into<String> {
        TextFont {
            base_font: font::Font::new(<BaseStr as Into<String>>::into(base_font_path), 0).expect("invalid font path"),
            bold_font: font::Font::new(<BoldStr as Into<String>>::into(bold_font_path), 0).expect("invalid font path")
        }
    }
    /// Creates a new [`TextFont`] using fonts with a face index
    pub fn new_indexed<BaseStr, BoldStr>(base_font_path: (BaseStr, isize), bold_font_path: (BoldStr, isize)) -> TextFont
    where BaseStr: Into<String>, BoldStr: Into<String> {
        TextFont {
            base_font: font::Font::new(<BaseStr as Into<String>>::into(base_font_path.0), base_font_path.1).expect("invalid font path or face index"),
            bold_font: font::Font::new(<BoldStr as Into<String>>::into(bold_font_path.0), bold_font_path.1).expect("invalid font path or face index")
        }
    }
}

use std::cell::RefCell;

#[derive(Clone)]
pub enum TextPart<'a, 'font> {
    Text {
        text: String,
        bold: bool,
        italic: bool,
        color: util::ExprVector<'a, 4>,
        size: util::ResolutionDependentExpr<'a>,
        font: &'font RefCell<TextFont>
    },
    Tab,
    Space {
        size: util::ResolutionDependentExpr<'a>,
        font: &'font RefCell<TextFont>
    },
    NewLine,
    Placeholder {
        index: String,
        pad_char: &'a str,
        pad_amount: i8,

        bold: bool,
        italic: bool,
        color: util::ExprVector<'a, 4>,
        size: util::ResolutionDependentExpr<'a>,
        font: &'font RefCell<TextFont>
    },
}

impl<'a, 'font> std::fmt::Debug for TextPart<'a, 'font> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => { write!(f, "\"{}\"", text) },
            TextPart::Tab => { write!(f, "\\t") },
            TextPart::Space { size, font } => { write!(f, "\\s") },
            TextPart::NewLine => { write!(f, "\\n") },
            TextPart::Placeholder { index, pad_char, pad_amount, bold, italic, color, size, font } => {
                if *pad_amount<0 {
                    write!(f, "{{{}<{}{{{}}}", pad_char, pad_amount.abs(), index)
                } else {
                    write!(f, "{{{}>{}{{{}}}", pad_amount, pad_amount, index)
                }
            },
        }
    }
}

impl<'a, 'font> TextPart<'a, 'font> {

    pub fn set_bold(&mut self, set: bool) -> Result<(), PropertyError> {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *bold = set,
            _ => {}
        }
        Ok(())
    }
    pub fn set_italic(&mut self, set: bool) -> Result<(), PropertyError> {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *italic = set,
            _ => {}
        }
        Ok(())
    }
    pub fn set_color(&mut self, set: String) -> Result<(), PropertyError> {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *color = util::parse_expression_list(set, &util::DEFAULT_CONTEXT)?.try_into()?,
            _ => {}
        }
        Ok(())
    }
    pub fn set_size(&mut self, set: String) -> Result<(), PropertyError> {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *size = util::res_dependent_expr(set, &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased)?,
            _ => {}
        }
        Ok(())
    }
    pub fn set_font(&mut self, set: &'font RefCell<TextFont>) -> Result<(), PropertyError> {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *font = set,
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Text<'a> {
    base: BaseProperties<'a>,
    text: Vec<TextPart<'a, 'a>>,
    text_alignment: util::Alignment,
    placeholders: HashMap<String, TextPlaceholderExpr<'a>>
}

pub struct TextPlaceholderExpr<'a> {
    /// The function for evaluating the expression's value.
    pub(self) expr: &'a mut dyn Fn(&[f64]) -> f64,
    /// The string the expression was parsed from.
    /// 
    /// Used for debugging.
    pub(self) base_string: String,
    /// The context that was used to construct the evaluation function.
    /// 
    /// Used to recreate the function when cloning.
    pub(self) base_context: &'a meval::Context<'a>,
}
impl<'a> Clone for TextPlaceholderExpr<'a> {
    fn clone(&self) -> Self {
        Self::parse(&self.base_string, self.base_context)
    }
}
impl<'a> Drop for TextPlaceholderExpr<'a> {
    fn drop(&mut self) {
        let boxed_expr = unsafe {
            Box::from_raw(self.expr as *mut dyn Fn(&[f64]) -> f64)
        };
    }
}
impl<'a> Debug for TextPlaceholderExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextPlaceholder({})", self.base_string)
    }
}
impl<'a> TextPlaceholderExpr<'a> {
    fn modify_context(context: meval::Context) -> meval::Context {
        // use chrono::{ Datelike, Timelike, Local };

        // let datetime = Local::now();

        // context.var("day", datetime.day() as f64);
        // context.var("month", datetime.month() as f64);
        // context.var("year", datetime.year() as f64);
        // context.var("hour", datetime.hour() as f64);
        // context.var("minute", datetime.minute() as f64);
        // context.var("second", datetime.second() as f64);

        context
    }

    pub fn parse<S: Into<String>>(expr: S, context: &'a meval::Context) -> Self {
        static FUNC_VARS: [&str; 9] = ["w", "h", "t", "day", "month", "year", "hour", "minute", "second"];

        let expr_string: String = expr.into();

        let expr: meval::Expr = expr_string.as_str().parse().unwrap();
        let ctx = Self::modify_context(context.clone());
        let func = expr.bindn_with_context(ctx, FUNC_VARS.as_slice()).unwrap();
        TextPlaceholderExpr { expr: Box::leak(Box::new(func)), base_string: expr_string, base_context: context }
    }

    pub fn call(&self, width: f64, height: f64, time: f64) -> f64 {
        use chrono::{ Datelike, Timelike, Local };
        let datetime = Local::now();
        (self.expr)(&[
            width,
            height,
            time,
            datetime.day() as f64,
            datetime.month() as f64,
            datetime.year() as f64,
            datetime.hour() as f64,
            datetime.minute() as f64,
            datetime.second() as f64
        ])
    }
}

use regex::Regex;
use once_cell::sync::Lazy;

const PLACEHOLDER_REGEX: Lazy<Regex> = Lazy::new(||Regex::new(r"\{((?<padchar>[^:])(?<paddir>[<>])(?<padamount>\d+))?\{(?<name>[^}]*)\}\}").unwrap());

impl<'a> Text<'a> {
    pub const PLACEHOLDER_AMOUNT: usize = 64;

    fn parse<'b, S: AsRef<str>>(string: String, base_size: util::ResolutionDependentExpr<'b>, base_font: S, bold: bool, italic: bool, color: util::ExprVector<'b, 4>, font_list: &'static HashMap<String, RefCell<TextFont>>) -> Result<Vec<TextPart<'b, 'b>>, PropertyError> {
        use regex::Captures;
        use std::sync::OnceLock;
        lazy_static::lazy_static! {
            static ref BOLD_REGEX: Regex = Regex::new(r"\*\*(?<content>.+?)\*\*").unwrap();
            static ref ITALIC_REGEX: Regex = Regex::new(r"\*(?<content>.+?)\*").unwrap();
            static ref FONT_REGEX: Regex = Regex::new(r"_(?<font>.+?)_(?<content>.+?)__").unwrap();
            static ref COLOR_REGEX: Regex = Regex::new(r"`(?<r>[^;`]+);\s*(?<g>[^;`]+);\s*(?<b>[^;`]+)(;\s*(?<a>[^;`]+))?`(?<content>.+?)``").unwrap();
            static ref SIZE_REGEX: Regex = Regex::new(r"~(?<size>[^~]+?)~(?<content>.+?)~~").unwrap();
        }
        static REGEXES: OnceLock<[Regex; 5]> = OnceLock::new();
        if REGEXES.get().is_none() {
            REGEXES.set([
                SIZE_REGEX.clone(),
                COLOR_REGEX.clone(),
                FONT_REGEX.clone(),
                BOLD_REGEX.clone(),
                ITALIC_REGEX.clone(), 
            ]).map_err(|_| "error initializing regex list").unwrap();
        }

        let regex_error_fn = |str: &'static str| { PropertyError::SyntaxError(
            "Text".to_owned(),
            "text".to_owned(),
            Some(str.to_owned())) };

        let regex_fns: [Box<dyn Fn(&mut TextPart, &Captures, &'static HashMap<String, RefCell<TextFont>>) -> Result<(), PropertyError>>; 5] = [
            Box::new(|part, captures, fonts| {
                let size = captures.name("size")
                    .ok_or((regex_error_fn)("No size expression in size redefinition!"))?
                    .as_str().to_string();
                part.set_size(size)
            }),
            Box::new(|part, captures, fonts| {

                let error_msg = (regex_error_fn)("Invalid or missing color tuple in color redefinition!");

                let alpha = match part {
                    TextPart::Text { text: _, bold: _, italic: _, color, size: _, font: _ } => {
                        color.list[3].base_string.clone()
                    },
                    _ => "1.0".to_owned()
                };

                let r = captures.name("r").ok_or(error_msg.clone())?.as_str();
                let g = captures.name("g").ok_or(error_msg.clone())?.as_str();
                let b = captures.name("b").ok_or(error_msg)?.as_str();
                let a = captures.name("a").map(|m|m.as_str()).unwrap_or(alpha.as_str());

                part.set_color(format!("{};{};{};{}",r,g,b,a))
            }),
            Box::new(|part, captures, fonts| {
                let f = fonts
                    .get(captures.name("font")
                        .ok_or((regex_error_fn)("No font name in font redefinition!"))?
                        .as_str()
                    ).ok_or((regex_error_fn)("Invalid font name in font redefinition!"))?;
                part.set_font(f)
            }),
            Box::new(|part, captures, fonts| part.set_bold(true)),
            Box::new(|part, captures, fonts| part.set_italic(true)),
        ];

        let mut vec = vec![ TextPart::Text { text: string.as_str().into(), bold, italic, color, size: base_size, font: font_list.get(base_font.as_ref()).unwrap() } ];

        let mut construct_vec = Vec::new();

        for (i, regex) in REGEXES.get().unwrap().iter().enumerate() {
            for text_part in vec.into_iter() {
                match text_part {
                    TextPart::Text { ref text, bold, italic, color, size, font } => {
                        let mut last_match_end: usize = 0;
                        for text_captures in regex.captures_iter(text) {
                            let text_match = text_captures.get(0).unwrap();
                            let text_content = text_captures.name("content").expect("No content matched! This shouldn't happen!");
                            construct_vec.push(TextPart::Text { text: text[last_match_end..text_match.start()].into(), bold, italic, color: color.clone(), size: size.clone(), font });
                            let mut modified = TextPart::Text { text: text[text_content.start()..text_content.end()].into(), bold, italic, color: color.clone(), size: size.clone(), font };
                            (regex_fns[i])(&mut modified, &text_captures, font_list)?;
                            construct_vec.push(modified);
                            last_match_end = text_match.end();
                        }
                        construct_vec.push(TextPart::Text { text: text[last_match_end..].into(), bold, italic, color: color.clone(), size: size.clone(), font })
                    },
                    _ => construct_vec.push(text_part)
                }
            }
            vec = std::mem::replace(&mut construct_vec, Vec::new());
        }

        // Find any placeholders and split them from the rest of the text.
        for text_part in vec.into_iter() {
            match text_part {
                TextPart::Text { text, bold, italic, color, size, font } => {
                    let mut leftover_text = text.clone();
                    let mut placeholders_exist = true;
                    while placeholders_exist {
                        if let Some(capture) = PLACEHOLDER_REGEX.captures_iter(leftover_text.clone().leak()).next() {
                            let placeholder_match = capture.get(0).unwrap();
                            let index = capture.name("name").expect("No placeholder name matched! This shouldn't happen!").as_str();
                            let padchar = capture.name("padchar").map(|m| m.as_str()).unwrap_or(" ");
                            let padamount = capture.name("padamount").map(|m| {
                                m.as_str().parse::<i32>().map_err(|_| {
                                    (regex_error_fn)("Invalid placeholder padding amount!")
                                })
                            }).unwrap_or(Ok(0))?;
                            let paddir = capture.name("paddir").map(|m| m.as_str()).unwrap_or("<");
                            
                            let (before, after) = (&leftover_text[..placeholder_match.start()], &leftover_text[placeholder_match.end()..]);
    
                            construct_vec.push(TextPart::Text { text: before.to_owned(), bold, italic, color: color.clone(), size: size.clone(), font });
    
                            construct_vec.push(TextPart::Placeholder {
                                index: index.to_owned(),
                                pad_char: padchar,
                                pad_amount: padamount as i8,
                                bold,
                                italic,
                                color: color.clone(),
                                size: size.clone(),
                                font
                            });

                            leftover_text = after.to_owned();
                        } else {
                            placeholders_exist = false;
                        }
                    }
                    if leftover_text.len()>0 {
                        construct_vec.push(TextPart::Text { text: leftover_text, bold, italic, color: color.clone(), size: size.clone(), font });
                    }
                },
                _ => construct_vec.push(text_part)
            }
        }
        vec = std::mem::replace(&mut construct_vec, Vec::new());

        // Add tabs
        construct_vec = Vec::new();
        for text_part in vec.into_iter() {
            match text_part {
                TextPart::Text { text, bold, italic, color, size, font } => {
                    let mut new_text_parts = vec![text.clone()];
                    while new_text_parts[new_text_parts.len()-1].find('\t').is_some() && new_text_parts[new_text_parts.len()-1].len()>1 {
                        let i = new_text_parts[new_text_parts.len()-1].find('\t').unwrap();
                        let txt = new_text_parts.remove(new_text_parts.len()-1);
                        new_text_parts.push(txt[..i].to_owned());
                        new_text_parts.push(txt[i..i].to_owned());
                        if txt.len()>=i {
                            new_text_parts.push(txt[i+1..].to_owned());
                        }
                    }
                    for txt in new_text_parts.into_iter() {
                        if &txt == "\t" {
                            construct_vec.push(TextPart::Tab);
                        } else {
                            construct_vec.push(TextPart::Text { text: txt, bold, italic, color: color.clone(), size: size.clone(), font });
                        }
                    }
                },
                _ => construct_vec.push(text_part)
            }
        }
        vec = std::mem::replace(&mut construct_vec, Vec::new());

        // Split the text parts at every space or hyphen to allow for text wrapping.
        for c in [' ', '-'] {
            construct_vec = Vec::new();
            for text_part in vec.into_iter() {
                match text_part {
                    TextPart::Text { text, bold, italic, color, size, font } => {
                        let split = text.split(c).collect::<Vec<&str>>();

                        for (i, &txt) in split.iter().enumerate() {
                            construct_vec.push(TextPart::Text { text: txt.into(), bold, italic, color: color.clone(), size: size.clone(), font });
                            if i<split.len()-1 {
                                construct_vec.push(TextPart::Space { size: size.clone(), font });
                            }
                        }
                    },
                    _ => construct_vec.push(text_part)
                }
            }
            vec = construct_vec;
        }

        // Remove any strings of zero length
        vec = vec.into_iter().filter(|p| match &p {
            TextPart::Text { text, bold, italic, color, size, font } => text.len()>0,
            _ => true
        }).collect();

        Ok(vec)
    }

    pub fn new<TextStr, TxtAlignStr>(
        base: BaseProperties<'a>,
        text: Vec<TextStr>,
        base_font: String,
        font_list: &'static HashMap<String, RefCell<TextFont>>,
        placeholders: HashMap<String, TextPlaceholderExpr<'a>>,
        text_alignment: TxtAlignStr
    ) -> Result<Text<'a>, PropertyError>
    where
        TextStr: Into<String>,
        TxtAlignStr: Into<String>, {
        let mut text_parts = Vec::new();

        let size_expr = &base.size.list[1];

        let col_expr = &base.color;

        for into_string in text {
            let string: String = into_string.into();

            for part in Text::parse(string, size_expr.clone(), base_font.clone(), false, false, col_expr.clone(), font_list)? {
                text_parts.push(part);
            }

            text_parts.push(TextPart::NewLine);
        }

        // DEBUG: Check if the parsed text actually got parsed correctly
        // println!("{:?}",text_parts);

        // Text {
        //     pos: util::parse_expression_list(<PosStr as Into<String>>::into(pos), &util::DEFAULT_CONTEXT).try_into().unwrap(),
        //     text: text_parts,
        //     wrapping_width: util::res_dependent_expr(<WidthStr as Into<String>>::into(wrapping_width), &util::DEFAULT_CONTEXT, util::ResExprType::WidthBased),
        //     size: size_expr,
        //     alignment: <AlignStr as Into<String>>::into(alignment).into(),
        //     text_alignment: format!("TOP_{}",<TxtAlignStr as Into<String>>::into(text_alignment)).into(),
        //     placeholders, }
        Ok(Text {
            base,
            text: text_parts,
            text_alignment: format!("TOP_{}",<TxtAlignStr as Into<String>>::into(text_alignment)).try_into()?,
            placeholders
        })
    }

    fn pad_num<'b>(num: f64, pad_amount: i8, pad_char: &str, pad_dir_str: &str) -> &'b str {
        let numstr = num.to_string();
        let mut padstr = String::new();
        if pad_amount-numstr.len() as i8 > 0 {
            for _ in 0..pad_amount as usize-numstr.len() {
                padstr.push_str(pad_char);
            }
        }
        match pad_dir_str {
            "<" => format!("{padstr}{numstr}").leak(),
            ">" => format!("{numstr}{padstr}").leak(),
            _ => padstr.leak()
        }
    }
}

impl<'a> Renderable for Text<'a> {
    fn get_base_properties(&self) -> &BaseProperties<'_> {
        &self.base
    }

    fn copy<'b>(&self) -> Box<dyn Renderable + 'b> {
        let leaked = Box::leak(Box::new(<Self as Clone>::clone(self))) as &mut (dyn Renderable + 'a) as *mut (dyn Renderable + 'a);
        unsafe {
            let result_ptr = std::mem::transmute::<*mut (dyn Renderable + 'a), *mut (dyn Renderable + 'b)>(leaked);
            Box::from_raw(result_ptr)
        }
    }

    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        const ITALIC_ADVANCE_FAC: f64 = 0.10;

        let view_size = context.get_view_size();
        let max_width = self.base.size.list[0].evaluate(view_size[0], view_size[1], time);
        let mut current_pos = self.base.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let alignment: (f64, f64) = self.base.alignment.into();
        let text_align: f64 = self.text_alignment.multipliers().0;
        
        let default_size = self.base.size.list[1].evaluate(view_size[0], view_size[1], time);

        let mut height = 0.0;
        let mut line_widths: Vec<f64> = Vec::with_capacity(self.text.len()/2+4);
        let mut line_heights: Vec<f64> = Vec::with_capacity(self.text.len()/8);
        let mut curr_width = 0.0;
        let mut curr_max_height = default_size;

        // Calculate the dimensions of the object for the alignment
        for part in self.text.iter() {
            match part {
                TextPart::Tab => {
                    let size_incs = default_size*12.0;
                    if (curr_width/size_incs).ceil()*size_incs<=max_width {
                        curr_width = (curr_width/size_incs).ceil()*size_incs;
                    }
                },
                TextPart::NewLine => {
                    line_widths.push(curr_width);
                    line_heights.push(curr_max_height);

                    height += curr_max_height;
                    curr_width = 0.0;
                    curr_max_height = default_size;
                },
                TextPart::Space { size, font } => {
                    let part_size = size.evaluate(view_size[0], view_size[1], time);
                    if part_size>curr_max_height { curr_max_height = part_size; }

                    let width = font.borrow_mut().base_font.size(" ", part_size).0;
                    if curr_width+width<=max_width {
                        curr_width += width as f64;
                    }
                },
                TextPart::Text { text, bold, italic, color, size, font } => {

                    let part_size = size.evaluate(view_size[0], view_size[1], time);
                    if part_size>curr_max_height { curr_max_height = part_size; }
                    let mut part_width;
                    match bold {
                        false => { part_width = font.borrow_mut().base_font.size(text, part_size).0 },
                        true => { part_width = font.borrow_mut().bold_font.size(text, part_size).0 }
                    }
                    if *italic {
                        part_width += part_size * ITALIC_ADVANCE_FAC;
                    }
                    if curr_width+part_width>max_width {
                        height += curr_max_height;
                        line_widths.push(curr_width);
                        line_heights.push(curr_max_height);
                        curr_width = 0.0;
                        curr_max_height = default_size;
                    }
                    curr_width += part_width;
                },
                TextPart::Placeholder { index, pad_char, pad_amount, bold, italic, color, size, font } => {
                    match self.placeholders.get(index) {
                        Some(expr) => {
                            let part_size = size.evaluate(view_size[0], view_size[1], time);
                            if curr_max_height<part_size { curr_max_height = part_size; }

                            let mut part_width;
                            
                            let val = expr.call(view_size[0], view_size[1], time);

                            let pad_dir_str = if *pad_amount<0 {
                                "<"
                            } else {
                                ">"
                            };

                            part_width = if *bold {
                                font.borrow_mut().bold_font.size(Self::pad_num(val, pad_amount.abs(), *pad_char, pad_dir_str), part_size).0
                            } else {
                                font.borrow_mut().base_font.size(Self::pad_num(val, pad_amount.abs(), *pad_char, pad_dir_str), part_size).0
                            };
                            
                            if *italic {
                                part_width += part_size * ITALIC_ADVANCE_FAC;
                            }

                            if curr_width+part_width>max_width {
                                height += curr_max_height;
                                line_widths.push(curr_width);
                                line_heights.push(curr_max_height);
                                curr_width = 0.0;
                                curr_max_height = default_size;
                            }
                            curr_width += part_width;
                        },
                        None => {}
                    }
                }
            }
        }

        line_widths.push(0.0);
        line_heights.push(default_size);

        let mut current_line: usize = 0;

        let starting_pos = (current_pos.0 - max_width*alignment.0, current_pos.1 - height*alignment.1);
        current_pos = (starting_pos.0 + (max_width - line_widths[current_line])*text_align, starting_pos.1);

        // Draw the text
        for part in self.text.iter() {
            match part {
                TextPart::Tab => {
                    // current_pos.0 += default_size*4.0;
                    /*
                    if (curr_width/size_incs).ceil()*size_incs<=max_width {
                        curr_width = (curr_width/size_incs).ceil()*size_incs;
                    }
                    */
                    let size_incs = default_size*12.0;
                    if (current_pos.0/size_incs).ceil()*size_incs - starting_pos.0 <= max_width {
                        current_pos.0 = (current_pos.0/size_incs).ceil()*size_incs;
                    }
                },
                TextPart::NewLine => {
                    current_pos.0 = starting_pos.0 + (max_width - line_widths[current_line])*text_align;
                    current_pos.1 += line_heights[current_line];
                    current_line += 1;
                },
                TextPart::Space { size, font } => {
                    let part_size = size.evaluate(view_size[0], view_size[1], time);
                    let width = font.borrow_mut().base_font.size(" ", part_size).0;
                    current_pos.0 += width as f64;
                },
                TextPart::Text { text, bold, italic, color, size, font } => {
                    let part_font_size = size.evaluate(view_size[0], view_size[1], time);
                    let color_eval = color.evaluate_tuple(view_size[0], view_size[1], time);

                    let mut font_borrow = font.borrow_mut();
                    let font_instance;
                    match bold {
                        true => font_instance = &mut font_borrow.bold_font,
                        false => font_instance = &mut font_borrow.base_font
                    }

                    let mut part_size = font_instance.size(text.clone(), part_font_size);

                    if *italic {
                        // part_size.0 += part_font_size * ITALIC_ADVANCE_FAC/2.0;
                        current_pos.0 += part_font_size * ITALIC_ADVANCE_FAC;
                    }

                    if current_pos.0 + part_size.0 - starting_pos.0 > max_width {
                        current_pos.0 = starting_pos.0 + (max_width - line_widths[current_line])*text_align;
                        current_pos.1 += line_heights[current_line];
                        current_line += 1;
                    }

                    let ctx = context.trans(current_pos.0, current_pos.1 + line_heights[current_line] - part_font_size);

                    font_instance.draw(text, part_font_size, (color_eval.0 as f32, color_eval.1 as f32, color_eval.2 as f32, color_eval.3 as f32), *italic, &ctx, opengl);

                    current_pos.0 += part_size.0;
                },
                TextPart::Placeholder { index, pad_char, pad_amount, bold, italic, color, size, font } => {
                    match self.placeholders.get(index) {
                        Some(expr) => {
                            let part_font_size = size.evaluate(view_size[0], view_size[1], time);
                            let color_eval = color.evaluate_tuple(view_size[0], view_size[1], time);

                            let val = expr.call(view_size[0], view_size[1], time);

                            let pad_dir_str = if *pad_amount<0 {
                                "<"
                            } else {
                                ">"
                            };

                            let text = Self::pad_num(val, pad_amount.abs(), pad_char, pad_dir_str);

                            let mut font_borrow = font.borrow_mut();
                            let font_instance;
                            match bold {
                                true => font_instance = &mut font_borrow.bold_font,
                                false => font_instance = &mut font_borrow.base_font
                            }

                            let mut part_size = font_instance.size(text, part_font_size);

                            if *italic {
                                part_size.0 += part_font_size * ITALIC_ADVANCE_FAC;
                            }

                            if current_pos.0 + part_size.0 - starting_pos.0 > max_width {
                                current_pos.0 = starting_pos.0 + (max_width - line_widths[current_line])*text_align;
                                current_pos.1 += line_heights[current_line];
                                current_line += 1;
                            }

                            let ctx = context.trans(current_pos.0, current_pos.1 + line_heights[current_line] - part_font_size);

                            font_instance.draw(text, part_font_size, (color_eval.0 as f32, color_eval.1 as f32, color_eval.2 as f32, color_eval.3 as f32), *italic, &ctx, opengl);

                            current_pos.0 += part_size.0;
                        },
                        None => {}
                    }
                }
            }
        }
    }
}

use graphics::Image as ImageRect;
use opengl_graphics::Texture;
use std::path::Path;

use std::sync::RwLock;
static IMAGE_TEXTURES: RwLock<Vec<Texture>> = RwLock::new(Vec::new());

#[derive(Clone)]
pub struct Image<'a> {
    base: BaseProperties<'a>,
    texture_path: String,
    texture: usize
}

impl<'a> Debug for Image<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image{{ pos: {:?}, size: {:?}, alignment: {:?}, texture: {} }}",self.base.pos,self.base.size,self.base.alignment,self.texture_path)
    }
}

impl<'a> Image<'a> {
    pub fn new<P: AsRef<Path>>(base: BaseProperties<'a>, path: P) -> Result<Self, PropertyError> {
        use crate::render::sprite::DEFAULT_TEXTURE_SETTINGS;

        let texture_path = path.as_ref().to_str()
            .ok_or(PropertyError::SyntaxError(
                "Image".to_owned(),
                "path".to_owned(),
                Some("Path isn't valid unicode!".to_owned())))?
            .to_owned();
        let texture = Texture::from_path(path, &DEFAULT_TEXTURE_SETTINGS)
            .map_err(|e|PropertyError::SyntaxError(
                "Image".to_owned(),
                "path".to_owned(),
                Some(format!("Loading image at path {texture_path} failed: {e}"))))?;
        
        IMAGE_TEXTURES.write().unwrap().push(texture);

        Ok(Self { base, texture: IMAGE_TEXTURES.read().unwrap().len()-1, texture_path })
    }
}

impl<'a> Renderable for Image<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        use graphics::DrawState;

        let view_size = context.get_view_size();
        let pos_eval = self.base.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.base.size.evaluate_tuple(view_size[0], view_size[1], time);
        let col_eval = self.base.color.evaluate_arr(view_size[0], view_size[1], time).map(|f|f as f32);
        let alignment: (f64, f64) = self.base.alignment.into();

        let rect = ImageRect::new().rect([pos_eval.0-size_eval.0*alignment.0,pos_eval.1-size_eval.1*alignment.1,size_eval.0,size_eval.1]).color(col_eval);

        let lock = IMAGE_TEXTURES.read().unwrap();
        let texture = lock.get(self.texture).unwrap();

        rect.draw(texture, &DrawState::default(), context.transform, opengl);
    }

    fn get_base_properties(&self) -> &BaseProperties<'_> {
        &self.base
    }

    fn copy<'b>(&self) -> Box<dyn Renderable + 'b> {
        let leaked = Box::leak(Box::new(<Self as Clone>::clone(self))) as &mut (dyn Renderable + 'a) as *mut (dyn Renderable + 'a);
        unsafe {
            let result_ptr = std::mem::transmute::<*mut (dyn Renderable + 'a), *mut (dyn Renderable + 'b)>(leaked);
            Box::from_raw(result_ptr)
        }
    }
}