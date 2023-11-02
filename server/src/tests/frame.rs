use crate::frame::Frame;
use std::io::Cursor;

macro_rules! into_cursor {
    ($b:tt) => {
        Cursor::new(&$b[..])
    };
}

#[test]
fn validation_simple_string() {
    assert_eq!(Frame::validate(&mut into_cursor!(b"+Hello\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"+hello\r")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"+\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"+")), Ok(()));
}

#[test]
fn validation_error() {
    assert_eq!(Frame::validate(&mut into_cursor!(b"-ERROR\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"-Error\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"-Error\r")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"-Error")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"-\r\n")), Ok(()));
}

#[test]
fn validation_int() {
    assert_eq!(Frame::validate(&mut into_cursor!(b":0\r\n")), Ok(()));
    assert_eq!(Frame::validate(&mut into_cursor!(b":000001\r\n")), Ok(()));
    assert_eq!(Frame::validate(&mut into_cursor!(b":992123\r\n")), Ok(()));

    let buff = format!(":{}\r\n", u64::MAX).as_bytes();
    assert_eq!(Frame::validate(&mut into_cursor!(buff)), Ok(()));

    assert_ne!(Frame::validate(&mut into_cursor!(b":0 0\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b":0.0\r\n")), Ok(()));

    let buff = format!(":{}9\r\n", u64::MAX).as_bytes();
    assert_ne!(Frame::validate(&mut into_cursor!(buff)), Ok(()));
}

#[test]
fn validation_null() {
    assert_eq!(Frame::validate(&mut into_cursor!(b"_\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"_asd\r\n")), Ok(()));
}

#[test]
fn validation_bulk_string() {
    assert_eq!(
        Frame::validate(&mut into_cursor!(b"$0005\r\nHello\r\n")),
        Ok(())
    );
    assert_eq!(Frame::validate(&mut into_cursor!(b"$0000\r\n\r\n")), Ok(()));
    assert_ne!(
        Frame::validate(&mut into_cursor!(b"$0004\r\nHello\r\n")),
        Ok(())
    );
    assert_ne!(
        Frame::validate(&mut into_cursor!(b"$\r\nHello\r\n")),
        Ok(())
    );
    assert_ne!(
        Frame::validate(&mut into_cursor!(b"$03\r\nHel\r\n")),
        Ok(())
    );
    assert_ne!(Frame::validate(&mut into_cursor!(b"$0003\r\n")), Ok(()));
    assert_ne!(Frame::validate(&mut into_cursor!(b"$0000\r\n")), Ok(()));
}

#[test]
fn validation_array() {}
