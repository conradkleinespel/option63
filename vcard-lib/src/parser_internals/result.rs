use crate::parser_internals::version::Version;

/// Errors during parsing.
#[non_exhaustive]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParserError {
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
    InvalidGender(#[from] crate::property::GenderError),
    /// Invalid ADR value.
    #[error("invalid adr: {0}")]
    InvalidAdr(#[from] crate::property::AdrError),
    /// Invalid email value.
    #[error("invalid email: {0}")]
    InvalidEmail(#[from] crate::property::EmailError),
    /// Invalid anniversary value.
    #[error("invalid anniversary: {0}")]
    InvalidAnniversary(#[from] crate::property::AnniversaryError),
    /// Invalid BDAY value.
    #[error("invalid bday: {0}")]
    InvalidBday(#[from] crate::property::BdayError),
    /// Invalid BEGIN value.
    #[error("invalid begin: {0}")]
    InvalidBegin(#[from] crate::property::BeginError),
    /// Invalid END value.
    #[error("invalid end: {0}")]
    InvalidEnd(#[from] crate::property::EndError),
    /// Invalid CALADRURI value.
    #[error("invalid caladruri: {0}")]
    InvalidCaladruri(#[from] crate::property::CaladruriError),
    /// Invalid CALURI value.
    #[error("invalid caluri: {0}")]
    InvalidCaluri(#[from] crate::property::CaluriError),
    /// Invalid CATEGORIES value.
    #[error("invalid categories: {0}")]
    InvalidCategories(#[from] crate::property::CategoriesError),
    /// Invalid CLIENTPIDMAP value.
    #[error("invalid clientpidmap: {0}")]
    InvalidClientpidmap(#[from] crate::property::ClientpidmapError),
    /// Invalid FBURL value.
    #[error("invalid fburl: {0}")]
    InvalidFburl(#[from] crate::property::FburlError),
    /// Invalid FN value.
    #[error("invalid fn: {0}")]
    InvalidFn(#[from] crate::property::FnError),
    /// Invalid GEO value.
    #[error("invalid geo: {0}")]
    InvalidGeo(#[from] crate::property::GeoError),
    /// Invalid IMPP value.
    #[error("invalid impp: {0}")]
    InvalidImpp(#[from] crate::property::ImppError),
    /// Invalid KEY value.
    #[error("invalid key: {0}")]
    InvalidKey(#[from] crate::property::KeyError),
    /// Invalid KIND value.
    #[error("invalid kind: {0}")]
    InvalidKind(#[from] crate::property::KindError),
    /// Invalid LANG value.
    #[error("invalid lang: {0}")]
    InvalidLang(#[from] crate::property::LangError),
    /// Invalid LOGO value.
    #[error("invalid logo: {0}")]
    InvalidLogo(#[from] crate::property::LogoError),
    /// Invalid MEMBER value.
    #[error("invalid member: {0}")]
    InvalidMember(#[from] crate::property::MemberError),
    /// Invalid NICKNAME value.
    #[error("invalid nickname: {0}")]
    InvalidNickname(#[from] crate::property::NicknameError),
    /// Invalid NOTE value.
    #[error("invalid note: {0}")]
    InvalidNote(#[from] crate::property::NoteError),
    /// Invalid N value.
    #[error("invalid n: {0}")]
    InvalidN(#[from] crate::property::NError),
    /// Invalid ORG value.
    #[error("invalid org: {0}")]
    InvalidOrg(#[from] crate::property::OrgError),
    /// Invalid PHOTO value.
    #[error("invalid photo: {0}")]
    InvalidPhoto(#[from] crate::property::PhotoError),
    /// Invalid PRODID value.
    #[error("invalid prodid: {0}")]
    InvalidProdid(#[from] crate::property::ProdidError),
    /// Invalid RELATED value.
    #[error("invalid related: {0}")]
    InvalidRelated(#[from] crate::property::RelatedError),
    /// Invalid REV value.
    #[error("invalid rev: {0}")]
    InvalidRev(#[from] crate::property::RevError),
    /// Invalid ROLE value.
    #[error("invalid role: {0}")]
    InvalidRole(#[from] crate::property::RoleError),
    /// Invalid SOUND value.
    #[error("invalid sound: {0}")]
    InvalidSound(#[from] crate::property::SoundError),
    /// Invalid SOURCE value.
    #[error("invalid source: {0}")]
    InvalidSource(#[from] crate::property::SourceError),
    /// Invalid TEL value.
    #[error("invalid tel: {0}")]
    InvalidTel(#[from] crate::property::TelError),
    /// Invalid TITLE value.
    #[error("invalid title: {0}")]
    InvalidTitle(#[from] crate::property::TitleError),
    /// Invalid TZ value.
    #[error("invalid tz: {0}")]
    InvalidTz(#[from] crate::property::TzError),
    /// Invalid UID value.
    #[error("invalid uid: {0}")]
    InvalidUid(#[from] crate::property::UidError),
    /// Invalid URL value.
    #[error("invalid url: {0}")]
    InvalidUrl(#[from] crate::property::UrlError),
    /// Invalid XML value.
    #[error("invalid xml: {0}")]
    InvalidXml(#[from] crate::property::XmlError),
    /// Invalid Iana value.
    #[error("invalid iana: {0}")]
    InvalidIana(#[from] crate::property::GenericIanaError),
    /// Invalid XName value.
    #[error("invalid x-name: {0}")]
    InvalidXName(#[from] crate::property::GenericXNameError),
    /// Invalid VERSION value.
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] crate::property::VersionError),
    /// Unsupported VERSION value.
    #[error("unsupported vCard version: {0}")]
    UnsupportedVersion(Version),
    /// Multiple VERSION properties found.
    #[error("multiple VERSION properties found")]
    MultipleVersion,
    /// VERSION property not at second line for vCard 4.0.
    #[error("VERSION property not at second line")]
    VersionNotSecondLine,
}

/// Result of parsing.
#[derive(Debug, PartialEq)]
pub struct ParserOutput<'a, T: 'a> {
    /// The content that was matched by the parser_internals.
    matched: &'a [u8],
    /// The content that was left after the matched content.
    remaining: &'a [u8],
    /// A field that can wrap the matched content to provide specific business logic,
    /// such as unescaping a value, returning a normalized property name, etc.
    output: T,
}

impl<'a> ParserOutput<'a, ()> {
    pub fn new(matched: &'a [u8], remaining: &'a [u8]) -> ParserOutput<'a, ()> {
        ParserOutput {
            matched,
            remaining,
            output: (),
        }
    }
}

impl<'a, T> ParserOutput<'a, T> {
    pub fn with_output(matched: &'a [u8], remaining: &'a [u8], output: T) -> ParserOutput<'a, T> {
        ParserOutput {
            matched,
            remaining,
            output,
        }
    }

    pub fn matched(&self) -> &'a [u8] {
        self.matched
    }

    pub fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    pub fn output(&self) -> &T {
        &self.output
    }

    pub fn into_output(self) -> T {
        self.output
    }
}

/// Alias for parsing results.
pub type ParserResult<'a, T> = Result<ParserOutput<'a, T>, ParserError>;
