use rocket::serde::json::Json;

mod or;
mod proto;
mod textproto;

pub use or::{Error as OrError, Or, WellKnownEncoding};
pub use proto::{Error as ProtoError, Proto};
pub use textproto::{Error as TextProtoError, TextProto};

pub type ProtoTextProtoJson<T> = Or<Proto<T>, Or<TextProto<T>, Json<T>, T>, T>;
