
use neon::types::{JsString, JsArray, JsBoolean, JsUndefined, JsNull, JsNumber, JsValue};
use neon::handle::Handle;
use neon::context::{Context, FunctionContext};
use neon::object::{Object};
use std::rc::Rc;

#[derive(Clone, std::cmp::PartialEq)]
pub enum RawValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Vec(Rc<Vec<RawValue>>),
    Undefined,
    Null
}

pub trait IntoRawValue {
    fn raw(&self, cx: &mut FunctionContext) -> RawValue;
}

impl<'a> IntoRawValue for Handle<'a, JsValue> {
    fn raw(&self, cx: &mut FunctionContext) -> RawValue {
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
        } else {
            RawValue::Undefined
        }
    }
}

impl RawValue {
    pub fn js<'a>(self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue> {
        match self {
            RawValue::String(val) => cx.string(val).upcast::<JsValue>(),
            RawValue::Number(num) => cx.number(num).upcast::<JsValue>(),
            RawValue::Boolean(b) => cx.boolean(b).upcast::<JsValue>(),
            RawValue::Undefined => cx.undefined().upcast::<JsValue>(),
            RawValue::Null => cx.null().upcast::<JsValue>(),
            RawValue::Vec(v) => {
                let arr = JsArray::new(cx, v.len() as u32);
                for (ind, val) in v.iter().enumerate() {
                    let js_val = val.clone().js(cx);
                    arr.set(cx, ind as u32, js_val).unwrap();
                };
                arr.upcast::<JsValue>()
             }
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Self::String(str) => str.is_empty(),
            Self::Number(num) => *num == 0.0,
            Self::Boolean(bol) => !(*bol),
            Self::Null | Self::Undefined => true,
            Self::Vec(_) => false,
        }
    }

    pub fn convert_to_string(self) -> String {
        match self {
            Self::String(st) => st,
            Self::Number(num) => num.to_string(),
            Self::Boolean(bol) => bol.to_string(),
            Self::Undefined => String::from("undefined"),
            Self::Null => String::from("null"),
            Self::Vec(v) => v.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", ")
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
            Self::Vec(v) => v.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", ")
        }
    }
}