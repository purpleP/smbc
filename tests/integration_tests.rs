use smbc;

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;

#[test]
fn test_read() {
    let mut buf = Vec::new();
    let mut file = File::open("/var/test_data/test.txt")
        .expect("Test data not found in /var/test_data/test.txt!");
    file.read_to_end(&mut buf).unwrap();
    let expected_data = buf;
    (0..10)
        .map(|_| {
            std::thread::spawn(|| {
                let auth = |_srv: &[u8], _shr: &[u8]| smbc::Credentials {
                    raw_workgroup: Cow::Borrowed(&b"WORKGROUP\0"[..]),
                    raw_username: Cow::Borrowed(&b"user\0"[..]),
                    raw_password: Cow::Borrowed(&b"pass\0"[..]),
                };
                let ctx = smbc::Context::new(&auth).unwrap();
                let mut file =
                    ctx.open_ro(b"smb://smbserver/public/test.txt\0").unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                buf
            })
        })
        .for_each(|handle| {
            let buf = handle.join().unwrap();
            assert_eq!(&expected_data[..], &buf[..])
        })
}
