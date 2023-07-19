use plist::Value;

#[must_use]
#[inline]
pub fn pv<'a>(path: &[String], mut p: &'a Value) -> &'a Value {
    for k in path {
        p = match p {
            Value::Dictionary(v) => &v[k],
            Value::Array(v) => &v[k.parse::<usize>().unwrap()],
            _ => unreachable!(),
        };
    }
    p
}

#[must_use]
#[inline]
pub fn pv_mut<'a>(path: &[String], mut p: &'a mut Value) -> &'a mut Value {
    for k in path {
        p = match p {
            Value::Dictionary(v) => &mut v[k],
            Value::Array(v) => &mut v[k.parse::<usize>().unwrap()],
            _ => unreachable!(),
        };
    }
    p
}

#[must_use]
#[inline]
pub fn child_keys(path: &[String], p: &Value) -> Vec<String> {
    match pv(path, p) {
        Value::Dictionary(v) => v.keys().cloned().collect(),
        Value::Array(v) => v.iter().enumerate().map(|(i, _)| i.to_string()).collect(),
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    String,
    Integer,
    Real,
    Boolean,
    Data,
    Array,
    Dictionary,
}

impl ValueType {
    #[must_use]
    #[inline]
    pub fn from_val(path: &[String], p: &Value) -> Self {
        match pv(path, p) {
            Value::String(_) => Self::String,
            Value::Integer(_) => Self::Integer,
            Value::Real(_) => Self::Real,
            Value::Boolean(_) => Self::Boolean,
            Value::Data(_) => Self::Data,
            Value::Array(_) => Self::Array,
            Value::Dictionary(_) => Self::Dictionary,
            _ => unimplemented!(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_expandable(self) -> bool {
        matches!(self, Self::Array | Self::Dictionary)
    }
}
