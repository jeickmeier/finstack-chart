use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
macro_rules! wire_identity {
    ($ty:ty) => {
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.get().to_string())
            }
        }
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                crate::portable::unsigned::deserialize(d).map(Self::new)
            }
        }
    };
}
pub(crate) use wire_identity;
macro_rules! integer {
    ($module:ident, $ty:ty) => {
        pub(crate) mod $module {
            use super::*;
            pub fn serialize<S: Serializer>(v: &$ty, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&v.to_string())
            }
            pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<$ty, D::Error> {
                let text = String::deserialize(d)?;
                let n = text.parse::<$ty>().map_err(D::Error::custom)?;
                if n.to_string() != text {
                    return Err(D::Error::custom("Use a canonical decimal integer string"));
                }
                Ok(n)
            }
        }
    };
}
integer!(signed, i64);
integer!(unsigned, u64);
macro_rules! integers {
    ($module:ident, $ty:ty, $single:literal) => {
        pub(crate) mod $module {
            use super::*;
            #[derive(Serialize, Deserialize)]
            struct Number(#[serde(with = $single)] $ty);
            pub fn serialize<S: Serializer>(v: &[$ty], s: S) -> Result<S::Ok, S::Error> {
                v.iter()
                    .map(|n| Number(*n))
                    .collect::<Vec<_>>()
                    .serialize(s)
            }
            pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<$ty>, D::Error> {
                Vec::<Number>::deserialize(d).map(|v| v.into_iter().map(|n| n.0).collect())
            }
        }
    };
}
integers!(signed_vec, i64, "super::signed");
integers!(unsigned_vec, u64, "super::unsigned");
pub(crate) mod floats {
    use super::*;
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum Number {
        Finite(f64),
        Special(String),
    }
    pub fn serialize<S: Serializer>(v: &[f64], s: S) -> Result<S::Ok, S::Error> {
        v.iter()
            .map(|n| {
                if n.is_finite() {
                    Number::Finite(*n)
                } else {
                    Number::Special(format!("bits:{:016x}", n.to_bits()))
                }
            })
            .collect::<Vec<_>>()
            .serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
        Vec::<Number>::deserialize(d)?.into_iter().map(|n|match n {
            Number::Finite(n) if n.is_finite()=>Ok(n),
            Number::Special(s)=> {
                let bits=s.strip_prefix("bits:").filter(|s|s.len()==16).and_then(|s|u64::from_str_radix(s,16).ok()).ok_or_else(||D::Error::custom("Use finite numbers or bits: plus 16 hex digits for nonfinite floats"))?;
                let n=f64::from_bits(bits);
                if n.is_finite() { return Err(D::Error::custom("Finite source floats must be JSON numbers")); }
                Ok(n)
            },
            _=>Err(D::Error::custom("Nonfinite JSON number")),
        }).collect()
    }
}
