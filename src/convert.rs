
use neon::types::{JsString, JsArray, JsBoolean, JsUndefined, JsNull, JsNumber, JsValue, JsObject, Value, JsFunction};
use neon::handle::{Handle, Root};
use neon::context::{Context, FunctionContext};
use neon::object::{Object};
use std::rc::Rc;
use std::cmp::Ordering;
use crate::parser::ExpressionKind;
use crate::compiler::CompilerContext;

use crate::error::{FinchError, FinchResult};

pub struct RawObject<T: Value>(Root<T>);

impl<T: Value> PartialEq<RawObject<T>> for RawObject<T> {

    fn eq(&self, _other: &RawObject<T>) -> bool {
        false
    }
}


#[derive(std::cmp::PartialEq)]
pub enum RawValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Vec(Rc<Vec<RawValue>>),
    Object(RawObject<JsObject>),
    Function(RawObject<JsFunction>),
    Undefined,
    Null
}

impl RawValue {
    
    pub fn clone(&self, cx: &mut FunctionContext) -> Self {
        match self {
            Self::String(string) => Self::String(string.clone()),
            Self::Number(num) => Self::Number(*num),
            Self::Vec(v) => Self::Vec(v.clone()),
            Self::Boolean(bol) => Self::Boolean(*bol),
            Self::Object(obj) => Self::Object(RawObject(obj.0.clone(cx))),
            Self::Function(func) => Self::Function(RawObject(func.0.clone(cx))),
            Self::Undefined => Self::Undefined,
            Self::Null => Self::Null
        }
    }

}

pub trait IntoRawValue<'a> {
    fn raw(&self, cx: &mut FunctionContext<'a>) -> RawValue;
}

impl<'a> IntoRawValue<'a> for Handle<'a, JsValue> {
    fn raw(&self, cx: &mut FunctionContext<'a>) -> RawValue {
        if let Ok(str_handle) = self.downcast::<JsString, _>(cx) {
            RawValue::String(str_handle.value(cx))
        } else if let Ok(num_handle) = self.downcast::<JsNumber, _>(cx) {
            RawValue::Number(num_handle.value(cx))
        } else if let Ok(bool_handle) = self.downcast::<JsBoolean, _>(cx) {
            RawValue::Boolean(bool_handle.value(cx))
        } else if self.is_a::<JsNull, _>(cx) {
            RawValue::Null
        } else if self.is_a::<JsUndefined, _>(cx) {
            RawValue::Undefined
        } else if let Ok(arr_handle) = self.downcast::<JsArray, _>(cx) {
            if let Ok(vec) = arr_handle.to_vec(cx) {
                RawValue::Vec(Rc::new(vec.into_iter().map(|i| i.raw(cx)).collect()))
            } else {
                RawValue::Undefined
            }
        } else if let Ok(obj_handle) = self.downcast::<JsObject, _>(cx) {
            RawValue::Object(RawObject(obj_handle.root(cx)))
        } else if let Ok(fn_handle) = self.downcast::<JsFunction, _>(cx) {
            RawValue::Function(RawObject(fn_handle.root(cx)))
        }
        else {
            RawValue::Undefined
        }
    }
}

impl RawValue {
    pub fn js<'a>(&self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue> {
        match self {
            RawValue::String(val) => cx.string(val).upcast::<JsValue>(),
            RawValue::Number(num) => cx.number(*num).upcast::<JsValue>(),
            RawValue::Boolean(b) => cx.boolean(*b).upcast::<JsValue>(),
            RawValue::Undefined => cx.undefined().upcast::<JsValue>(),
            RawValue::Null => cx.null().upcast::<JsValue>(),
            RawValue::Vec(v) => {
                let arr = JsArray::new(cx, v.len() as u32);
                for (ind, val) in v.iter().enumerate() {
                    let js_val = val.clone(cx).js(cx);
                    arr.set(cx, ind as u32, js_val).unwrap();
                };
                arr.upcast::<JsValue>()
             },
             RawValue::Object(obj) => obj.0.to_inner(cx).upcast::<JsValue>(),
             RawValue::Function(func) => func.0.to_inner(cx).upcast::<JsValue>()
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Self::String(str) => str.is_empty(),
            Self::Number(num) => *num == 0.0,
            Self::Boolean(bol) => !(*bol),
            Self::Null | Self::Undefined => true,
            Self::Vec(_) | Self::Object(_) | Self::Function(_) => false
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Self::String(st) => st,
            Self::Number(num) => num.to_string(),
            Self::Boolean(bol) => bol.to_string(),
            Self::Undefined => String::from("undefined"),
            Self::Null => String::from("null"),
            Self::Vec(v) => v.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", "),
            Self::Object(_) => String::from("[object Object]"),
            Self::Function(_) => String::from("[function]")
        }
    }

}

impl std::string::ToString for RawValue {

    fn to_string(&self) -> String {
        match self {
            Self::String(st) => st.clone(),
            Self::Number(num) => num.to_string(),
            Self::Boolean(bol) => bol.to_string(),
            Self::Undefined => String::from("undefined"),
            Self::Null => String::from("null"),
            Self::Vec(v) => v.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", "),
            Self::Object(_) => String::from("[object Object]"),
            Self::Function(_) => String::from("[function]")
        }
    }
}

pub fn compare_vals(left: &ExpressionKind, right: &ExpressionKind, ctx: &mut CompilerContext) -> FinchResult<Ordering> {
    let num_left = if let ExpressionKind::Number(num) = left {
        num.clone()
    } else {
        if let RawValue::Number(num) = left.compile(ctx)? {
            num
        } else {
            return Err(FinchError::NotNumbers);
        }
    };
    let num_right = if let ExpressionKind::Number(num) = right {
        num.clone()
    } else {
        if let RawValue::Number(num) = right.compile(ctx)? {
            num
        } else {
            return Err(FinchError::NotNumbers);
        }
    };
    Ok(if num_left > num_right { Ordering::Greater }
    else if num_left == num_right { Ordering::Equal }
    else { Ordering::Less })
}