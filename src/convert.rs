
use neon::types::{JsString, JsArray, JsBoolean, JsUndefined, JsNull, JsNumber, JsValue};
use neon::handle::Handle;
use neon::context::{Context, FunctionContext};
use neon::object::{Object};

#[derive(Clone)]
pub enum RawValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Vec(Vec<RawValue>),
    Undefined,
    ShortCircuit,
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
                RawValue::Vec(vec.into_iter().map(|i| i.raw(cx)).collect())
            } else {
                RawValue::Undefined
            }
        } else {
            RawValue::Undefined
        }
    }
}

impl RawValue {
    fn js<'a>(self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue> {
        match self {
            RawValue::String(val) => cx.string(val).upcast::<JsValue>(),
            RawValue::Number(num) => cx.number(num).upcast::<JsValue>(),
            RawValue::Boolean(b) => cx.boolean(b).upcast::<JsValue>(),
            RawValue::Undefined => cx.undefined().upcast::<JsValue>(),
            RawValue::Null => cx.null().upcast::<JsValue>(),
            RawValue::Vec(v) => {
                let arr = JsArray::new(cx, v.len() as u32);
                for (ind, val) in v.into_iter().enumerate() {
                    let js_val = val.js(cx);
                    arr.set(cx, ind as u32, js_val);
                };
                arr.upcast::<JsValue>()
             }
             RawValue::ShortCircuit => cx.undefined().upcast::<JsValue>()
        }
    }
}