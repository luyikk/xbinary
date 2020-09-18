use bytes::{BytesMut, Bytes, Buf};
use bytes::buf::BufMut;
use std::mem::MaybeUninit;


#[derive(Debug)]
pub struct XBWrite{
    buffer:BytesMut,
    position:usize
}

impl BufMut for XBWrite{
    fn remaining_mut(&self) -> usize {
        self.buffer.remaining_mut()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.buffer.advance_mut(cnt)
    }

    fn bytes_mut(&mut self) -> &mut [MaybeUninit<u8>] {
       self.buffer.bytes_mut()
    }

    fn put_slice(&mut self, src: &[u8]) {
        let len=self.check_resize(src.len());
        self.buffer[self.position..self.position+len].as_mut().put_slice(src);
        self.position+=len;
    }
}


impl XBWrite{
    pub fn new()->XBWrite{
        XBWrite{
            buffer:BytesMut::new(),
            position:0
        }
    }

    pub fn len(&self)->usize{
        self.buffer.len()
    }

    pub fn reset(&mut self){
        self.buffer.resize(0,0);
        self.position=0;

    }

    pub fn get_position(&self)->usize{
        self.position
    }

    pub fn set_position(&mut self,position:usize)->bool{
        if position>self.buffer.len(){
            return false;
        }
        self.position=position;
        return true;
    }

    pub fn check_resize(&mut self,put_len:usize)->usize{
        let have_len = self.buffer.len() - self.position;
        let need_add = put_len as isize - have_len as isize;
        if need_add > 0 {
            self.buffer.resize(self.buffer.len() + need_add as usize, 0);
        }
        put_len
    }

    pub fn write(&mut self,buff:&[u8]) {
        let buff_len=buff.len();
        self.check_resize(buff_len);
        self.buffer[self.position..self.position + buff_len].copy_from_slice(buff);
        self.position += buff_len;
    }

    pub fn write_bit7_len(&mut self,buff:&[u8]){
        self.bit7_write_u32(buff.len() as u32);
        self.write(buff);
    }

    pub fn write_u32_len(&mut self,buff:&[u8]){
        self.put_u32_le(buff.len() as u32);
        self.write(buff);
    }

    pub fn write_string(&mut self,str:&str){
        self.write(str.as_bytes());
    }

    pub fn write_string_bit7_len(&mut self,str:&str){
        let buff=str.as_bytes();
        self.bit7_write_u32(buff.len() as u32);
        self.write(buff);
    }

    pub fn write_string_u32_le_len(&mut self,str:&str){
        let buff=str.as_bytes();
        self.put_u32_le(buff.len() as u32);
        self.write(buff);
    }

    pub fn bit7_write_u16(&mut self,value:u16){
        let mut offset=0;
        let mut v=value;
        let mut buff=[0;16];
        while v>=1<<7 {
            buff[offset]=(v&0x7f|0x80) as u8;
            offset+=1;
            v= v>>7;
        }
        buff[offset]=v as u8;
        offset+=1;
        self.write(&buff[..offset]);
    }

    pub fn bit7_write_i16(&mut self,value:i16) {
        self.bit7_write_u16(zig_zag_encode_u16(value));
    }

    pub fn bit7_write_u32(&mut self,value:u32){
        let mut offset=0;
        let mut v=value;
        let mut buff=[0;16];
        while v>=1<<7 {
            buff[offset]=(v&0x7f|0x80) as u8;
            offset+=1;
            v= v>>7;
        }
        buff[offset]=v as u8;
        offset+=1;
        self.write(&buff[..offset]);
    }

    pub fn bit7_write_i32(&mut self,value:i32) {
        self.bit7_write_u32(zig_zag_encode_u32(value));
    }

    pub fn bit7_write_u64(&mut self,value:u64){
        let mut offset=0;
        let mut v=value;
        let mut buff=[0;16];
        while v>=1<<7 {
            buff[offset]=(v&0x7f|0x80) as u8;
            offset+=1;
            v= v>>7;
        }
        buff[offset]=v as u8;
        offset+=1;
        self.write(&buff[..offset]);
    }

    pub fn bit7_write_i64(&mut self,value:i64) {
        self.bit7_write_u64(zig_zag_encode_u64(value));
    }

    pub fn to_vec(&self)->Vec<u8>{
        self.buffer.to_vec()
    }

    pub fn flush(self)->BytesMut{
        self.buffer
    }
}

fn zig_zag_encode_u16(v:i16)->u16{
    ((v << 1) ^ (v >> 15)) as u16
}

fn zig_zag_encode_u32(v:i32)->u32{
    ((v << 1) ^ (v >> 31)) as u32
}

fn zig_zag_encode_u64(v:i64)->u64{
    ((v << 1) ^ (v >> 63)) as u64
}

fn zig_zag_decode_i16(v:u16)->i16{
    ((v>>1) as i16)^(-((v&1) as i16))
}

fn zig_zag_decode_i32(v:u32) ->i32{
    ((v>>1) as i32)^(-((v&1) as i32))
}

fn zig_zag_decode_i64(v:u64) ->i64{
    ((v>>1) as i64)^(-((v&1) as i64))
}

pub struct XBRead{
    buffer:Bytes
}

impl Buf for XBRead{
    fn remaining(&self) -> usize {
       self.buffer.remaining()
    }

    fn bytes(&self) -> &[u8] {
        self.buffer.bytes()
    }

    fn advance(&mut self, cnt: usize) {
        self.buffer.advance(cnt)
    }
}

impl XBRead{
    pub fn new(buff:Bytes)->XBRead{
        XBRead{
            buffer:buff
        }
    }

    pub fn read_bit7_u16(&self)->(usize,u16){
        let mut v=0;
        let mut offset=0;
        let mut shift=0;
        while shift<2*8 {
            if offset>self.buffer.len(){
                return (0,v);
            }

            let b=self.buffer[offset];
            offset+=1;
            v|=((b&0x7F) as u16)<<shift;
            if b&0x80 ==0{
                return (offset,v);
            }
            shift+=7;
        }
        (0,0)
    }

    pub fn read_bit7_i16(&self)->(usize,i16){
        let (offset,v)=self.read_bit7_u16();
        let v= zig_zag_decode_i16(v);
        (offset,v)
    }

    pub fn read_bit7_u32(&self)->(usize,u32){
        let mut v=0;
        let mut offset=0;
        let mut shift=0;
        while shift<4*8 {
            if offset>self.buffer.len(){
                return (0,v);
            }

            let b=self.buffer[offset];
            offset+=1;
            v|=((b&0x7F) as u32)<<shift;
            if b&0x80 ==0{
                return (offset,v);
            }
            shift+=7;
        }
        (0,0)
    }

    pub fn read_bit7_i32(&self)->(usize,i32){
        let (offset,v)=self.read_bit7_u32();
        let v= zig_zag_decode_i32(v);
        (offset,v)
    }

    pub fn read_bit7_u64(&self)->(usize,u64){
        let mut v=0;
        let mut offset=0;
        let mut shift=0;
        while shift<8*8 {
            if offset>self.buffer.len(){
                return (0,v);
            }

            let b=self.buffer[offset];
            offset+=1;
            v|=((b&0x7F) as u64)<<shift;
            if b&0x80 ==0{
                return (offset,v);
            }
            shift+=7;
        }
        (0,0)
    }

    pub fn read_bit7_i64(&self)->(usize,i64){
        let (offset,v)=self.read_bit7_u64();
        let v= zig_zag_decode_i64(v);
        (offset,v)
    }

    pub fn read_string_bit7_len(&mut self)->Option<String>{
        let (offset,len)=self.read_bit7_u32();
        if offset == 0||len==0{
            None
        }
        else{
            self.advance(offset);
            Some(self.read_string(len as usize))
        }
    }

    pub fn read_vec_bit7_len(&mut self)->Option<Vec<u8>>{
        let (offset,len)=self.read_bit7_u32();
        if offset == 0||len==0{
            None
        }
        else{
            self.advance(offset);
            Some(self.read_vec(len as usize))
        }
    }

    pub fn read_string_u32_le(&mut self)->Option<String>{
        let len=self.get_u32_le();
        if len==0||len==0{
            None
        }
        else {
            Some(self.read_string(len as usize))
        }
    }

    pub fn read_vec_u32_le(&mut self)->Option<Vec<u8>>{
        let len=self.get_u32_le();
        if len==0||len==0{
            None
        }
        else {
            Some(self.read_vec(len as usize))
        }
    }

    pub fn read_string(&mut self,len:usize)->String{
        let str=String::from_utf8_lossy(&self.buffer[..len as usize]).to_string();
        self.advance(len as usize);
        str
    }

    pub fn read_vec(&mut self,len:usize)->Vec<u8>{
        let vec=self.buffer[..len as usize].to_vec();
        self.advance(len as usize);
        vec
    }


}