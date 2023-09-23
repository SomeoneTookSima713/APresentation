#![allow(dead_code)]

#[derive(Clone, Copy)]
pub enum Alignment {
    TopLeft,
    TopRight,
    TopCentered,
    MidLeft,
    MidRight,
    MidCentered,
    BottomLeft,
    BottomRight,
    BottomCentered
}

impl Into<(f64,f64)> for Alignment {
    fn into(self) -> (f64,f64) {
        match self {
            Self::TopLeft => (0.0,0.0),
            Self::TopRight => (1.0,0.0),
            Self::TopCentered => (0.5,0.0),
            Self::MidLeft => (0.0,0.5),
            Self::MidRight => (1.0,0.5),
            Self::MidCentered => (0.5,0.5),
            Self::BottomLeft => (0.0,1.0),
            Self::BottomRight => (1.0,1.0),
            Self::BottomCentered => (0.5,1.0)
        }
    }
}

impl From<&str> for Alignment {
    fn from(value: &str) -> Self {
        match value {
            "TOP_LEFT" | "TopLeft" => Alignment::TopLeft,
            "TOP_RIGHT" | "TopRight" => Alignment::TopRight,
            "TOP_CENTERED" | "TopCentered" => Alignment::TopCentered,
            "MID_LEFT" | "MidLeft" => Alignment::MidLeft,
            "MID_RIGHT" | "MidRight" => Alignment::MidRight,
            "MID_CENTERED" | "MidCentered" => Alignment::MidCentered,
            "BOTTOM_LEFT" | "BottomLeft" => Alignment::BottomLeft,
            "BOTTOM_RIGHT" | "BottomRight" => Alignment::BottomRight,
            "BOTTOM_CENTERED" | "BottomCentered" => Alignment::BottomCentered,
            _ => panic!("No Alignment found!")
        }
    }
}
impl From<String> for Alignment {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}

use std::hash::{ Hasher, BuildHasher };

/// Primitively simple hashing algorithm to simply ensure that a [`HashMap`]s ordering when using [`u8`]s as keys is known at compile-time.
pub struct SimplestHasher {
    bytes: Vec<u8>
}

impl Default for SimplestHasher {
    fn default() -> Self {
        SimplestHasher { bytes: Vec::new() }
    }
}

impl Hasher for SimplestHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.bytes.push(i);
    }

    fn finish(&self) -> u64 {
        let mut bytes: Vec<u8> = self.bytes.iter().map(|val| *val).collect();
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        bytes[0] as u64 + (bytes[1] as u64).wrapping_shl(8) + (bytes[2] as u64).wrapping_shl(16) + (bytes[3] as u64).wrapping_shl(24)
    }
}

impl BuildHasher for SimplestHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        SimplestHasher::default()
    }
}

use meval::{ Expr, Context };
use std::ops::Deref;
use once_cell::sync::Lazy;

pub struct DefaultContext(Lazy<Context<'static>>);
impl DefaultContext {
    pub const fn new() -> Self {
        DefaultContext(Lazy::new(|| {
            let ctx = Context::new();
        
            ctx
        }))
    }
}
impl Deref for DefaultContext {
    type Target = Lazy<Context<'static>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
unsafe impl Send for DefaultContext {}
unsafe impl Sync for DefaultContext {}

pub static DEFAULT_CONTEXT: DefaultContext = DefaultContext::new();

pub struct ResolutionDependentExpr<'a> {
    pub(self) expr: Box<dyn Fn(f64, f64, f64) -> f64 + 'a>,
    pub(self) base_string: String,
    pub(self) base_context: &'a Context<'a>,
    pub(self) base_expr_type: ResExprType
}

impl<'a> Clone for ResolutionDependentExpr<'a> {
    fn clone(&self) -> Self {
        res_dependent_expr(&self.base_string, self.base_context, self.base_expr_type)
    }
}

use std::fmt::Debug;
impl<'a> Debug for ResolutionDependentExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResolutionDependentExpr()")
    }
}

impl<'a> ResolutionDependentExpr<'a> {
    pub fn evaluate(&self, width: f64, height: f64, time: f64) -> f64 {
        (self.expr)(width,height,time)
    }
}

#[derive(Clone)]
pub struct ExprVector<'a, const N: usize> {
    pub(self) list: [ResolutionDependentExpr<'a>; N]
}

impl<'a, const N: usize> From<[ResolutionDependentExpr<'a>; N]> for ExprVector<'a, N> {
    fn from(value: [ResolutionDependentExpr<'a>; N]) -> Self {
        ExprVector { list: value }
    }
}

// impl<'a, const N: usize> From<Vec<ResolutionDependentExpr<'a>>> for ExprVector<'a, N> {
//     fn from(value: Vec<ResolutionDependentExpr<'a>>) -> Self {
//         ExprVector { list: value.try_into().unwrap() }
//     }
// }

impl<'a, const N: usize> TryFrom<Vec<ResolutionDependentExpr<'a>>> for ExprVector<'a, N> {
    type Error = String;

    fn try_from(value: Vec<ResolutionDependentExpr<'a>>) -> Result<Self, Self::Error> {
        let list = value.try_into().map_err(|_| format!("amount of given Expressions doesn't match the required amount of {}", N))?;
        Ok(ExprVector { list })
    }
}

impl<'a, const N: usize> ExprVector<'a, N> {

    pub fn evaluate_arr(&self, width: f64, height: f64, time: f64) -> [f64; N] {
        self.list.iter().map(|v| v.evaluate(width, height, time)).collect::<Vec<f64>>().try_into().unwrap()
    }
}
impl<'a> ExprVector<'a, 2> {
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64) {
        (self.list[0].evaluate(width, height, time),self.list[1].evaluate(width, height, time))
    }
}
impl<'a> ExprVector<'a, 3> {
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64, f64) {
        (
            self.list[0].evaluate(width, height, time),
            self.list[1].evaluate(width, height, time),
            self.list[2].evaluate(width, height, time)
        )
    }
}
impl<'a> ExprVector<'a, 4> {
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64, f64, f64) {
        (
            self.list[0].evaluate(width, height, time),
            self.list[1].evaluate(width, height, time),
            self.list[2].evaluate(width, height, time),
            self.list[3].evaluate(width, height, time)
        )
    }
}

#[derive(Clone, Copy)]
pub enum ResExprType {
    WidthBased,
    HeightBased
}
impl Into<&'static str> for ResExprType {
    fn into(self) -> &'static str {
        match self {
            Self::WidthBased => "w",
            Self::HeightBased => "h"
        }
    }
}
impl ResExprType {
    pub fn str(self) -> &'static str {
        self.into()
    }
}

/// Parses a string into a mathematical expression, bound into a function with an argument for width, height and time
pub fn res_dependent_expr<'a, S: Into<String>>(expr: S, context: &'a Context, expr_type: ResExprType) -> ResolutionDependentExpr<'a> {
    let string = <S as Into<String>>::into(expr).replace("%", &("/100*".to_owned()+expr_type.str()));
    let parsed_expr = string.clone().parse::<Expr>().unwrap();
    match parsed_expr.bind3_with_context(context, "w", "h", "t") {
        Ok(e) => ResolutionDependentExpr { expr: Box::new(e), base_string: string, base_context: context, base_expr_type: expr_type },
        Err(err) => panic!("Expression parsing failed!\n\t{err}")
    }
}

/// Parses a list of expressions separated by semicolons with the [`res_dependent_expr()`]-function
pub fn parse_expression_list<'a, S: Into<String>>(string: S, context: &'a Context) -> Vec<ResolutionDependentExpr<'a>> {
    let mut expr_vec = Vec::new();

    for (i,expression) in <S as Into<String>>::into(string).split(";").enumerate() {
        expr_vec.push(res_dependent_expr(expression.to_owned(), context, match i%2 { 0 => ResExprType::WidthBased, _ => ResExprType::HeightBased }));
    }

    expr_vec
}