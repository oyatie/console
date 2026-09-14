//! In-owner native28 streaming contracts; no fabricated business fixtures.
use super::native28::{decode_input_stream,encode_input_stream,InputFrame};
use std::io::{self,Read,Write,Cursor};
struct Fragmented<'a>{bytes:&'a[u8],position:usize,max_chunk:usize,read_bytes:usize}
impl Read for Fragmented<'_>{fn read(&mut self,out:&mut[u8])->io::Result<usize>{
 let n=out.len().min(self.max_chunk).min(self.bytes.len()-self.position);
 out[..n].copy_from_slice(&self.bytes[self.position..self.position+n]);self.position+=n;self.read_bytes+=n;Ok(n)
}}
#[test]
fn native_stream_decoder_accepts_short_reads_without_duplicate_consumption(){
 let bytes=include_bytes!("codec-goldens/input-resolved-one.bin");
 let expected=decode_input_stream(&mut Cursor::new(bytes)).unwrap();
 for max_chunk in [1,7,31,4096]{
  let mut reader=Fragmented{bytes,position:0,max_chunk,read_bytes:0};
  let actual=decode_input_stream(&mut reader).unwrap();assert_eq!(actual.frame,expected.frame);
  assert_eq!(reader.read_bytes,bytes.len());
 }
}
struct HeaderOnly{prefix:Cursor<Vec<u8>>,body_reads:usize}
impl Read for HeaderOnly{fn read(&mut self,out:&mut[u8])->io::Result<usize>{
 if self.prefix.position() as usize==self.prefix.get_ref().len(){self.body_reads+=1;return Err(io::Error::other("decoder requested an oversized body"));}
 self.prefix.read(out)
}}
#[test]
fn native_stream_declared_u32_max_header_refuses_before_any_body_read(){
 let complete=include_bytes!("codec-goldens/input-resolved-one.bin");
 // R2 fixed frame: magic/version+count ends41; header length is u32BE41..45.
 let mut prefix=complete[..45].to_vec();prefix[41..45].copy_from_slice(&u32::MAX.to_be_bytes());
 let mut reader=HeaderOnly{prefix:Cursor::new(prefix),body_reads:0};
 assert!(decode_input_stream(&mut reader).is_err());assert_eq!(reader.body_reads,0);
}
struct FailAfter{remaining:usize,written:usize}
impl Write for FailAfter{fn write(&mut self,bytes:&[u8])->io::Result<usize>{
 if self.remaining==0{return Err(io::Error::other("actual destination failed"));}
 let n=bytes.len().min(self.remaining);self.remaining-=n;self.written+=n;Ok(n)
}fn flush(&mut self)->io::Result<()>{Ok(())}}
#[test]
fn native_stream_writer_never_returns_complete_manifest_after_partial_destination_failure(){
 let fixture:InputFrame=serde_json::from_slice(include_bytes!("codec-goldens/input-resolved-one.json")).unwrap();
 for limit in [0,1,44,45,4363]{
  let mut writer=FailAfter{remaining:limit,written:0};
  assert!(encode_input_stream(&fixture,&mut writer).is_err());assert_eq!(writer.written,limit);
 }
}
