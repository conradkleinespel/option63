use crate::parser_internals::result::ParserError;

pub trait ParamTrait: Sized {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError>;
}

mod altid;
mod calscale;
mod geo;
mod label;
mod language;
mod mediatype;
mod other;
mod pid;
mod pref;
mod sort_as;
mod type_param;
mod tz;
mod value;

pub use altid::AltidParam;
pub use calscale::CalscaleParam;
pub use geo::GeoParam;
pub use label::LabelParam;
pub use language::LanguageParam;
pub use mediatype::MediatypeParam;
pub use other::OtherParam;
pub use pid::{PidParam, PidParamValue};
pub use pref::PrefParam;
pub use sort_as::SortAsParam;
pub use type_param::{TypeParam, TypeParamRelated, TypeParamTel, TypeParamValue};
pub use tz::TzParam;
pub use value::ValueParam;
