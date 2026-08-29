mod adr;
mod anniversary;
mod bday;
mod begin;
mod caladruri;
mod caluri;
mod categories;
mod clientpidmap;
mod email;
mod end;
mod fburl;
mod fn_prop;
mod gender;
mod geo;
mod iana_token;
mod impp;
mod key;
mod kind;
mod lang;
mod logo;
mod member;
mod n;
mod nickname;
mod note;
mod org;
mod photo;
mod prodid;
mod related;
mod rev;
mod role;
mod sound;
mod source;
mod tel;
mod title;
mod tz;
mod uid;
mod url;
mod version;
mod x_name;
mod xml;

use crate::property_support::{Property, TryFromProperty};

pub use adr::{AdrError, AdrProperty};
pub use anniversary::{AnniversaryError, AnniversaryProperty};
pub use bday::{BdayError, BdayProperty};
pub use begin::{BeginError, BeginProperty};
pub use caladruri::{CaladruriError, CaladruriProperty};
pub use caluri::{CaluriError, CaluriProperty};
pub use categories::{CategoriesError, CategoriesProperty};
pub use clientpidmap::{ClientpidmapError, ClientpidmapProperty};
pub use email::{EmailError, EmailProperty};
pub use end::{EndError, EndProperty};
pub use fburl::{FburlError, FburlProperty};
pub use fn_prop::{FnError, FnProperty};
pub use gender::{GenderError, GenderProperty};
pub use geo::{GeoError, GeoProperty};
pub use iana_token::{GenericIanaError, GenericIanaProperty};
pub use impp::{ImppError, ImppProperty};
pub use key::{KeyError, KeyProperty};
pub use kind::{KindError, KindProperty};
pub use lang::{LangError, LangProperty};
pub use logo::{LogoError, LogoProperty};
pub use member::{MemberError, MemberProperty};
pub use n::{NError, NProperty};
pub use nickname::{NicknameError, NicknameProperty};
pub use note::{NoteError, NoteProperty};
pub use org::{OrgError, OrgProperty};
pub use photo::{PhotoError, PhotoProperty};
pub use prodid::{ProdidError, ProdidProperty};
pub use related::{RelatedError, RelatedProperty};
pub use rev::{RevError, RevProperty};
pub use role::{RoleError, RoleProperty};
pub use sound::{SoundError, SoundProperty};
pub use source::{SourceError, SourceProperty};
pub use tel::{TelError, TelProperty};
pub use title::{TitleError, TitleProperty};
pub use tz::{TzError, TzProperty};
pub use uid::{UidError, UidProperty};
pub use url::{UrlError, UrlProperty};
pub use version::{VersionError, VersionProperty};
pub use x_name::{GenericXNameError, GenericXNameProperty};
pub use xml::{XmlError, XmlProperty};

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
