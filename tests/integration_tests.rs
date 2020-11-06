use smbc;

use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Read;
use std::thread::JoinHandle;
use std::fs::File;

#[test]
fn test_read() {
    let mut buf = Vec::new();
    let file = File::open("/var/test_data/test.txt")
        .expect("Test data not found in /var/test_data/test.txt!");
    file.read_to_end(&expected_data).unwrap();
    let expected_data = buf;
    (0..10)
        .map(|_| {
            std::thread::spawn(|| unsafe {
                let auth = |srv: &[u8], shr: &[u8]| {
                    println!(
                        "srv {:?}, shr {:?}",
                        CStr::from_bytes_with_nul_unchecked(srv),
                        CStr::from_bytes_with_nul_unchecked(shr)
                    );
                    (
                        Cow::Borrowed(&b"WORKGROUP\0"[..]),
                        Cow::Borrowed(&b"user\0"[..]),
                        Cow::Borrowed(&b"pass\0"[..]),
                    )
                };
                let ctx = smbc::Context::new(&auth).unwrap();
                let mut file =
                    ctx.open_ro(b"smb://smbserver/public/test.txt\0").unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                assert_eq!(expected_data, buf);
            })
        })
        .try_for_each(JoinHandle::join)
        .unwrap();
}
