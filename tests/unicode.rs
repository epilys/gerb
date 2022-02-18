/*

use crate::unicode::names::CharName;
fn some_s(s: &str) -> Option<String> {
    Some(s.to_string())
}
#[test]
fn test_names() {
    assert_eq!(
        some_s("LATIN CAPITAL LETTER A"),
        'A'.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s(
            "ARABIC LIGATURE UIGHUR KIRGHIZ YEH WITH HAMZA ABOVE WITH ALEF MAKSURA ISOLATED FORM"
        ),
        0xFBF9u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(some_s("OX"), 0x1F402u32.char_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("HANGUL JUNGSEONG O-E"),
        0x1180u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRAKCET"),
        0xFE18u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(None, 0x0009u32.property_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("<control-0009>"),
        0x0009u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("ZERO WIDTH NO-BREAK SPACE"),
        0xFEFFu32.property_name().map(|x| x.to_string())
    );
    assert_eq!(None, 0x0081u32.property_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("<control-0081>"),
        0x0081u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("ZERO WIDTH SPACE"),
        0x200Bu32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("HANGUL SYLLABLE PWILH"),
        0xD4DBu32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("CJK UNIFIED IDEOGRAPH-4E00"),
        0x4E00u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("TANGUT IDEOGRAPH-17000"),
        0x17000u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("GURMUKHI LETTER KA"),
        0x0A15u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("ZERO WIDTH JOINER"),
        0x200Du32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("CJK COMPATIBILITY IDEOGRAPH-FA0E"),
        0xFA0Eu32.char_name().map(|x| x.to_string())
    );
    assert_eq!(
        some_s("CJK COMPATIBILITY IDEOGRAPH-FA29"),
        0xFA29u32.char_name().map(|x| x.to_string())
    );
    assert_eq!(None, 0x1029Fu32.property_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("<reserved-1029F>"),
        0x1029Fu32.char_name().map(|x| x.to_string())
    );
    assert_eq!(None, 0xFFFEu32.property_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("<noncharacter-FFFE>"),
        0xFFFEu32.char_name().map(|x| x.to_string())
    );
    assert_eq!(None, 0xDC00u32.property_name().map(|x| x.to_string()));
    assert_eq!(
        some_s("<surrogate-DC00>"),
        0xDC00u32.char_name().map(|x| x.to_string())
    );
}
*/
