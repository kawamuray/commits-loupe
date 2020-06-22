/// Create an instance of JS `Closure<T>`, store it into vec, and return the reference to it.
macro_rules! closure {
    ($store:expr, $type:ty, $fn:expr) => {{
        let closure = Closure::wrap(Box::new($fn) as Box<$type>);
        $store.push(Box::new(closure) as Box<dyn Drop>);
        unsafe { &**($store.last().unwrap() as *const Box<dyn Drop> as *const Box<Closure<$type>>) }
    }};
}

macro_rules! js_arr {
    { $($val:expr),* $(,)? } => {
        {
            let array = Array::new();
            $(
                array.push($val);
            )*
                array
        }
    }
}

macro_rules! js_obj {
    { $($name:ident => $val:expr),* $(,)? } => {{
        let obj = Object::new();
        $(
            Reflect::set(&obj, &JsValue::from_str(stringify!($name)), $val).expect(concat!("error setting js attribute: ", stringify!($name)));
        )*
            obj
    }};
}

macro_rules! js_ref {
    ($val:expr) => {
        &JsValue::from($val)
    };
}
