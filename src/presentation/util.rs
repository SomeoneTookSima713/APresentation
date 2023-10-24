#![allow(dead_code)]

#[allow(unused)]
use log::{ debug as log_dbg, info as log_info, warn as log_warn, error as log_err };

/// The alignment of an object.
#[derive(Clone, Copy, Debug)]
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

impl Alignment {
    pub fn multipliers(&self) -> (f64, f64) {
        (*self).into()
    }
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
            s => panic!("No Alignment found! ({s})")
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

/// The default context used for evaluating mathematical expressions.
pub struct DefaultContext(Lazy<Context<'static>>);
impl DefaultContext {
    pub const fn new() -> Self {
        DefaultContext(Lazy::new(|| {
            use std::f64::consts::{ FRAC_PI_2, PI };
            let mut ctx = Context::new();
            /*
                Current list of available functions:

                Functions built into the meval crate:
                sqrt(x)                      - computes the square root of x
                exp(x)                       - computes e^x
                ln(x)                        - computes the natural log of x
                abs(x)                       - computes the absolute value of x
                sin(x), cos(x), tan(x)       - the base trigonometric functions
                asin(x), acos(x), atan(x)    - the inverses of the base trigonometric functions
                sinh(x), cosh(x), tanh(x)    - the hyperbolic versions of sin, cos and tan
                asinh(x), acosh(x), atanh(x) - the inverses of sinh, cosh and tanh
                floor(x), ceil(x)            - rounds down/up to the nearest integer
                round(x)                     - rounds to the nearest integer
                signum(x)                    - computes the sign of x (-1 if negative, 1 if positive)
                atan2(y,x)                   - computes the four quadrant arc tangent of y and x
                max(x,y,z,...)               - returns the highest supplied value
                min(x,y,z,...)               - returns the lowest supplied value

                Easing functions: (see their graphs at https://easings.net/)
                easeInSine(t)                - an easing function based on the sin function
                easeOutSine(t)               - an easing function based on the sin function
                easeInOutSine(t)             - an easing function based on the sin function
                easeInPow(t,p)               - an easing function based on an exponent (uses p as the power)
                easeOutPow(t,p)              - an easing function based on an exponent (uses p as the power)
                easeInOutPow(t,p)            - an easing function based on an exponent (uses p as the power)
                easeInExp(t)                 - an easing function based on exponential functions
                easeOutExp(t)                - an easing function based on exponential functions
                easeInOutExp(t)              - an easing function based on exponential functions

                Other functions:
                clamp(x,min,max)             - clamps x in the range from min to max
                isEqual(a,b)                 - returns 1 if a and b are equal, otherwise returns 0
                isGreater(a,b)               - returns 1 if a is greater than b, otherwise returns 0
                isLess(a,b)                  - returns 1 if a is less than b, otherwise returns 0
            */
            
            // Easing functions
            ctx.func("easeInSine", |mut t|{ t=t.clamp(0.0,1.0); 1.0-(t*FRAC_PI_2).cos() });
            ctx.func("easeOutSine", |mut t|{ t=t.clamp(0.0,1.0); (t*FRAC_PI_2).sin() });
            ctx.func("easeInOutSine", |mut t|{ t=t.clamp(0.0,1.0); -(t*PI).cos()/2.0 + 0.5 });
            ctx.func2("easeInPow", |mut t,pow|{ t=t.clamp(0.0,1.0); t.powf(pow) });
            ctx.func2("easeOutPow", |mut t,pow|{ t=t.clamp(0.0,1.0); 1.0-(1.0-t).powf(pow) });
            ctx.func2("easeInOutPow", |mut t,pow|{ t=t.clamp(0.0,1.0); if t<0.5 {(2.0_f64).powf(pow-1.0)*t.powf(pow)} else {1.0-(-2.0*t+2.0).powf(pow)/2.0} });
            ctx.func("easeInExp", |mut t|{ t=t.clamp(0.0,1.0); if t==0.0 {0.0} else {2.0_f64.powf(10.0*t-10.0)} });
            ctx.func("easeOutExp", |mut t|{ t=t.clamp(0.0,1.0); if t==1.0 {1.0} else {1.0-2.0_f64.powf(-10.0*t)} });
            ctx.func("easeInOutExp", |mut t|{ t=t.clamp(0.0,1.0); if t==0.0 {0.0} else if t==1.0 {1.0} else if t<0.5 {2_f64.powf(20.0*t-10.0)/2.0} else {1.0-2_f64.powf(-20.0*t+10.0)/2.0} });

            // Random functions
            ctx.func3("clamp",|num,min,max|num.clamp(min, max));
            ctx.func2("isEqual",|a,b|match a==b { true=>1.0, false=>0.0 });
            ctx.func2("isGreater",|a,b|match a>b { true=>1.0, false=>0.0 });
            ctx.func2("isLess",|a,b|match a<b { true=>1.0, false=>0.0 });

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

/// The default context used for evaluating mathematical expressions.
pub static DEFAULT_CONTEXT: DefaultContext = DefaultContext::new();

/// A mathematical expression that depends on the applications resolution.
/// 
/// Allows the usage of a percent-sign inside of expressions.
pub struct ResolutionDependentExpr<'a> {
    /// The function for evaluating the expression's value.
    pub(self) expr: Box<dyn Fn(f64, f64, f64) -> f64 + 'a>,
    /// The string the expression was parsed from.
    /// 
    /// Used for debugging.
    pub(self) base_string: String,
    /// The context that was used to construct the evaluation function.
    /// 
    /// Used to recreate the function when cloning.
    pub(self) base_context: &'a Context<'a>,
    /// The type of the expression.
    /// 
    /// Decides what the percent sign (`%`) gets replaced with.
    /// 
    /// When this value equals [`ResExprType::WidthBased`], any percent sign gets replaced with `/100*w`.
    /// When it equals [`ResExprType::HeightBased`], percent signs get replaced with `/100*h`.
    pub(self) base_expr_type: ResExprType
}

impl<'a> Clone for ResolutionDependentExpr<'a> {
    fn clone(&self) -> Self {
        // Reconstruct the expression
        res_dependent_expr(&self.base_string, self.base_context, self.base_expr_type)
    }
}

use std::fmt::Debug;
impl<'a> Debug for ResolutionDependentExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResolutionDependentExpr({})", self.base_string)
    }
}

impl<'a> ResolutionDependentExpr<'a> {
    pub fn evaluate(&self, width: f64, height: f64, time: f64) -> f64 {
        (self.expr)(width,height,time)
    }
}

/// A list/tuple of expressions.
#[derive(Clone)]
pub struct ExprVector<'a, const N: usize> {
    pub(self) list: [ResolutionDependentExpr<'a>; N]
}

impl<'a, const N: usize> Debug for ExprVector<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExprVector<{}>(", N)?;
        for (i, expr) in self.list.iter().enumerate() {
            write!(f,"{:?}", expr)?;
            if i<N-1 {
                write!(f,", ")?;
            }
        }
        write!(f, ")")
    }
}

impl<'a, const N: usize> From<[ResolutionDependentExpr<'a>; N]> for ExprVector<'a, N> {
    fn from(value: [ResolutionDependentExpr<'a>; N]) -> Self {
        ExprVector { list: value }
    }
}

impl<'a, const N: usize> TryFrom<Vec<ResolutionDependentExpr<'a>>> for ExprVector<'a, N> {
    type Error = String;

    fn try_from(value: Vec<ResolutionDependentExpr<'a>>) -> Result<Self, Self::Error> {
        let list = value.try_into().map_err(|v| format!("amount of given Expressions doesn't match the required amount of {} ({:?})", N, v))?;
        Ok(ExprVector { list })
    }
}

impl<'a, const N: usize> ExprVector<'a, N> {
    /// Evaluates all expressions into an array of size `N`
    pub fn evaluate_arr(&self, width: f64, height: f64, time: f64) -> [f64; N] {
        self.list.iter().map(|v| v.evaluate(width, height, time)).collect::<Vec<f64>>().try_into().unwrap()
    }
}
impl<'a> ExprVector<'a, 2> {
    /// Evaluates all expressions into a tuple of 2 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64) {
        (self.list[0].evaluate(width, height, time),self.list[1].evaluate(width, height, time))
    }
}
impl<'a> ExprVector<'a, 3> {
    /// Evaluates all expressions into a tuple of 3 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64, f64) {
        (
            self.list[0].evaluate(width, height, time),
            self.list[1].evaluate(width, height, time),
            self.list[2].evaluate(width, height, time)
        )
    }
}
impl<'a> ExprVector<'a, 4> {
    /// Evaluates all expressions into a tuple of 4 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64) -> (f64, f64, f64, f64) {
        (
            self.list[0].evaluate(width, height, time),
            self.list[1].evaluate(width, height, time),
            self.list[2].evaluate(width, height, time),
            self.list[3].evaluate(width, height, time)
        )
    }
}

/// Defines on which dimension of the application window a [`ResolutionDependentExpr`] is relative to.
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

/// Parses a string as a function in relation to width, height and time.
/// 
/// These expressions also support the percent-sign (`%`). It functions like the percent sign in
/// CSS.
/// It gets replaced with '/100*w' or '/100*h' when parsing the expression (which one it is depends
/// on the specified [`ResExprType`]).
/// 
/// Example: `50%` = `50/100*w` = `0.5*w` (`50%` refers to half of the window's width)
pub fn res_dependent_expr<'a, S: Into<String>>(expr: S, context: &'a Context, expr_type: ResExprType) -> ResolutionDependentExpr<'a> {
    // Replace percent sign to be able to parse it with meval's parser.
    let string = <S as Into<String>>::into(expr).replace("%", &("/100*".to_owned()+expr_type.str()));

    // Parse the expression and bind it to a function with three arguments
    // (the window's dimensions and time)
    let parsed_expr = string.clone().parse::<Expr>().unwrap();
    match parsed_expr.bind3_with_context(context, "w", "h", "t") {
        Ok(e) => ResolutionDependentExpr { expr: Box::new(e), base_string: string, base_context: context, base_expr_type: expr_type },
        Err(err) => panic!("Expression parsing failed!\n\t{err}")
    }
}

/// Parses a list of expressions separated by semicolons using the [`res_dependent_expr()`] function.
pub fn parse_expression_list<'a, S: Into<String>>(string: S, context: &'a Context) -> Vec<ResolutionDependentExpr<'a>> {
    let mut expr_vec = Vec::new();

    for (i,expression) in <S as Into<String>>::into(string).split(";").enumerate() {
        expr_vec.push(res_dependent_expr(expression.to_owned(), context, match i%2 { 0 => ResExprType::WidthBased, _ => ResExprType::HeightBased }));
    }

    expr_vec
}