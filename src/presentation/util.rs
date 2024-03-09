#![allow(dead_code)]

#[allow(unused)]
use log::{ debug as log_dbg, info as log_info, warn as log_warn, error as log_err };

use std::collections::HashMap;
use std::sync::Arc;

/// All errors that can happen while constructing, converting or using mainly [`ExprVector`]s and [`Alignment`]s.
/// 
/// This error type also includes syntax errors when constructing Renerable objects.
#[derive(Clone)]
pub enum PropertyError {
    /// This error indicates that you supplied the incorrect number of expressions while
    /// constructing an [`ExprVector`] from a [`Vec`].
    MismatchedExprCount,
    /// This error indicates that you supplied an invalid string while constructing an
    /// [`Alignment`] from a [`String`].
    BadAlignment,
    /// This error indicates a syntax error in the properties of a Renderable object that isn't
    /// covered in the other possible errors.
    /// 
    /// The first string is the name of the Renderable, the second is the name of the affected
    /// property and the optional third string is a more precise description of the error.
    SyntaxError(String, String, Option<String>),
    /// This error indicates that something went wrong doing something lua-related.
    LuaError(mlua::Error),
    /// This error is a collection of errors, in case multiple things could have gone wrong or
    /// there are multiple possible causes for the program not working.
    MultiError(Vec<PropertyError>)
}
impl PropertyError {
    /// Returns the values of a syntax error, converting all non-syntax related errors to syntax
    /// errors and adding a description if one is missing, both using the supplied default values.
    pub fn syntax_error<S: AsRef<str>>(&self, rtype: S, property: S, desc: S) -> (String, String, String) {
        let rtype: &str = rtype.as_ref();
        let property: &str = property.as_ref();
        let desc: &str = desc.as_ref();

        fn me_rends<S: AsRef<str> + Copy>(errors: &Vec<PropertyError>, rtype: S, property: S, desc: S) -> String {
            errors.iter().map(|e|e.syntax_error(rtype, property, desc).0).collect::<Vec<_>>().join(" / ")
        }
        fn me_props<S: AsRef<str> + Copy>(errors: &Vec<PropertyError>, rtype: S, property: S, desc: S) -> String {
            errors.iter().map(|e|e.syntax_error(rtype, property, desc).1).collect::<Vec<_>>().join(" / ")
        }
        fn me_descs<S: AsRef<str> + Copy>(errors: &Vec<PropertyError>, rtype: S, property: S, desc: S) -> String {
            errors.iter().map(|e|e.syntax_error(rtype, property, desc).2).collect::<Vec<_>>().join(" / ")
        }

        match self {
            Self::MismatchedExprCount => (
                rtype.to_owned(),
                property.to_owned(),
                "Mismatched expression count!".to_owned()
            ),
            Self::BadAlignment => (
                rtype.to_owned(),
                property.to_owned(),
                "Invalid alignment string!".to_owned()
            ),
            Self::SyntaxError(t, p, d) => (
                t.clone(),
                p.clone(),
                d.as_ref().map(|s|s.as_str()).unwrap_or(desc).to_owned()
            ),
            Self::LuaError(e) => (
                rtype.to_owned(),
                property.to_owned(),
                e.to_string()
            ),
            Self::MultiError(errors) => (
                me_rends(errors, rtype, property, desc),
                me_props(errors, rtype, property, desc),
                me_descs(errors, rtype, property, desc)
            )
        }
    }
}

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


impl<'a> Into<String> for &'a Alignment {
    fn into(self) -> String {
        match *self {
            Alignment::TopLeft => "TopLeft".to_owned(),
            Alignment::TopRight => "TopRight".to_owned(),
            Alignment::TopCentered => "TopCentered".to_owned(),
            Alignment::MidLeft => "MidLeft".to_owned(),
            Alignment::MidRight => "MidRight".to_owned(),
            Alignment::MidCentered => "MidCentered".to_owned(),
            Alignment::BottomLeft => "BottomLeft".to_owned(),
            Alignment::BottomRight => "BottomRight".to_owned(),
            Alignment::BottomCentered => "BottomCentered".to_owned()
        }
    }
}

impl ToString for Alignment {
    fn to_string(&self) -> String {
        self.into()
    }
}

impl<'a> TryFrom<&'a str> for Alignment {
    type Error = PropertyError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "TOP_LEFT" | "TopLeft" => Ok(Alignment::TopLeft),
            "TOP_RIGHT" | "TopRight" => Ok(Alignment::TopRight),
            "TOP_CENTERED" | "TopCentered" => Ok(Alignment::TopCentered),
            "MID_LEFT" | "MidLeft" => Ok(Alignment::MidLeft),
            "MID_RIGHT" | "MidRight" => Ok(Alignment::MidRight),
            "MID_CENTERED" | "MidCentered" => Ok(Alignment::MidCentered),
            "BOTTOM_LEFT" | "BottomLeft" => Ok(Alignment::BottomLeft),
            "BOTTOM_RIGHT" | "BottomRight" => Ok(Alignment::BottomRight),
            "BOTTOM_CENTERED" | "BottomCentered" => Ok(Alignment::BottomCentered),
            _ => Err(PropertyError::BadAlignment)
        }
    }
}
impl TryFrom<String> for Alignment {
    type Error = PropertyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value.as_str())
    }
}

use meval::{ Expr, Context };
use std::ops::Deref;
use once_cell::sync::Lazy;

/// The default context used for evaluating mathematical expressions.
pub struct DefaultContext;
impl DefaultContext {
    pub const fn new() -> Lazy<Arc<Context<'static>>> {
        Lazy::new(|| {
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
                easeInPow(t,p)               - an easing function based on a power function (uses p as the power)
                easeOutPow(t,p)              - an easing function based on a power function (uses p as the power)
                easeInOutPow(t,p)            - an easing function based on a power function (uses p as the power)
                easeInExp(t)                 - an easing function based on exponential functions
                easeOutExp(t)                - an easing function based on exponential functions
                easeInOutExp(t)              - an easing function based on exponential functions

                Other functions:
                clamp(x,min,max)             - clamps x in the range from min to max
                isEqual(a,b)                 - returns 1 if a and b are equal, otherwise returns 0
                isGreater(a,b)               - returns 1 if a is greater than b, otherwise returns 0
                isLess(a,b)                  - returns 1 if a is less than b, otherwise returns 0
                mod(a,b)                     - returns the the remainder of the division of a by b, also called the modulo of a and b
            */
            
            // Easing functions
            ctx.func("easeInSine", |mut t|{ t=t.clamp(0.0,1.0); 1.0-(t*FRAC_PI_2).cos() });
            ctx.func("easeOutSine", |mut t|{ t=t.clamp(0.0,1.0); (t*FRAC_PI_2).sin() });
            ctx.func("easeInOutSine", |mut t|{ t=t.clamp(0.0,1.0); -(t*PI).cos()/2.0 + 0.5 });
            ctx.func2("easeInPow", |mut t,pow|{ t=t.clamp(0.0,1.0); t.powf(pow) });
            ctx.func2("easeOutPow", |mut t,pow|{ t=t.clamp(0.0,1.0); 1.0-(1.0-t).powf(pow) });
            ctx.func2("easeInOutPow", |mut t,pow|{ t=t.clamp(0.0,1.0); if t<0.5 {(2.0_f64).powf(pow-1.0)*t.powf(pow)} else {1.0-(-2.0*t+2.0).powf(pow)/2.0} });
            ctx.func("easeInExpo", |mut t|{ t=t.clamp(0.0,1.0); if t==0.0 {0.0} else {2.0_f64.powf(10.0*t-10.0)} });
            ctx.func("easeOutExpo", |mut t|{ t=t.clamp(0.0,1.0); if t==1.0 {1.0} else {1.0-2.0_f64.powf(-10.0*t)} });
            ctx.func("easeInOutExpo", |mut t|{ t=t.clamp(0.0,1.0); if t==0.0 {0.0} else if t==1.0 {1.0} else if t<0.5 {2_f64.powf(20.0*t-10.0)/2.0} else {1.0-2_f64.powf(-20.0*t+10.0)/2.0} });
            ctx.func("easeInCirc", |mut t|{ t=t.clamp(0.0, 1.0); 1.0-(1.0-t.powi(2)).sqrt() });
            ctx.func("easeOutCirc", |mut t|{ t=t.clamp(0.0, 1.0); (1.0-(t-1.0).powi(2)).sqrt() });
            ctx.func("easeInOutCirc", |mut t|{ t=t.clamp(0.0, 1.0); if t<0.5 {1.0-(1.0-t.powi(2)).sqrt()} else {(1.0-(t-1.0).powi(2)).sqrt()} });

            // Random functions
            ctx.func3("clamp",|num,min,max|num.clamp(min, max));
            ctx.func2("isEqual",|a,b|match a==b { true=>1.0, false=>0.0 });
            ctx.func2("isGreater",|a,b|match a>b { true=>1.0, false=>0.0 });
            ctx.func2("isLess",|a,b|match a<b { true=>1.0, false=>0.0 });
            ctx.func2("mod", |a,b|a%b);

            Arc::new(ctx)
        })
    }
}

/// The default context used for evaluating mathematical expressions.
pub static DEFAULT_CONTEXT: crate::util::AssumeThreadSafe<Lazy<Arc<Context<'static>>>> = crate::util::AssumeThreadSafe(DefaultContext::new());

/// A mathematical expression that depends on the applications resolution.
/// 
/// Allows the usage of a percent-sign inside of expressions.
pub enum ResolutionDependentExpr {
    MathExpr {
        /// The function for evaluating the expression's value.
        expr: Arc<dyn Fn(f64, f64, f64) -> f64>,
        /// The string the expression was parsed from.
        /// 
        /// Used for debugging.
        base_string: String,
        /// The context that was used to construct the evaluation function.
        /// 
        /// Used to recreate the function when cloning.
        base_context: Arc<Context<'static>>,
        /// The type of the expression.
        /// 
        /// Decides what the percent sign (`%`) gets replaced with.
        /// 
        /// When this value equals [`ResExprType::WidthBased`], any percent sign gets replaced with `/100*w`.
        /// When it equals [`ResExprType::HeightBased`], percent signs get replaced with `/100*h`.
        base_expr_type: ResExprType
    },
    LuaExpr(mlua::Function<'static>, String)
}

impl Clone for ResolutionDependentExpr {
    fn clone(&self) -> Self {
        match self {
            Self::MathExpr { expr: _, base_string, base_context, base_expr_type } => {
                // Reconstruct the expression
                res_dependent_expr(base_string.clone(), base_context.clone(), *base_expr_type).unwrap_or_else(|_|panic!("Reconstruction of an expression failed! This shouldn't happen!"))
            },
            Self::LuaExpr(func, func_str) => {
                Self::LuaExpr(func.clone(), func_str.clone())
            }
        }
    }
}

impl mlua::UserData for ResolutionDependentExpr {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("evaluate", |lua, s, args: (f64, f64, f64, HashMap<String, mlua::Value>)| {
            s.evaluate(args.0, args.1, args.2, &args.3).map_err(|e| mlua::Error::runtime(e.to_string()))
        });
    }
}

impl<'a> mlua::UserData for &'a ResolutionDependentExpr {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("evaluate", |lua, s, args: (f64, f64, f64, HashMap<String, mlua::Value>)| {
            s.evaluate(args.0, args.1, args.2, &args.3).map_err(|e| mlua::Error::runtime(e.to_string()))
        });
    }
}

use std::fmt::Debug;
impl Debug for ResolutionDependentExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MathExpr { expr: _, base_string, base_context: _, base_expr_type: _ } => {
                write!(f, "ResolutionDependentExpr({})", base_string)
            },
            Self::LuaExpr(func, str) => {
                write!(f,"ResolutionDependentExpr({str})")
            }
        }
    }
}

impl ResolutionDependentExpr {
    pub fn evaluate(&self, width: f64, height: f64, time: f64, object: &HashMap<String, mlua::Value>) -> anyhow::Result<ExprEval> {
        match self {
            Self::MathExpr { expr, base_string: _, base_context: _, base_expr_type: _ } => {
                Ok(ExprEval::F64((expr)(width,height,time)))
            },
            Self::LuaExpr(func, _) => {
                use mlua::FromLuaMulti;

                // TODO: Replace this clone, as it's getting called every frame and clones a HashMap.
                let val: mlua::MultiValue = func.call(object.clone())?;
                if let Ok(str) = String::from_lua_multi(val.clone(), crate::LUA_INSTANCE.get().unwrap()) {
                    Ok(ExprEval::String(str))
                } else if let Ok(float) = f64::from_lua_multi(val, crate::LUA_INSTANCE.get().unwrap()) {
                    Ok(ExprEval::F64(float))
                } else {
                    anyhow::bail!("Invalid return type of lua expression!")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprEval {
    F64(f64),
    String(String)
}

impl<'lua> mlua::IntoLua<'lua> for ExprEval {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        match self {
            Self::F64(val) => val.into_lua(lua),
            Self::String(val) => val.into_lua(lua)
        }
    }
}

/// A list/tuple of expressions.
#[derive(Clone)]
pub struct ExprVector<const N: usize> {
    pub(crate) list: [ResolutionDependentExpr; N]
}

impl<const N: usize> mlua::UserData for ExprVector<N> {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("evaluate", |lua, s, args: (f64, f64, f64, HashMap<String, mlua::Value>)| {
            s.evaluate_arr(args.0, args.1, args.2, &args.3)
                .map(|arr| Vec::from(arr))
                .map_err(|e| mlua::Error::runtime(e.to_string()))
        });
    }
}

impl<'a, const N: usize> mlua::UserData for &'a ExprVector<N> {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("evaluate", |lua, s, args: (f64, f64, f64, HashMap<String, mlua::Value>)| {
            s.evaluate_arr(args.0, args.1, args.2, &args.3)
                .map(|arr| Vec::from(arr))
                .map_err(|e| mlua::Error::runtime(e.to_string()))
        });
    }
}

impl<const N: usize> Debug for ExprVector<N> {
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

impl<const N: usize> From<[ResolutionDependentExpr; N]> for ExprVector<N> {
    fn from(value: [ResolutionDependentExpr; N]) -> Self {
        ExprVector { list: value }
    }
}

impl<const N: usize> TryFrom<Vec<ResolutionDependentExpr>> for ExprVector<N> {
    type Error = PropertyError;

    fn try_from(value: Vec<ResolutionDependentExpr>) -> Result<Self, Self::Error> {
        // let list = value.try_into().map_err(|v| format!("amount of given Expressions doesn't match the required amount of {} ({:?})", N, v))?;

        let value_len = value.len();
        let value_str = format!("{value:#?}");

        let list = value.try_into().map_err(|_| {
            log::error!("ExprVector::try_from(): Invalid nubmer of expressions supplied (expected: {N}, got: {value_len})\n\t{value_str}");
            PropertyError::MismatchedExprCount
        })?;
        Ok(ExprVector { list })
    }
}

impl<const N: usize> ExprVector<N> {
    /// Evaluates all expressions into an array of size `N`
    pub fn evaluate_arr(&self, width: f64, height: f64, time: f64, object: &HashMap<String, mlua::Value>) -> anyhow::Result<[ExprEval; N]> {
        let mut errors = Vec::new();

        let rval: [ExprEval; N] = self.list.iter().map(|v| v.evaluate(width, height, time, object).unwrap_or_else(|e| {
            errors.push(e);
            ExprEval::F64(0.0)
        })).collect::<Vec<ExprEval>>().try_into().unwrap();

        if errors.len() == 0 {
            Ok(rval)
        } else {
            anyhow::bail!("{:#?}",errors)
        }
    }
}
impl ExprVector<2> {
    /// Evaluates all expressions into a tuple of 2 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64, object: &HashMap<String, mlua::Value>) -> (anyhow::Result<ExprEval>, anyhow::Result<ExprEval>) {
        (self.list[0].evaluate(width, height, time, object),self.list[1].evaluate(width, height, time, object))
    }
}
impl ExprVector<3> {
    /// Evaluates all expressions into a tuple of 3 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64, object: &HashMap<String, mlua::Value>) -> (anyhow::Result<ExprEval>, anyhow::Result<ExprEval>, anyhow::Result<ExprEval>) {
        (
            self.list[0].evaluate(width, height, time, object),
            self.list[1].evaluate(width, height, time, object),
            self.list[2].evaluate(width, height, time, object)
        )
    }
}
impl ExprVector<4> {
    /// Evaluates all expressions into a tuple of 4 elements.
    pub fn evaluate_tuple(&self, width: f64, height: f64, time: f64, object: &HashMap<String, mlua::Value>) -> (anyhow::Result<ExprEval>, anyhow::Result<ExprEval>, anyhow::Result<ExprEval>, anyhow::Result<ExprEval>) {
        (
            self.list[0].evaluate(width, height, time, object),
            self.list[1].evaluate(width, height, time, object),
            self.list[2].evaluate(width, height, time, object),
            self.list[3].evaluate(width, height, time, object)
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
/// These expressions also support the percent-sign (`%`). It works like the percent sign in CSS.
/// It gets replaced with '/100*w' or '/100*h' when parsing the expression (which one it is depends
/// on the specified [`ResExprType`]).
/// 
/// Example: `50%` = `50/100*w` = `0.5*w` = half of the window's width
pub fn res_dependent_expr<S: Into<String>>(expr: S, context: Arc<Context<'static>>, expr_type: ResExprType) -> Result<ResolutionDependentExpr, PropertyError> {
    const EMPTY: String = String::new();

    let exprstr: String = expr.into();

    // Replace percent sign to be able to parse it with meval's parser.
    let mstring = exprstr.replace("%", &("/100*".to_owned()+expr_type.str()));
    let lstring = exprstr;

    use meval::{ Error, FuncEvalError, ParseError, RPNError };

    // Parse the expression and bind it to a function with three arguments
    // (the window's dimensions and time)
    let parsed_expr = mstring.clone().parse::<Expr>().map_err(|e| {
        let errdesc: String = match e {
            Error::ParseError(errtype) => {
                match errtype {
                    ParseError::MissingArgument => format!("Expression Parsing error: Missing argument at end of the expression!"),
                    ParseError::MissingRParen(n) => format!("Expression Parsing error: {n} missing right parentheses!"),
                    ParseError::UnexpectedToken(n) => {
                        let mut indicatorstr: String = String::with_capacity(n+6);
                        for _ in 0..n {
                            indicatorstr.push(' ');
                        }
                        indicatorstr.push_str("^ Here");
                        format!("Expression Parsing error: Unexpected token at position {n}:\n\t{}\n\t{indicatorstr}",mstring.clone())
                    }
                }
            },
            _ => format!("{e}")
        };
        PropertyError::SyntaxError(EMPTY.clone(), EMPTY.clone(), Some(errdesc))
    })?;
    let mut math_error = None;
    match parsed_expr.bind3_with_context(context.clone(), "w", "h", "t") {
        Ok(e) => { return Ok(ResolutionDependentExpr::MathExpr { expr: Arc::new(e), base_string: mstring, base_context: context, base_expr_type: expr_type }) },
        Err(err) => {
            let errdesc = match err {
                Error::Function(name, errtype) => match errtype {
                    FuncEvalError::NumberArgs(num) => format!("Invalid number of arguments for function '{name}()'! ({num} arguments supplied)"),
                    FuncEvalError::TooFewArguments => format!("Too few supplied arguments for function '{name}()'"),
                    FuncEvalError::TooManyArguments => format!("Too many supplied arguments for function '{name}()'"),
                    FuncEvalError::UnknownFunction => format!("Unknown function '{name}()'")
                },
                Error::ParseError(errtype) => match errtype {
                    ParseError::MissingArgument => format!("Missing operator or missing function argument at the end of the expression!"),
                    ParseError::MissingRParen(num) => match num { 1=> "1 missing right parenthesis!".to_owned(), _ => format!("{num} missing right parentheses!")},
                    ParseError::UnexpectedToken(pos) => format!("Unexpected token at position {pos}: \"...{}...\"", &mstring[(pos-2).max(0)..(pos+2).min(mstring.len()-1)]),
                },
                Error::RPNError(errtype) => match errtype {
                    RPNError::MismatchedLParen(pos) => format!("Unmatched left parenthesis at position {pos}: \"...{}...\"", &mstring[(pos-2).max(0)..(pos+2).min(mstring.len()-1)]),
                    RPNError::MismatchedRParen(pos) => format!("Unmatched right parenthesis at position {pos}: \"...{}...\"", &mstring[(pos-2).max(0)..(pos+2).min(mstring.len()-1)]),
                    RPNError::NotEnoughOperands(pos) => format!("Too few operands at position {pos}: \"...{}...\"", &mstring[(pos-2).max(0)..(pos+2).min(mstring.len()-1)]),
                    RPNError::TooManyOperands => format!("Too many operands!"),
                    RPNError::UnexpectedComma(pos) => format!("Unexpected comma at position {pos}: \"...{}...\"", &mstring[(pos-2).max(0)..(pos+2).min(mstring.len()-1)]),
                },
                Error::UnknownVariable(name) => format!("Unknown variable '{name}'!"),
                Error::EvalError(err) => format!("Evaluation error: {err}"),
            };
            math_error = Some(PropertyError::SyntaxError(EMPTY.clone(), EMPTY.clone(), Some(errdesc)));
        }
    }

    // If this part of the function is getting executed, we know the expression
    // isn't valid as a math expression, thus we'll try to parse it as a lua
    // snippet.

    let mut lua_error = None;

    match lua_expr(&lstring) {
        Ok(f) => { return Ok(f) },
        Err(e) => lua_error = Some(e)
    }

    // If neither the math parser's match arm nor the lua parser's match arm
    // returned a value, we know that whichever expression type the user chose
    // contains an invalid expression
    Err(PropertyError::MultiError(vec![math_error.unwrap(), lua_error.unwrap()]))
}

pub fn lua_expr<S: Into<String>>(expr: S) -> Result<ResolutionDependentExpr, PropertyError> {
    let str = expr.into();
    crate::LUA_INSTANCE.get().unwrap().load(&str).into_function()
        .map(|f| ResolutionDependentExpr::LuaExpr(f, str))
        .map_err(|e| PropertyError::LuaError(e))
}

/// Parses a list of expressions separated by semicolons using the [`res_dependent_expr()`] function.
pub fn parse_expression_list<'a, S: Into<String>>(string: S, context: Arc<Context<'static>>) -> Result<Vec<ResolutionDependentExpr>, PropertyError> {
    let mut expr_vec = Vec::new();

    for (i,expression) in <S as Into<String>>::into(string).split(";").enumerate() {
        expr_vec.push(res_dependent_expr(expression.to_owned(), context.clone(), match i%2 { 0 => ResExprType::WidthBased, _ => ResExprType::HeightBased })?);
    }

    Ok(expr_vec)
}