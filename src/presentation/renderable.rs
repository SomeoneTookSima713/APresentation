use opengl_graphics::GlGraphics;
use graphics::Context;
use graphics;
use super::util;

pub trait Renderable {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics);
}

pub struct RenderableRef<'a> {
    reference: &'a dyn Renderable
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
        ColoredRect { pos: util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).into(), size: util::parse_expression_list(size, &util::DEFAULT_CONTEXT).into(), color: util::parse_expression_list(color, &util::DEFAULT_CONTEXT).into(), alignment: <AlignStr as Into<String>>::into(alignment).into() }
    }
}

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
            pos: util::parse_expression_list(pos, &util::DEFAULT_CONTEXT).into(),
            size: util::parse_expression_list(size, &util::DEFAULT_CONTEXT).into(),
            color: util::parse_expression_list(color, &util::DEFAULT_CONTEXT).into(),
            corner_rounding: util::res_dependent_expr(corner_rounding, &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased),
            alignment: <AlignStr as Into<String>>::into(alignment).into()
        }
    }
}

use crate::render::font;
use crate::app::Application;

#[derive(Clone)]
pub struct TextFont {
    base_font: font::Font,
    bold_font: font::Font
}
impl TextFont {
    /// Creates a new [`TextFont`]
    pub fn new<BaseStr, BoldStr>(app: &Application, base_font_path: BaseStr, bold_font_path: BoldStr) -> TextFont
    where BaseStr: Into<String>, BoldStr: Into<String> {
        TextFont {
            base_font: font::Font::new(app, <BaseStr as Into<String>>::into(base_font_path), 0).expect("invalid font path"),
            bold_font: font::Font::new(app, <BoldStr as Into<String>>::into(bold_font_path), 0).expect("invalid font path")
        }
    }
    /// Creates a new [`TextFont`] using fonts with a face index
    pub fn new_indexed<BaseStr, BoldStr>(app: &Application, base_font_path: (BaseStr, isize), bold_font_path: (BoldStr, isize)) -> TextFont
    where BaseStr: Into<String>, BoldStr: Into<String> {
        TextFont {
            base_font: font::Font::new(app, <BaseStr as Into<String>>::into(base_font_path.0), base_font_path.1).expect("invalid font path or face index"),
            bold_font: font::Font::new(app, <BoldStr as Into<String>>::into(bold_font_path.0), bold_font_path.1).expect("invalid font path or face index")
        }
    }
}

use std::cell::RefCell;

pub enum TextPart<'a> {
    Text {
        text: String,
        bold: bool,
        italic: bool,
        color: util::ExprVector<'a, 4>,
        size: util::ResolutionDependentExpr<'a>,
        font: RefCell<TextFont>
    },
    Tab,
    NewLine
}

pub struct Text<'a> {
    pos: util::ExprVector<'a, 2>,
    text: Vec<TextPart<'a>>,
    wrapping_width: util::ResolutionDependentExpr<'a>,
    size: util::ResolutionDependentExpr<'a>,
    alignment: util::Alignment
}

impl<'a> Text<'a> {
    fn parse<'b>(string: String, base_size: util::ResolutionDependentExpr<'b>, base_font: TextFont, bold: bool, italic: bool, color: util::ExprVector<'a, 4>) -> Vec<TextPart<'b>> {
        let vec = Vec::new();
        
        // TODO

        vec
    }

    pub fn new<PosStr, TextStr, WidthStr, SizeStr, AlignStr, ColStr>(pos: PosStr, text: Vec<TextStr>, wrapping_width: WidthStr, size: SizeStr, alignment: AlignStr, color: ColStr, base_font: TextFont) -> Text<'a>
    where PosStr: Into<String>, TextStr: Into<String>, WidthStr: Into<String>, SizeStr: Into<String>, AlignStr: Into<String>, ColStr: Into<String> {
        let mut text_parts = Vec::new();

        let size_expr = util::res_dependent_expr(<SizeStr as Into<String>>::into(size), &util::DEFAULT_CONTEXT, util::ResExprType::HeightBased);

        let col_expr: util::ExprVector<4> = util::parse_expression_list(<ColStr as Into<String>>::into(color), &util::DEFAULT_CONTEXT).into();

        for into_string in text {
            let string: String = into_string.into();

            text_parts.append(&mut Text::parse(string, size_expr.clone(), base_font.clone(), false, false, col_expr.clone()));

            text_parts.push(TextPart::NewLine);
        }

        Text {
            pos: util::parse_expression_list(<PosStr as Into<String>>::into(pos), &util::DEFAULT_CONTEXT).into(),
            text: text_parts,
            wrapping_width: util::res_dependent_expr(<WidthStr as Into<String>>::into(wrapping_width), &util::DEFAULT_CONTEXT, util::ResExprType::WidthBased),
            size: size_expr,
            alignment: <AlignStr as Into<String>>::into(alignment).into() }
    }
}

impl<'a> Renderable for Text<'a> {
    fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
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
                TextPart::Text { text, bold, italic: _italic, color: _color, size, font } => {
                    let part_size = size.evaluate(view_size[0], view_size[1], time);
                    if curr_max_height<part_size { curr_max_height = part_size }
                    let part_width;
                    match bold {
                        false => { part_width = font.borrow_mut().base_font.size(text, part_size as u32).unwrap().0 },
                        true => { part_width = font.borrow_mut().bold_font.size(text, part_size as u32).unwrap().0 }
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

        curr_max_height = 0.0;

        // Draw the text
        for part in self.text.iter() {
            match part {
                TextPart::Tab => {
                    current_pos.0 += default_size*4.0;
                },
                TextPart::NewLine => {
                    current_pos.0 = starting_pos.0;
                    current_pos.1 += curr_max_height;
                    curr_max_height = 0.0;
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

                    let part_size = font_instance.size(text, part_font_size as u32).unwrap();

                    if current_pos.0 + part_size.0 > max_width {
                        current_pos.0 = starting_pos.0;
                        current_pos.1 += curr_max_height;
                        curr_max_height = 0.0;
                    }

                    font_instance.draw(text, part_font_size as u32, (color_eval.0 as f32, color_eval.1 as f32, color_eval.2 as f32, color_eval.3 as f32), *italic, &context, opengl).unwrap();

                    current_pos.0 += part_size.0;
                }
            }
        }
    }
}