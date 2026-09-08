use o63::vcard::VCard;
use o63::vcard::property::EmailProperty;
use o63::vcard::property::PropertyBase;

fn main() {
    let (_, vcard) = VCard::parse(
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nEMAIL;PREF=1:john@example.com\r\nEMAIL;PREF=2:jdoe@example.net\r\nEND:VCARD\r\n",
        false,
    )
        .unwrap();

    // Retrieves the preferred email address based on the priority from the `PREF` param
    if let Some(email) = vcard.get_preferred::<EmailProperty>() {
        println!(
            "preferred email: {}",
            String::from_utf8_lossy(&email.value_to_vcard_vec())
        );
    }
}
