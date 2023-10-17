#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::fmt::Debug;

use opengl_graphics::GlGraphics;
use graphics::{ Context, Transformed };
use graphics;
use super::util;

/// This trait defines shared behaviour for any object of a slide that should be rendered to the
/// screen (referred to in this project as `Renderable objects` or `objects`).
pub trait Renderable: Debug {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics);
}

/// A wrapper for a reference to any object implementing [`Renderable`]
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
}
impl<'a> Debug for RenderableRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reference.fmt(f)
    }
}

#[derive(Debug)]
pub struct ColoredRect<'a> {
    pos: util::ExprVector<'a, 2>,
    size: util::ExprVector<'a, 2>,
    color: util::ExprVector<'a, 4>,
    alignment: util::Alignment
}
impl<'a> Renderable for ColoredRect<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        let view_size = context.get_view_size();
        let color_eval = self.color.evaluate_arr(view_size[0], view_size[1], time);
        let pos_eval = self.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.size.evaluate_tuple(view_size[0], view_size[1], time);
        // Convert the alignment to scalar values.
        //   Subtracting the size of the object multiplied by this value from the position of the
        //   object correctly positions it relative to it's pivot.
        let alignment: (f64, f64) = self.alignment.into();
        graphics::rectangle(
            [color_eval[0] as f32, color_eval[1] as f32, color_eval[2] as f32, color_eval[3] as f32],
            [pos_eval.0-size_eval.0*alignment.0,pos_eval.1-size_eval.1*alignment.1,size_eval.0,size_eval.1],
            context.transform, opengl);
    }
}
impl<'a> ColoredRect<'a> {
    pub fn new<PosStr, SizeStr, ColorStr, AlignStr>(pos: PosStr, size: SizeStr, color: ColorStr, alignment: AlignStr) -> Self
    where PosStr: Into<String>, SizeStr: Into<String>, ColorStr: Into<String>, AlignStr: Into<String> {
        ColoredRect {
            pos: util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            size: util::parse_expression_list(size, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            color: util::parse_expression_list(color, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            alignment: <AlignStr as Into<String>>::into(alignment).into() }
    }
}

#[derive(Debug)]
pub struct RoundedRect<'a> {
    pos: util::ExprVector<'a, 2>,
    size: util::ExprVector<'a, 2>,
    color: util::ExprVector<'a, 4>,
    corner_rounding: util::ResolutionDependentExpr<'a>,
    alignment: util::Alignment
}
impl<'a> Renderable for RoundedRect<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        let view_size = context.get_view_size();
        let color_eval = self.color.evaluate_arr(view_size[0], view_size[1], time);
        let color_arr = [color_eval[0] as f32, color_eval[1] as f32, color_eval[2] as f32, color_eval[3] as f32];
        let mut pos_eval = self.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.size.evaluate_tuple(view_size[0], view_size[1], time);
        let corner_rounding_eval = self.corner_rounding.evaluate(view_size[0], view_size[1], time);
        let alignment: (f64, f64) = self.alignment.into();
        pos_eval = (pos_eval.0 - size_eval.0 * alignment.0, pos_eval.1 - size_eval.1 * alignment.1);
        graphics::ellipse(color_arr,
            [pos_eval.0,pos_eval.1,corner_rounding_eval,corner_rounding_eval],
            context.transform, opengl);
        graphics::ellipse(color_arr,
            [pos_eval.0+size_eval.0-corner_rounding_eval,pos_eval.1,corner_rounding_eval,corner_rounding_eval],
            context.transform, opengl);
        graphics::ellipse(color_arr,
            [pos_eval.0,pos_eval.1+size_eval.1-corner_rounding_eval,corner_rounding_eval,corner_rounding_eval],
            context.transform, opengl);
        graphics::ellipse(color_arr,
            [pos_eval.0+size_eval.0-corner_rounding_eval,pos_eval.1+size_eval.1-corner_rounding_eval,corner_rounding_eval,corner_rounding_eval],
            context.transform, opengl);
        graphics::rectangle(
            color_arr,
            [pos_eval.0+corner_rounding_eval/2.0,pos_eval.1,size_eval.0-corner_rounding_eval,size_eval.1],
            context.transform, opengl);
        graphics::rectangle(
            color_arr,
            [pos_eval.0,pos_eval.1+corner_rounding_eval/2.0,size_eval.0,size_eval.1-corner_rounding_eval],
            context.transform, opengl);
    }
}
impl<'a> RoundedRect<'a> {
    pub fn new<PosStr, SizeStr, ColorStr, RoundingStr, AlignStr>(pos: PosStr, size: SizeStr, color: ColorStr, corner_rounding: RoundingStr, alignment: AlignStr) -> Self
    where PosStr: Into<String>, SizeStr: Into<String>, ColorStr: Into<String>, RoundingStr: Into<String>, AlignStr: Into<String> {
        RoundedRect {
            pos: util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            size: util::parse_expression_list(size, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            color: util::parse_expression_list(color, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            corner_rounding: util::res_dependent_expr(corner_rounding, &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased),
            alignment: <AlignStr as Into<String>>::into(alignment).into()
        }
    }
}

use crate::render::font;

#[derive(Clone)]
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
    NewLine
}

impl<'a, 'font> std::fmt::Debug for TextPart<'a, 'font> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => { write!(f, "\"{}\"", text) },
            TextPart::Tab => { write!(f, "\\t") },
            TextPart::NewLine => { write!(f, "\\n") }
        }
    }
}

impl<'a, 'font> TextPart<'a, 'font> {
    pub fn set_bold(&mut self, set: bool) {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *bold = set,
            _ => {}
        }
    }
    pub fn set_italic(&mut self, set: bool) {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *italic = set,
            _ => {}
        }
    }
    pub fn set_color(&mut self, set: String) {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *color = util::parse_expression_list(set, &util::DEFAULT_CONTEXT).try_into().unwrap(),
            _ => {}
        }
    }
    pub fn set_size(&mut self, set: String) {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *size = util::res_dependent_expr(set, &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased),
            _ => {}
        }
    }
    pub fn set_font(&mut self, set: &'font RefCell<TextFont>) {
        match self {
            TextPart::Text { text, bold, italic, color, size, font } => *font = set,
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct Text<'a> {
    pos: util::ExprVector<'a, 2>,
    text: Vec<TextPart<'a, 'a>>,
    wrapping_width: util::ResolutionDependentExpr<'a>,
    size: util::ResolutionDependentExpr<'a>,
    alignment: util::Alignment,
    placeholders: HashMap<String, TextPlaceholderExpr<'a>>
}

pub struct TextPlaceholderExpr<'a> {
    /// The function for evaluating the expression's value.
    pub(self) expr: Box<dyn Fn(f64, f64, f64) -> f64 + 'a>,
    /// The string the expression was parsed from.
    /// 
    /// Used for debugging.
    pub(self) base_string: String,
    /// The context that was used to construct the evaluation function.
    /// 
    /// Used to recreate the function when cloning.
    pub(self) base_context: &'a meval::Context<'a>,
}
impl<'a> Debug for TextPlaceholderExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextPlaceholder({})", self.base_string)
    }
}
impl<'a> TextPlaceholderExpr<'a> {
    fn modify_context(mut context: meval::Context) -> meval::Context {
        use chrono::{ Datelike, Timelike, Local };

        let datetime = Local::now();

        context.var("day", datetime.day() as f64);
        context.var("month", datetime.month() as f64);
        context.var("year", datetime.year() as f64);
        context.var("hour", datetime.hour() as f64);
        context.var("minute", datetime.minute() as f64);
        context.var("second", datetime.second() as f64);

        context
    }

    pub fn parse<S: Into<String>>(expr: S, context: &'a meval::Context) -> Self {
        let expr_string: String = expr.into();

        let expr: meval::Expr = expr_string.as_str().parse().unwrap();
        let ctx = Self::modify_context(context.clone());
        let func = expr.bind3_with_context(ctx, "w", "h", "t").unwrap();
        TextPlaceholderExpr { expr: Box::new(func), base_string: expr_string, base_context: context }
    }
}

impl<'a> Text<'a> {
    fn parse<'b, S: AsRef<str>>(string: String, base_size: util::ResolutionDependentExpr<'b>, base_font: S, bold: bool, italic: bool, color: util::ExprVector<'b, 4>, font_list: &'static HashMap<String, RefCell<TextFont>>) -> Vec<TextPart<'b, 'b>> {
        use regex::{ Regex, Captures };
        use std::sync::OnceLock;
        lazy_static::lazy_static! {
            static ref BOLD_REGEX: Regex = Regex::new(r"\*\*(?<content>.+?)\*\*").unwrap();
            static ref ITALIC_REGEX: Regex = Regex::new(r"\*(?<content>.+?)\*").unwrap();
            static ref FONT_REGEX: Regex = Regex::new(r"_(?<font>.+?)_(?<content>.+?)__").unwrap();
            static ref COLOR_REGEX: Regex = Regex::new(r"`(?<r>\d+);\s*(?<g>\d+);\s*(?<b>\d+)`(?<content>.+?)``").unwrap();
            static ref SIZE_REGEX: Regex = Regex::new(r"~(?<size>\d+?)~(?<content>.+?)~~").unwrap();
        }
        static REGEXES: OnceLock<[Regex; 5]> = OnceLock::new();
        if REGEXES.get().is_none() {
            REGEXES.set([
                SIZE_REGEX.clone(),
                FONT_REGEX.clone(),
                BOLD_REGEX.clone(),
                ITALIC_REGEX.clone(), 
                COLOR_REGEX.clone(), 
            ]).map_err(|_| "error initializing regex list").unwrap();
        }

        let regex_fns: [Box<dyn Fn(&mut TextPart, &Captures, &'static HashMap<String, RefCell<TextFont>>)>; 5] = [
            Box::new(|part, captures, fonts| part.set_size(captures.name("size").unwrap().as_str().to_string())),
            Box::new(|part, captures, fonts| part.set_font(fonts.get(captures.name("font").unwrap().as_str()).unwrap())),
            Box::new(|part, captures, fonts| part.set_bold(true)),
            Box::new(|part, captures, fonts| part.set_italic(true)),
            Box::new(|part, captures, fonts| part.set_color(format!("{};{};{};1.0",captures.name("r").unwrap().as_str(),captures.name("g").unwrap().as_str(),captures.name("b").unwrap().as_str()))),
        ];

        let mut vec = vec![ TextPart::Text { text: string, bold, italic, color, size: base_size, font: font_list.get(base_font.as_ref()).unwrap() } ];

        let mut construct_vec = Vec::new();

        for (i, regex) in REGEXES.get().unwrap().iter().enumerate() {
            for text_part in vec.into_iter() {
                match text_part {
                    TextPart::Text { ref text, bold, italic, color, size, font } => {
                        let mut last_match_end: usize = 0;
                        for text_captures in regex.captures_iter(text) {
                            let text_match = text_captures.get(0).unwrap();
                            let text_content = text_captures.name("content").unwrap();
                            construct_vec.push(TextPart::Text { text: text[last_match_end..text_match.start()].to_string(), bold, italic, color: color.clone(), size: size.clone(), font });
                            let mut modified = TextPart::Text { text: text[text_content.start()..text_content.end()].to_string(), bold, italic, color: color.clone(), size: size.clone(), font };
                            (regex_fns[i])(&mut modified, &text_captures, font_list);
                            construct_vec.push(modified);
                            last_match_end = text_match.end();
                        }
                        construct_vec.push(TextPart::Text { text: text[last_match_end..].to_string(), bold, italic, color: color.clone(), size: size.clone(), font })
                    },
                    _ => construct_vec.push(text_part)
                }
            }
            vec = std::mem::replace(&mut construct_vec, Vec::new());
        }

        vec
    }

    pub fn new<PosStr, TextStr, WidthStr, SizeStr, AlignStr, ColStr>(
        pos: PosStr,
        text: Vec<TextStr>,
        wrapping_width: WidthStr,
        size: SizeStr,
        alignment: AlignStr,
        color: ColStr,
        base_font: String,
        font_list: &'static HashMap<String, RefCell<TextFont>>,
        placeholders: HashMap<String, TextPlaceholderExpr<'a>>
    ) -> Text<'a>
    where
        PosStr: Into<String>,
        TextStr: Into<String>,
        WidthStr: Into<String>,
        SizeStr: Into<String>,
        AlignStr: Into<String>,
        ColStr: Into<String> {
        let mut text_parts = Vec::new();
        
        //  Update: I redid font rendering, now it's fixed.
        // // Small hack/bodge: The text renders one row to high for some reason, so I just added a newline at the beginning
        // text_parts.push(TextPart::NewLine);

        let size_expr = util::res_dependent_expr(<SizeStr as Into<String>>::into(size), &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased);

        let col_expr: util::ExprVector<4> = util::parse_expression_list(<ColStr as Into<String>>::into(color), &util::DEFAULT_CONTEXT).try_into().unwrap();

        for into_string in text {
            let string: String = into_string.into();

            text_parts.append(&mut Text::parse(string, size_expr.clone(), base_font.clone(), false, false, col_expr.clone(), font_list));

            text_parts.push(TextPart::NewLine);
        }

        // DEBUG: Check if the parsed text actually got parsed correctly
        // println!("{:?}",text_parts);

        Text {
            pos: util::parse_expression_list(<PosStr as Into<String>>::into(pos), &util::DEFAULT_CONTEXT).try_into().unwrap(),
            text: text_parts,
            wrapping_width: util::res_dependent_expr(<WidthStr as Into<String>>::into(wrapping_width), &util::DEFAULT_CONTEXT, util::ResExprType::WidthBased),
            size: size_expr,
            alignment: <AlignStr as Into<String>>::into(alignment).into(),
            placeholders, }
    }

    fn pad_num<'b>(num: f64, pad_amount: i32, pad_char: &str, pad_dir_str: &str) -> &'b str {
        let numstr = num.to_string();
        let mut padstr = String::new();
        if pad_amount-numstr.len() as i32 > 0 {
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
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        use regex::Regex;
        use once_cell::sync::Lazy;
        const PLACEHOLDER_REGEX: Lazy<Regex> = Lazy::new(||Regex::new(r"\{((?<padchar>[^:])(?<paddir>[<>])(?<padamount>\d+))?\{(?<name>[^}]*)\}\}").unwrap());

        let view_size = context.get_view_size();
        let max_width = self.wrapping_width.evaluate(view_size[0], view_size[1], time);
        let mut current_pos = self.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let alignment: (f64, f64) = self.alignment.into();
        
        let default_size = self.size.evaluate(view_size[0], view_size[1], time);

        let mut height = 0.0;
        let mut curr_width = 0.0;
        let mut curr_max_height = 0.0;

        // Calculate the height of the object for the alignment
        for part in self.text.iter() {
            match part {
                TextPart::Tab => { curr_width += default_size*4.0 },
                TextPart::NewLine => { height += curr_max_height; curr_width = 0.0 },
                TextPart::Text { text: text_base, bold, italic: _italic, color: _color, size, font } => {
                    let mut text: String = text_base.clone();
                    for capture in PLACEHOLDER_REGEX.captures_iter(text_base) {
                        let to_replace = capture.get(0).unwrap();
                        let placeholder_index = capture.name("name").unwrap();
                        let padchar = capture.name("padchar").map(|m| m.as_str()).unwrap_or(" ");
                        let padamount = capture.name("padamount").map(|m| m.as_str().parse::<i32>().unwrap()).unwrap_or(0);
                        let paddir = capture.name("paddir").map(|m| m.as_str()).unwrap_or("<");
                        match self.placeholders.get(placeholder_index.as_str()) {
                            Some(expression) => {
                                let eval = (expression.expr)(view_size[0],view_size[1],time);
                                text = text.replace(to_replace.as_str(), Self::pad_num(eval, padamount, padchar, paddir));
                            },
                            // Placeholder can't get replaced, since there's nothing to replace it with
                            None => {}
                        }
                    }
                    let part_size = size.evaluate(view_size[0], view_size[1], time);
                    if curr_max_height<part_size { curr_max_height = part_size }
                    let part_width;
                    match bold {
                        false => { part_width = font.borrow_mut().base_font.size(text, part_size).0 },
                        true => { part_width = font.borrow_mut().bold_font.size(text, part_size).0 }
                    }
                    if curr_width+part_width>max_width {
                        height += curr_max_height;
                        curr_width = 0.0;
                    }
                    curr_width += part_width
                }
            }
        }

        let starting_pos = (current_pos.0 - max_width*alignment.0, current_pos.1 - height*alignment.1);
        current_pos = (current_pos.0 - max_width*alignment.0, current_pos.1 - height*alignment.1);

        // curr_max_height = 0.0;

        // Draw the text
        for part in self.text.iter() {
            match part {
                TextPart::Tab => {
                    current_pos.0 += default_size*4.0;
                },
                TextPart::NewLine => {
                    current_pos.0 = starting_pos.0;
                    current_pos.1 += curr_max_height;
                    // curr_max_height = 0.0;
                },
                TextPart::Text { text: text_base, bold, italic, color, size, font } => {
                    let mut text: String = text_base.clone();
                    for capture in PLACEHOLDER_REGEX.captures_iter(text_base) {
                        let to_replace = capture.get(0).unwrap();
                        let placeholder_index = capture.name("name").unwrap();
                        let padchar = capture.name("padchar").map(|m| m.as_str()).unwrap_or(" ");
                        let padamount = capture.name("padamount").map(|m| m.as_str().parse::<i32>().unwrap()).unwrap_or(0);
                        let paddir = capture.name("paddir").map(|m| m.as_str()).unwrap_or("<");
                        match self.placeholders.get(placeholder_index.as_str()) {
                            Some(expression) => {
                                let eval = (expression.expr)(view_size[0],view_size[1],time);
                                text = text.replace(to_replace.as_str(), Self::pad_num(eval, padamount, padchar, paddir));
                            },
                            // Placeholder can't get replaced, since there's nothing to replace it with
                            None => {}
                        }
                        ()
                    }
                    let part_font_size = size.evaluate(view_size[0], view_size[1], time);
                    let color_eval = color.evaluate_tuple(view_size[0], view_size[1], time);

                    let mut font_borrow = font.borrow_mut();
                    let font_instance;
                    match bold {
                        true => font_instance = &mut font_borrow.bold_font,
                        false => font_instance = &mut font_borrow.base_font
                    }

                    let part_size = font_instance.size(&text, part_font_size);

                    if current_pos.0 + part_size.0 > max_width {
                        current_pos.0 = starting_pos.0;
                        current_pos.1 += curr_max_height;
                        // curr_max_height = 0.0;
                    }

                    let ctx = context.trans(current_pos.0, current_pos.1);

                    font_instance.draw(&text, part_font_size, (color_eval.0 as f32, color_eval.1 as f32, color_eval.2 as f32, color_eval.3 as f32), *italic, &ctx, opengl);

                    current_pos.0 += part_size.0;
                }
            }
        }
    }
}

use graphics::Image as ImageRect;
use opengl_graphics::Texture;
use std::path::Path;

pub struct Image<'a> {
    pos: util::ExprVector<'a, 2>,
    size: util::ExprVector<'a, 2>,
    alignment: util::Alignment,
    texture_path: String,
    texture: Texture
}

impl<'a> Debug for Image<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image{{ pos: {:?}, size: {:?}, alignment: {:?}, texture: {} }}",self.pos,self.size,self.alignment,self.texture_path)
    }
}

impl<'a> Image<'a> {
    pub fn new<P: AsRef<Path>, PosStr, SizeStr, AlignStr>(path: P, pos: PosStr, size: SizeStr, alignment: AlignStr) -> Self
    where PosStr: Into<String>, SizeStr: Into<String>, AlignStr: Into<String> {
        use crate::render::sprite::DEFAULT_TEXTURE_SETTINGS;

        let texture_path = path.as_ref().to_str().unwrap().to_owned();
        let texture = Texture::from_path(path, &DEFAULT_TEXTURE_SETTINGS).unwrap();

        let parsed_pos = util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).try_into().unwrap();
        let parsed_size = util::parse_expression_list(size, &util::DEFAULT_CONTEXT).try_into().unwrap();
        let parsed_alignment = <AlignStr as Into<String>>::into(alignment).into();

        Self { pos: parsed_pos, size: parsed_size, alignment: parsed_alignment, texture, texture_path }
    }
}

impl<'a> Renderable for Image<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        use graphics::DrawState;

        let view_size = context.get_view_size();
        let pos_eval = self.pos.evaluate_tuple(view_size[0], view_size[1], time);
        let size_eval = self.size.evaluate_tuple(view_size[0], view_size[1], time);
        let alignment: (f64, f64) = self.alignment.into();

        let rect = ImageRect::new().rect([pos_eval.0-size_eval.0*alignment.0,pos_eval.1-size_eval.1*alignment.1,size_eval.0,size_eval.1]);

        rect.draw(&self.texture, &DrawState::default(), context.transform, opengl);
    }
}