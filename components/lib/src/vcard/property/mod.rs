pub mod param;

mod datetime;
mod properties;
mod value;

use datetime::*;
pub use properties::*;
use value::*;

use crate::vcard::parser::{ParseContext, ParseError};

use crate::vcard::property::param::Param;

macro_rules! impl_try_from_property {
    ($prop_type:ident, $variant:ident) => {
        impl TryFromProperty for $prop_type {
            fn try_from_property(property: &Property) -> Option<&Self> {
                if let Property::$variant(p) = property {
                    Some(p)
                } else {
                    None
                }
            }
        }
    };
}

impl_try_from_property!(AdrProperty, Adr);
impl_try_from_property!(AnniversaryProperty, Anniversary);
impl_try_from_property!(BdayProperty, Bday);
impl_try_from_property!(BeginProperty, Begin);
impl_try_from_property!(CaladruriProperty, Caladruri);
impl_try_from_property!(CaluriProperty, Caluri);
impl_try_from_property!(CategoriesProperty, Categories);
impl_try_from_property!(ClientpidmapProperty, Clientpidmap);
impl_try_from_property!(EmailProperty, Email);
impl_try_from_property!(EndProperty, End);
impl_try_from_property!(FburlProperty, Fburl);
impl_try_from_property!(FnProperty, Fn);
impl_try_from_property!(GenderProperty, Gender);
impl_try_from_property!(GeoProperty, Geo);
impl_try_from_property!(ImppProperty, Impp);
impl_try_from_property!(KeyProperty, Key);
impl_try_from_property!(KindProperty, Kind);
impl_try_from_property!(LangProperty, Lang);
impl_try_from_property!(LogoProperty, Logo);
impl_try_from_property!(MemberProperty, Member);
impl_try_from_property!(NicknameProperty, Nickname);
impl_try_from_property!(NoteProperty, Note);
impl_try_from_property!(NProperty, N);
impl_try_from_property!(OrgProperty, Org);
impl_try_from_property!(PhotoProperty, Photo);
impl_try_from_property!(ProdidProperty, Prodid);
impl_try_from_property!(RelatedProperty, Related);
impl_try_from_property!(RevProperty, Rev);
impl_try_from_property!(RoleProperty, Role);
impl_try_from_property!(SoundProperty, Sound);
impl_try_from_property!(SourceProperty, Source);
impl_try_from_property!(TelProperty, Tel);
impl_try_from_property!(TitleProperty, Title);
impl_try_from_property!(TzProperty, Tz);
impl_try_from_property!(UidProperty, Uid);
impl_try_from_property!(UrlProperty, Url);
impl_try_from_property!(VersionProperty, Version);
impl_try_from_property!(XmlProperty, Xml);
impl_try_from_property!(GenericIanaProperty, IanaToken);
impl_try_from_property!(GenericXNameProperty, XName);

/// Common trait for all vCard properties
pub trait PropertyBase {
    /// Property-specific error type
    type Error: std::error::Error + 'static;

    /// Property name as defined in RFC 6350
    fn name(&self) -> Vec<u8>;

    /// Get the unescaped property value
    fn value(&self) -> Value;

    /// Get the property parameters
    fn params(&self) -> &[Param];

    /// Returns the value as it would appear in a vcard, escaped
    fn value_to_vcard_vec(&self) -> Vec<u8> {
        self.value()
            .fields()
            .iter()
            .map(|f: &Field| {
                f.values()
                    .iter()
                    .map(|v: &FieldValue| {
                        value::escape(
                            v.as_slice(),
                            v.escape_field_separator(),
                            v.escape_value_separator(),
                        )
                    })
                    .collect::<Vec<Vec<u8>>>()
                    .join(b",".as_slice())
            })
            .collect::<Vec<Vec<u8>>>()
            .join(b";".as_slice())
    }

    /// Returns the params as they would appear in a vcard, sometimes quotes
    fn param_to_vcard_vec(&self) -> Vec<u8> {
        if self.params().is_empty() {
            return vec![];
        }

        let mut out = b";".to_vec();

        out.extend(
            self.params()
                .iter()
                .map(|p| {
                    let mut out = vec![];
                    out.extend(p.name());
                    out.extend(b"=");
                    out.extend(
                        p.values()
                            .iter()
                            .map(|v| {
                                if param_value_needs_quoting(v) {
                                    let mut out = Vec::with_capacity(v.clone().len() + 2);
                                    out.push(b'"');
                                    out.extend(v.clone());
                                    out.push(b'"');
                                    out
                                } else {
                                    v.clone()
                                }
                            })
                            .collect::<Vec<Vec<u8>>>()
                            .join(b",".as_slice()),
                    );
                    out
                })
                .collect::<Vec<Vec<u8>>>()
                .join(b";".as_slice()),
        );

        out
    }

    fn to_vcard_vec(&self) -> Vec<u8> {
        let mut out = vec![];
        out.extend(self.name());
        out.extend(self.param_to_vcard_vec());
        out.extend(b":");
        out.extend(self.value_to_vcard_vec());
        out
    }
}

pub trait TryFromProperty: PropertyBase {
    fn try_from_property(property: &Property) -> Option<&Self>;
}

macro_rules! delegate_property_method {
    ($method_name:ident, $return_type:ty) => {
        pub fn $method_name(&self) -> $return_type {
            match self {
                Property::Adr(p) => p.$method_name(),
                Property::Anniversary(p) => p.$method_name(),
                Property::Bday(p) => p.$method_name(),
                Property::Begin(p) => p.$method_name(),
                Property::Caladruri(p) => p.$method_name(),
                Property::Caluri(p) => p.$method_name(),
                Property::Categories(p) => p.$method_name(),
                Property::Clientpidmap(p) => p.$method_name(),
                Property::Email(p) => p.$method_name(),
                Property::End(p) => p.$method_name(),
                Property::Fburl(p) => p.$method_name(),
                Property::Fn(p) => p.$method_name(),
                Property::Gender(p) => p.$method_name(),
                Property::Geo(p) => p.$method_name(),
                Property::Impp(p) => p.$method_name(),
                Property::Key(p) => p.$method_name(),
                Property::Kind(p) => p.$method_name(),
                Property::Lang(p) => p.$method_name(),
                Property::Logo(p) => p.$method_name(),
                Property::Member(p) => p.$method_name(),
                Property::Nickname(p) => p.$method_name(),
                Property::Note(p) => p.$method_name(),
                Property::N(p) => p.$method_name(),
                Property::Org(p) => p.$method_name(),
                Property::Photo(p) => p.$method_name(),
                Property::Prodid(p) => p.$method_name(),
                Property::Related(p) => p.$method_name(),
                Property::Rev(p) => p.$method_name(),
                Property::Role(p) => p.$method_name(),
                Property::Sound(p) => p.$method_name(),
                Property::Source(p) => p.$method_name(),
                Property::Tel(p) => p.$method_name(),
                Property::Title(p) => p.$method_name(),
                Property::Tz(p) => p.$method_name(),
                Property::Uid(p) => p.$method_name(),
                Property::Url(p) => p.$method_name(),
                Property::Version(p) => p.$method_name(),
                Property::Xml(p) => p.$method_name(),
                Property::IanaToken(p) => p.$method_name(),
                Property::XName(p) => p.$method_name(),
            }
        }
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Property {
    Adr(AdrProperty),
    Anniversary(AnniversaryProperty),
    Bday(BdayProperty),
    Begin(BeginProperty),
    Caladruri(CaladruriProperty),
    Caluri(CaluriProperty),
    Categories(CategoriesProperty),
    Clientpidmap(ClientpidmapProperty),
    Email(EmailProperty),
    End(EndProperty),
    Fburl(FburlProperty),
    Fn(FnProperty),
    Gender(GenderProperty),
    Geo(GeoProperty),
    Impp(ImppProperty),
    Key(KeyProperty),
    Kind(KindProperty),
    Lang(LangProperty),
    Logo(LogoProperty),
    Member(MemberProperty),
    Nickname(NicknameProperty),
    Note(NoteProperty),
    N(NProperty),
    Org(OrgProperty),
    Photo(PhotoProperty),
    Prodid(ProdidProperty),
    Related(RelatedProperty),
    Rev(RevProperty),
    Role(RoleProperty),
    Sound(SoundProperty),
    Source(SourceProperty),
    Tel(TelProperty),
    Title(TitleProperty),
    Tz(TzProperty),
    Uid(UidProperty),
    Url(UrlProperty),
    Version(VersionProperty),
    Xml(XmlProperty),
    IanaToken(GenericIanaProperty),
    XName(GenericXNameProperty),
}

impl Property {
    pub fn new(
        name: &[u8],
        params: Vec<Param>,
        value: Vec<u8>,
        ctx: ParseContext,
    ) -> Result<Self, ParseError> {
        match name.to_ascii_uppercase().as_slice() {
            b"ADR" => AdrProperty::parse(value, params, ctx)
                .map(Property::Adr)
                .map_err(ParseError::InvalidAdr),
            b"ANNIVERSARY" => AnniversaryProperty::parse(value, params, ctx)
                .map(Property::Anniversary)
                .map_err(ParseError::InvalidAnniversary),
            b"BDAY" => BdayProperty::parse(value, params, ctx)
                .map(Property::Bday)
                .map_err(ParseError::InvalidBday),
            b"BEGIN" => BeginProperty::parse(value, params, ctx)
                .map(Property::Begin)
                .map_err(ParseError::InvalidBegin),
            b"CALADRURI" => CaladruriProperty::parse(value, params, ctx)
                .map(Property::Caladruri)
                .map_err(ParseError::InvalidCaladruri),
            b"CALURI" => CaluriProperty::parse(value, params, ctx)
                .map(Property::Caluri)
                .map_err(ParseError::InvalidCaluri),
            b"CATEGORIES" => CategoriesProperty::parse(value, params, ctx)
                .map(Property::Categories)
                .map_err(ParseError::InvalidCategories),
            b"CLIENTPIDMAP" => ClientpidmapProperty::parse(value, params, ctx)
                .map(Property::Clientpidmap)
                .map_err(ParseError::InvalidClientpidmap),
            b"EMAIL" => EmailProperty::parse(value, params, ctx)
                .map(Property::Email)
                .map_err(ParseError::InvalidEmail),
            b"END" => EndProperty::parse(value, params, ctx)
                .map(Property::End)
                .map_err(ParseError::InvalidEnd),
            b"FBURL" => FburlProperty::parse(value, params, ctx)
                .map(Property::Fburl)
                .map_err(ParseError::InvalidFburl),
            b"FN" => FnProperty::parse(value, params, ctx)
                .map(Property::Fn)
                .map_err(ParseError::InvalidFn),
            b"GENDER" => GenderProperty::parse(value, params, ctx)
                .map(Property::Gender)
                .map_err(ParseError::InvalidGender),
            b"GEO" => GeoProperty::parse(value, params, ctx)
                .map(Property::Geo)
                .map_err(ParseError::InvalidGeo),
            b"IMPP" => ImppProperty::parse(value, params, ctx)
                .map(Property::Impp)
                .map_err(ParseError::InvalidImpp),
            b"KEY" => KeyProperty::parse(value, params, ctx)
                .map(Property::Key)
                .map_err(ParseError::InvalidKey),
            b"KIND" => KindProperty::parse(value, params, ctx)
                .map(Property::Kind)
                .map_err(ParseError::InvalidKind),
            b"LANG" => LangProperty::parse(value, params, ctx)
                .map(Property::Lang)
                .map_err(ParseError::InvalidLang),
            b"LOGO" => LogoProperty::parse(value, params, ctx)
                .map(Property::Logo)
                .map_err(ParseError::InvalidLogo),
            b"MEMBER" => MemberProperty::parse(value, params, ctx)
                .map(Property::Member)
                .map_err(ParseError::InvalidMember),
            b"NICKNAME" => NicknameProperty::parse(value, params, ctx)
                .map(Property::Nickname)
                .map_err(ParseError::InvalidNickname),
            b"NOTE" => NoteProperty::parse(value, params, ctx)
                .map(Property::Note)
                .map_err(ParseError::InvalidNote),
            b"N" => NProperty::parse(value, params, ctx)
                .map(Property::N)
                .map_err(ParseError::InvalidN),
            b"ORG" => OrgProperty::parse(value, params, ctx)
                .map(Property::Org)
                .map_err(ParseError::InvalidOrg),
            b"PHOTO" => PhotoProperty::parse(value, params, ctx)
                .map(Property::Photo)
                .map_err(ParseError::InvalidPhoto),
            b"PRODID" => ProdidProperty::parse(value, params, ctx)
                .map(Property::Prodid)
                .map_err(ParseError::InvalidProdid),
            b"RELATED" => RelatedProperty::parse(value, params, ctx)
                .map(Property::Related)
                .map_err(ParseError::InvalidRelated),
            b"REV" => RevProperty::parse(value, params, ctx)
                .map(Property::Rev)
                .map_err(ParseError::InvalidRev),
            b"ROLE" => RoleProperty::parse(value, params, ctx)
                .map(Property::Role)
                .map_err(ParseError::InvalidRole),
            b"SOUND" => SoundProperty::parse(value, params, ctx)
                .map(Property::Sound)
                .map_err(ParseError::InvalidSound),
            b"SOURCE" => SourceProperty::parse(value, params, ctx)
                .map(Property::Source)
                .map_err(ParseError::InvalidSource),
            b"TEL" => TelProperty::parse(value, params, ctx)
                .map(Property::Tel)
                .map_err(ParseError::InvalidTel),
            b"TITLE" => TitleProperty::parse(value, params, ctx)
                .map(Property::Title)
                .map_err(ParseError::InvalidTitle),
            b"TZ" => TzProperty::parse(value, params, ctx)
                .map(Property::Tz)
                .map_err(ParseError::InvalidTz),
            b"UID" => UidProperty::parse(value, params, ctx)
                .map(Property::Uid)
                .map_err(ParseError::InvalidUid),
            b"URL" => UrlProperty::parse(value, params, ctx)
                .map(Property::Url)
                .map_err(ParseError::InvalidUrl),
            b"VERSION" => VersionProperty::parse(value, params, ctx)
                .map(Property::Version)
                .map_err(ParseError::InvalidVersion),
            b"XML" => XmlProperty::parse(value, params, ctx)
                .map(Property::Xml)
                .map_err(ParseError::InvalidXml),
            _ => {
                let name_up = name.to_ascii_uppercase();
                if name_up.starts_with(b"X-") {
                    GenericXNameProperty::parse(name_up, value, params, ctx)
                        .map(Property::XName)
                        .map_err(ParseError::InvalidXName)
                } else {
                    GenericIanaProperty::parse(name_up, value, params, ctx)
                        .map(Property::IanaToken)
                        .map_err(ParseError::InvalidIana)
                }
            }
        }
    }

    delegate_property_method!(name, Vec<u8>);
    delegate_property_method!(value, Value);
    delegate_property_method!(to_vcard_vec, Vec<u8>);
    delegate_property_method!(value_to_vcard_vec, Vec<u8>);
    delegate_property_method!(param_to_vcard_vec, Vec<u8>);
    delegate_property_method!(params, &[Param]);
}

macro_rules! v {
    ($($field:expr),* $(,)?) => {
        $crate::vcard::property::Value::new(vec![$($field),*])
    };
}
pub(crate) use v;

macro_rules! f {
    ($($val:expr),* $(,)?) => {
        $crate::vcard::property::Field::new(vec![$($val),*])
    };
}
pub(crate) use f;

macro_rules! fv {
    ($val:expr) => {
        $crate::vcard::property::FieldValue::from($val)
    };
}
pub(crate) use fv;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    content: Vec<Field>,
}

impl Value {
    pub fn new(content: Vec<Field>) -> Value {
        Value { content }
    }

    pub fn fields(&self) -> &[Field] {
        &self.content
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        v!(f!(fv!(value)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    content: Vec<u8>,
    escape_field_separators: bool,
    escape_value_separators: bool,
}

impl FieldValue {
    pub fn new(content: Vec<u8>) -> FieldValue {
        FieldValue {
            content,
            escape_field_separators: true,
            escape_value_separators: true,
        }
    }
    pub fn raw(content: Vec<u8>) -> FieldValue {
        FieldValue {
            content,
            escape_field_separators: false,
            escape_value_separators: false,
        }
    }
    pub fn escape_field_separator(&self) -> bool {
        self.escape_field_separators
    }
    pub fn escape_value_separator(&self) -> bool {
        self.escape_value_separators
    }
    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.content
    }
}

impl From<Vec<u8>> for FieldValue {
    fn from(value: Vec<u8>) -> Self {
        FieldValue::new(value)
    }
}

impl From<&[u8]> for FieldValue {
    fn from(value: &[u8]) -> Self {
        FieldValue::new(value.to_vec())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        FieldValue::new(value.into_bytes())
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        FieldValue::new(value.as_bytes().to_vec())
    }
}

impl From<u8> for FieldValue {
    fn from(value: u8) -> Self {
        if value <= 127 {
            FieldValue::new(vec![value])
        } else {
            FieldValue::new(vec![b'?'])
        }
    }
}

impl From<&Vec<u8>> for FieldValue {
    fn from(value: &Vec<u8>) -> Self {
        FieldValue::new(value.clone())
    }
}

impl From<char> for FieldValue {
    fn from(value: char) -> Self {
        FieldValue::new(value.to_string().into_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    values: Vec<FieldValue>,
}

impl Field {
    pub fn new(values: Vec<FieldValue>) -> Field {
        Field { values }
    }
    pub fn values(&self) -> &[FieldValue] {
        self.values.as_slice()
    }
}

#[allow(unused)]
pub(crate) const MIN_PREF: u8 = 1;
pub(crate) const MAX_PREF: u8 = 100;

pub trait PropertyPref: PropertyBase {
    /// Get the preference value of the property
    fn pref(&self) -> Option<u8> {
        self.params()
            .iter()
            .find(|p| p.name() == b"PREF")
            .and_then(|p| p.first_value())
            .and_then(|v| String::from_utf8_lossy(&v).parse().ok())
    }
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum PropertyValueParseError {
    #[error("value fields must contain exactly one value, found {0}")]
    InvalidNumberFieldValues(usize),
    #[error("value must have one field, found {0}")]
    InvalidNumberFields(usize),
    #[error("unescape failed because of invalid input")]
    InvalidEscapeCharacters,
}
