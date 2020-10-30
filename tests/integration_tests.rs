use smbc;

use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Read;
use std::thread::JoinHandle;

#[test]
fn test_read() {
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
                    ctx.open_ro(b"smb://localhost/public/test.txt\0").unwrap();
                let mut buf = [0u8; 1024];
                let n = file.read(&mut buf).unwrap();
                assert_eq!(b"data\n", &buf[..n]);
            })
        })
        .try_for_each(JoinHandle::join)
        .unwrap();
}
