use crate::vcard::Version;

/// Context for parsing vCard properties
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseContext {
    pub version: Version,
    pub strict: bool,
}

impl ParseContext {
    pub fn new(version: Version, strict: bool) -> Self {
        ParseContext { version, strict }
    }

    pub fn v40_lax() -> Self {
        ParseContext {
            version: Version::V40,
            strict: false,
        }
    }

    pub fn v30_lax() -> Self {
        ParseContext {
            version: Version::V30,
            strict: false,
        }
    }

    pub fn v21_lax() -> Self {
        ParseContext {
            version: Version::V21,
            strict: false,
        }
    }
}

/// A nom parse result over byte slices, using this crate's [`ParseError`].
pub(crate) type R<'a, T> = nom::IResult<&'a [u8], T, ParseError>;
pub(crate) type RE<'a, T, E> = nom::IResult<&'a [u8], T, E>;

/// Errors during parsing.
#[non_exhaustive]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    /// Generic error.
    #[error("generic parser error")]
    Generic,
    /// Property name error.
    #[error("invalid property name")]
    PropertyName,
    /// Parameter name error.
    #[error("invalid parameter name")]
    ParamName,
    /// Equals sign error in parameter.
    #[error("invalid parameter equals sign")]
    ParamEquals,
    /// Parameter value error.
    #[error("invalid parameter value")]
    ParamValue,
    /// Colon error.
    #[error("missing or invalid colon")]
    Colon,
    /// Property value error.
    #[error("invalid property value")]
    PropertyValue,
    /// Invalid UTF-8 error.
    #[error("invalid UTF-8 sequence")]
    InvalidUtf8,
    /// Missing BEGIN.
    #[error("missing BEGIN property")]
    MissingBegin,
    /// Missing END.
    #[error("missing END property")]
    MissingEnd,
    /// Missing VERSION.
    #[error("missing VERSION property")]
    MissingVersion,
    /// Invalid gender value.
    #[error("invalid gender: {0}")]
    InvalidGender(#[from] crate::vcard::property::GenderError),
    /// Invalid ADR value.
    #[error("invalid adr: {0}")]
    InvalidAdr(#[from] crate::vcard::property::AdrError),
    /// Invalid email value.
    #[error("invalid email: {0}")]
    InvalidEmail(#[from] crate::vcard::property::EmailError),
    /// Invalid anniversary value.
    #[error("invalid anniversary: {0}")]
    InvalidAnniversary(#[from] crate::vcard::property::AnniversaryError),
    /// Invalid BDAY value.
    #[error("invalid bday: {0}")]
    InvalidBday(#[from] crate::vcard::property::BdayError),
    /// Invalid BEGIN value.
    #[error("invalid begin: {0}")]
    InvalidBegin(#[from] crate::vcard::property::BeginError),
    /// Invalid END value.
    #[error("invalid end: {0}")]
    InvalidEnd(#[from] crate::vcard::property::EndError),
    /// Invalid CALADRURI value.
    #[error("invalid caladruri: {0}")]
    InvalidCaladruri(#[from] crate::vcard::property::CaladruriError),
    /// Invalid CALURI value.
    #[error("invalid caluri: {0}")]
    InvalidCaluri(#[from] crate::vcard::property::CaluriError),
    /// Invalid CATEGORIES value.
    #[error("invalid categories: {0}")]
    InvalidCategories(#[from] crate::vcard::property::CategoriesError),
    /// Invalid CLIENTPIDMAP value.
    #[error("invalid clientpidmap: {0}")]
    InvalidClientpidmap(#[from] crate::vcard::property::ClientpidmapError),
    /// Invalid FBURL value.
    #[error("invalid fburl: {0}")]
    InvalidFburl(#[from] crate::vcard::property::FburlError),
    /// Invalid pub(crate) fn value.
    #[error("invalid fn: {0}")]
    InvalidFn(#[from] crate::vcard::property::FnError),
    /// Invalid GEO value.
    #[error("invalid geo: {0}")]
    InvalidGeo(#[from] crate::vcard::property::GeoError),
    /// Invalid IMPP value.
    #[error("invalid impp: {0}")]
    InvalidImpp(#[from] crate::vcard::property::ImppError),
    /// Invalid KEY value.
    #[error("invalid key: {0}")]
    InvalidKey(#[from] crate::vcard::property::KeyError),
    /// Invalid KIND value.
    #[error("invalid kind: {0}")]
    InvalidKind(#[from] crate::vcard::property::KindError),
    /// Invalid LANG value.
    #[error("invalid lang: {0}")]
    InvalidLang(#[from] crate::vcard::property::LangError),
    /// Invalid LOGO value.
    #[error("invalid logo: {0}")]
    InvalidLogo(#[from] crate::vcard::property::LogoError),
    /// Invalid MEMBER value.
    #[error("invalid member: {0}")]
    InvalidMember(#[from] crate::vcard::property::MemberError),
    /// Invalid NICKNAME value.
    #[error("invalid nickname: {0}")]
    InvalidNickname(#[from] crate::vcard::property::NicknameError),
    /// Invalid NOTE value.
    #[error("invalid note: {0}")]
    InvalidNote(#[from] crate::vcard::property::NoteError),
    /// Invalid N value.
    #[error("invalid n: {0}")]
    InvalidN(#[from] crate::vcard::property::NError),
    /// Invalid ORG value.
    #[error("invalid org: {0}")]
    InvalidOrg(#[from] crate::vcard::property::OrgError),
    /// Invalid PHOTO value.
    #[error("invalid photo: {0}")]
    InvalidPhoto(#[from] crate::vcard::property::PhotoError),
    /// Invalid PRODID value.
    #[error("invalid prodid: {0}")]
    InvalidProdid(#[from] crate::vcard::property::ProdidError),
    /// Invalid RELATED value.
    #[error("invalid related: {0}")]
    InvalidRelated(#[from] crate::vcard::property::RelatedError),
    /// Invalid REV value.
    #[error("invalid rev: {0}")]
    InvalidRev(#[from] crate::vcard::property::RevError),
    /// Invalid ROLE value.
    #[error("invalid role: {0}")]
    InvalidRole(#[from] crate::vcard::property::RoleError),
    /// Invalid SOUND value.
    #[error("invalid sound: {0}")]
    InvalidSound(#[from] crate::vcard::property::SoundError),
    /// Invalid SOURCE value.
    #[error("invalid source: {0}")]
    InvalidSource(#[from] crate::vcard::property::SourceError),
    /// Invalid TEL value.
    #[error("invalid tel: {0}")]
    InvalidTel(#[from] crate::vcard::property::TelError),
    /// Invalid TITLE value.
    #[error("invalid title: {0}")]
    InvalidTitle(#[from] crate::vcard::property::TitleError),
    /// Invalid TZ value.
    #[error("invalid tz: {0}")]
    InvalidTz(#[from] crate::vcard::property::TzError),
    /// Invalid UID value.
    #[error("invalid uid: {0}")]
    InvalidUid(#[from] crate::vcard::property::UidError),
    /// Invalid URL value.
    #[error("invalid url: {0}")]
    InvalidUrl(#[from] crate::vcard::property::UrlError),
    /// Invalid XML value.
    #[error("invalid xml: {0}")]
    InvalidXml(#[from] crate::vcard::property::XmlError),
    /// Invalid Iana value.
    #[error("invalid iana: {0}")]
    InvalidIana(#[from] crate::vcard::property::GenericIanaError),
    /// Invalid XName value.
    #[error("invalid x-name: {0}")]
    InvalidXName(#[from] crate::vcard::property::GenericXNameError),
    /// Invalid VERSION value.
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] crate::vcard::property::VersionError),
    /// Multiple VERSION properties found.
    #[error("multiple VERSION properties found")]
    MultipleVersion,
    /// VERSION property not at second line for vCard 4.0.
    #[error("VERSION property not at second line")]
    VersionNotSecondLine,
}

impl<'a> nom::error::ParseError<&'a [u8]> for ParseError {
    fn from_error_kind(_input: &'a [u8], _kind: nom::error::ErrorKind) -> Self {
        ParseError::Generic
    }

    fn append(_input: &'a [u8], _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> nom::error::FromExternalError<&'a [u8], ParseError> for ParseError {
    fn from_external_error(_input: &'a [u8], _kind: nom::error::ErrorKind, e: ParseError) -> Self {
        e
    }
}
