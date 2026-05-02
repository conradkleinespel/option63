use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeParamTel {
    Text,
    Voice,
    Fax,
    Cell,
    Video,
    Pager,
    TextPhone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeParamRelated {
    Contact,
    Acquaintance,
    Friend,
    Met,
    CoWorker,
    Colleague,
    CoResident,
    Neighbor,
    Child,
    Parent,
    Sibling,
    Spouse,
    Kin,
    Muse,
    Crush,
    Date,
    Sweetheart,
    Me,
    Agent,
    Emergency,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParam {
    values: Vec<TypeParamValue>,
}

impl ParamTrait for TypeParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let mut type_values = Vec::new();
        for value in values {
            match value.to_ascii_uppercase().as_slice() {
                b"WORK" => type_values.push(TypeParamValue::Work),
                b"HOME" => type_values.push(TypeParamValue::Home),
                v if v.starts_with(b"X-") => type_values.push(TypeParamValue::XName(value.clone())),
                _ => type_values.push(TypeParamValue::IanaToken(value.clone())),
            }
        }
        Ok(TypeParam {
            values: type_values,
        })
    }
}

impl TypeParam {
    pub fn values(&self) -> &[TypeParamValue] {
        &self.values
    }

    pub fn new(values: Vec<TypeParamValue>) -> Self {
        TypeParam { values }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeParamValue {
    Work,
    Home,
    IanaToken(Vec<u8>),
    XName(Vec<u8>),
}
