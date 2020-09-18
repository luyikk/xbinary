# xbinary
bytes extended read write

## Examples echo

```rust
use bytes::{BufMut, Bytes};
use bytes::Buf;
use xbinary::*;

fn main()
{
    let mut w = XBWrite::new();
    w.put_u32_le(0);
    w.put_u32_le(1000);
    w.write_string_bit7_len("cmd");
    w.write_string_u32_le_len("cmd2");
    w.bit7_write_u32(111111);
    w.bit7_write_u16(65535);
    w.put_f64_le(0.555);
    w.set_position(0);
    let len=w.len() as u32;
    w.put_u32_le( len- 4);
    let buff = w.flush();
    println!("{:#x?}",buff.to_vec());
    let mut r = XBRead::new(Bytes::from(buff));
    assert_eq!(r.get_u32_le(),len - 4);
    assert_eq!(r.get_u32_le(),1000);
    assert_eq!(r.read_string_bit7_len().unwrap(),"cmd");
    assert_eq!(r.read_string_u32_le().unwrap(),"cmd2");
    let (offset,v)=r.read_bit7_u32();
    assert_eq!(v,111111);
    r.advance(offset);
    let (offset,v)=r.read_bit7_u16();
    assert_eq!(v,65535);
    r.advance(offset);
    assert_eq!(r.get_f64_le(),0.555);
}
```