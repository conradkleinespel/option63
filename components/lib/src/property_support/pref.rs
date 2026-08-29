use crate::PropertyBase;

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
