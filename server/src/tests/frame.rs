use super::*;

#[test]
fn validation_simple_string() {
    let buff = b"+Hello\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    let buff = b"+Hello\r";
    assert_ne!(Frame::validate(buff), Ok(()));

    let buff = b"+\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    let buff = b"+";
    assert_ne!(Frame::validate(buff), Ok(()));
}

#[test]
fn validation_error() {
    let mut buff = b"-ERROR\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b"-Error\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"-Error\r";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"-Error";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"-\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));
}

#[test]
fn validation_int() {
    let mut buff = b":0\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b":000001\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b":992123\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = format!(b":{}\r\n", u64::MAX);
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b":0 0\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b":0.0\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = format!(b":{}9\r\n", u64::MAX);
    assert_ne!(Frame::validate(buff), Ok(()));
}

#[test]
fn validation_null() {
    let mut buff = b"_\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b"_asd\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));
}

#[test]
fn validation_bulk_string() {
    let mut buff = b"$0005\r\nHello\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b"$0000\r\n\r\n";
    assert_eq!(Frame::validate(buff), Ok(()));

    buff = b"$0004\r\nHello\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"$\r\nHello\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"$03\r\nHel\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"$0003\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));

    buff = b"$0000\r\n";
    assert_ne!(Frame::validate(buff), Ok(()));
}

#[test]
fn validation_array() {}
