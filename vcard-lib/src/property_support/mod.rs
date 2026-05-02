use crate::parser::ParserError;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Value;
use crate::parser_internals::property_value::escape;
use crate::property::AdrProperty;
use crate::property::AnniversaryProperty;
use crate::property::BdayProperty;
use crate::property::BeginProperty;
use crate::property::CaladruriProperty;
use crate::property::CaluriProperty;
use crate::property::CategoriesProperty;
use crate::property::ClientpidmapProperty;
use crate::property::EmailProperty;
use crate::property::EndProperty;
use crate::property::FburlProperty;
use crate::property::FnProperty;
use crate::property::GenderProperty;
use crate::property::GenericIanaProperty;
use crate::property::GenericXNameProperty;
use crate::property::GeoProperty;
use crate::property::ImppProperty;
use crate::property::KeyProperty;
use crate::property::KindProperty;
use crate::property::LangProperty;
use crate::property::LogoProperty;
use crate::property::MemberProperty;
use crate::property::NProperty;
use crate::property::NicknameProperty;
use crate::property::NoteProperty;
use crate::property::OrgProperty;
use crate::property::PhotoProperty;
use crate::property::ProdidProperty;
use crate::property::RelatedProperty;
use crate::property::RevProperty;
use crate::property::RoleProperty;
use crate::property::SoundProperty;
use crate::property::SourceProperty;
use crate::property::TelProperty;
use crate::property::TitleProperty;
use crate::property::TzProperty;
use crate::property::UidProperty;
use crate::property::UrlProperty;
use crate::property::VersionProperty;
use crate::property::XmlProperty;
use crate::property_support::param_quoting::param_value_needs_quoting;
use crate::{Field, FieldValue, Param};

pub(crate) mod date_and_or_time;
mod param_quoting;
pub(crate) mod pref;

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
                        escape(
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

impl Property {
    pub fn new(
        name: &[u8],
        params: Vec<Param>,
        value: Vec<u8>,
        ctx: ParseContext,
    ) -> Result<Self, ParserError> {
        match name.to_ascii_uppercase().as_slice() {
            b"ADR" => AdrProperty::parse(value, params, ctx)
                .map(Property::Adr)
                .map_err(ParserError::InvalidAdr),
            b"ANNIVERSARY" => AnniversaryProperty::parse(value, params, ctx)
                .map(Property::Anniversary)
                .map_err(ParserError::InvalidAnniversary),
            b"BDAY" => BdayProperty::parse(value, params, ctx)
                .map(Property::Bday)
                .map_err(ParserError::InvalidBday),
            b"BEGIN" => BeginProperty::parse(value, params, ctx)
                .map(Property::Begin)
                .map_err(ParserError::InvalidBegin),
            b"CALADRURI" => CaladruriProperty::parse(value, params, ctx)
                .map(Property::Caladruri)
                .map_err(ParserError::InvalidCaladruri),
            b"CALURI" => CaluriProperty::parse(value, params, ctx)
                .map(Property::Caluri)
                .map_err(ParserError::InvalidCaluri),
            b"CATEGORIES" => CategoriesProperty::parse(value, params, ctx)
                .map(Property::Categories)
                .map_err(ParserError::InvalidCategories),
            b"CLIENTPIDMAP" => ClientpidmapProperty::parse(value, params, ctx)
                .map(Property::Clientpidmap)
                .map_err(ParserError::InvalidClientpidmap),
            b"EMAIL" => EmailProperty::parse(value, params, ctx)
                .map(Property::Email)
                .map_err(ParserError::InvalidEmail),
            b"END" => EndProperty::parse(value, params, ctx)
                .map(Property::End)
                .map_err(ParserError::InvalidEnd),
            b"FBURL" => FburlProperty::parse(value, params, ctx)
                .map(Property::Fburl)
                .map_err(ParserError::InvalidFburl),
            b"FN" => FnProperty::parse(value, params, ctx)
                .map(Property::Fn)
                .map_err(ParserError::InvalidFn),
            b"GENDER" => GenderProperty::parse(value, params, ctx)
                .map(Property::Gender)
                .map_err(ParserError::InvalidGender),
            b"GEO" => GeoProperty::parse(value, params, ctx)
                .map(Property::Geo)
                .map_err(ParserError::InvalidGeo),
            b"IMPP" => ImppProperty::parse(value, params, ctx)
                .map(Property::Impp)
                .map_err(ParserError::InvalidImpp),
            b"KEY" => KeyProperty::parse(value, params, ctx)
                .map(Property::Key)
                .map_err(ParserError::InvalidKey),
            b"KIND" => KindProperty::parse(value, params, ctx)
                .map(Property::Kind)
                .map_err(ParserError::InvalidKind),
            b"LANG" => LangProperty::parse(value, params, ctx)
                .map(Property::Lang)
                .map_err(ParserError::InvalidLang),
            b"LOGO" => LogoProperty::parse(value, params, ctx)
                .map(Property::Logo)
                .map_err(ParserError::InvalidLogo),
            b"MEMBER" => MemberProperty::parse(value, params, ctx)
                .map(Property::Member)
                .map_err(ParserError::InvalidMember),
            b"NICKNAME" => NicknameProperty::parse(value, params, ctx)
                .map(Property::Nickname)
                .map_err(ParserError::InvalidNickname),
            b"NOTE" => NoteProperty::parse(value, params, ctx)
                .map(Property::Note)
                .map_err(ParserError::InvalidNote),
            b"N" => NProperty::parse(value, params, ctx)
                .map(Property::N)
                .map_err(ParserError::InvalidN),
            b"ORG" => OrgProperty::parse(value, params, ctx)
                .map(Property::Org)
                .map_err(ParserError::InvalidOrg),
            b"PHOTO" => PhotoProperty::parse(value, params, ctx)
                .map(Property::Photo)
                .map_err(ParserError::InvalidPhoto),
            b"PRODID" => ProdidProperty::parse(value, params, ctx)
                .map(Property::Prodid)
                .map_err(ParserError::InvalidProdid),
            b"RELATED" => RelatedProperty::parse(value, params, ctx)
                .map(Property::Related)
                .map_err(ParserError::InvalidRelated),
            b"REV" => RevProperty::parse(value, params, ctx)
                .map(Property::Rev)
                .map_err(ParserError::InvalidRev),
            b"ROLE" => RoleProperty::parse(value, params, ctx)
                .map(Property::Role)
                .map_err(ParserError::InvalidRole),
            b"SOUND" => SoundProperty::parse(value, params, ctx)
                .map(Property::Sound)
                .map_err(ParserError::InvalidSound),
            b"SOURCE" => SourceProperty::parse(value, params, ctx)
                .map(Property::Source)
                .map_err(ParserError::InvalidSource),
            b"TEL" => TelProperty::parse(value, params, ctx)
                .map(Property::Tel)
                .map_err(ParserError::InvalidTel),
            b"TITLE" => TitleProperty::parse(value, params, ctx)
                .map(Property::Title)
                .map_err(ParserError::InvalidTitle),
            b"TZ" => TzProperty::parse(value, params, ctx)
                .map(Property::Tz)
                .map_err(ParserError::InvalidTz),
            b"UID" => UidProperty::parse(value, params, ctx)
                .map(Property::Uid)
                .map_err(ParserError::InvalidUid),
            b"URL" => UrlProperty::parse(value, params, ctx)
                .map(Property::Url)
                .map_err(ParserError::InvalidUrl),
            b"VERSION" => VersionProperty::parse(value, params, ctx)
                .map(Property::Version)
                .map_err(ParserError::InvalidVersion),
            b"XML" => XmlProperty::parse(value, params, ctx)
                .map(Property::Xml)
                .map_err(ParserError::InvalidXml),
            _ => {
                let name_up = name.to_ascii_uppercase();
                if name_up.starts_with(b"X-") {
                    GenericXNameProperty::parse(name_up, value, params, ctx)
                        .map(Property::XName)
                        .map_err(ParserError::InvalidXName)
                } else {
                    GenericIanaProperty::parse(name_up, value, params, ctx)
                        .map(Property::IanaToken)
                        .map_err(ParserError::InvalidIana)
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

pub fn is_valid_property_name(field: &str) -> bool {
    let field_upper = field.to_ascii_uppercase();
    let bytes = field_upper.as_bytes();

    if bytes.starts_with(b"X-") {
        return true;
    }

    // TODO: Need to support IANA properties (generic IANA tokens)

    matches!(
        field_upper.as_str(),
        "ADR"
            | "ANNIVERSARY"
            | "BDAY"
            | "BEGIN"
            | "CALADRURI"
            | "CALURI"
            | "CATEGORIES"
            | "CLIENTPIDMAP"
            | "EMAIL"
            | "END"
            | "FBURL"
            | "FN"
            | "GENDER"
            | "GEO"
            | "IMPP"
            | "KEY"
            | "KIND"
            | "LANG"
            | "LOGO"
            | "MEMBER"
            | "NICKNAME"
            | "NOTE"
            | "N"
            | "ORG"
            | "PHOTO"
            | "PRODID"
            | "RELATED"
            | "REV"
            | "ROLE"
            | "SOUND"
            | "SOURCE"
            | "TEL"
            | "TITLE"
            | "TZ"
            | "UID"
            | "URL"
            | "VERSION"
            | "XML"
    )
}
