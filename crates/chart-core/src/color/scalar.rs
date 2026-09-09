//! Shared binary64 transport retains exceptional channels and signed zero.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub(super) fn serialize<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    crate::number::Number(*value).serialize(serializer)
}
pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    crate::number::Number::deserialize(deserializer).map(|number| number.0)
}
