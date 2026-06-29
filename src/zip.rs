//! ZIP archive read/write for LIPI — pure Rust, no external crates.
//!
//! Exposed as the stdlib module `भारत.संपीडन`. Reading supports both STORE (0) and
//! DEFLATE (8) entries (a full inflate implementation is included), so archives
//! produced by standard tools can be read. Writing uses STORE (uncompressed) which
//! is always a valid ZIP. Entry contents are treated as UTF-8 text.
//!
//! - `ज़िप_लिखो(path, कोश)`  — write {name: text} dict to a .zip (STORE)
//! - `ज़िप_पढ़ो(path)`       — read a .zip → {name: text} dict
//! - `ज़िप_सूची(path)`       — list entry names (List of Str)

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

// ── CRC-32 (IEEE, polynomial 0xEDB88320) ─────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

// ── DEFLATE inflate (RFC 1951) ───────────────────────────────────────────────

struct BitReader<'a> { data: &'a [u8], byte: usize, bit: u32 }

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self { BitReader { data, byte: 0, bit: 0 } }

    fn bit(&mut self) -> Result<u32, String> {
        if self.byte >= self.data.len() { return Err("ज़िप: deflate डेटा अधूरा".to_string()); }
        let b = (self.data[self.byte] >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 { self.bit = 0; self.byte += 1; }
        Ok(b as u32)
    }

    fn bits(&mut self, n: u32) -> Result<u32, String> {
        let mut v = 0u32;
        for i in 0..n { v |= self.bit()? << i; }
        Ok(v)
    }

    fn align(&mut self) { if self.bit != 0 { self.bit = 0; self.byte += 1; } }
}

/// Canonical Huffman decoder built from a list of code lengths.
struct Huffman { counts: Vec<u16>, symbols: Vec<u16> }

impl Huffman {
    fn new(lengths: &[u16]) -> Huffman {
        let max_bits = *lengths.iter().max().unwrap_or(&0) as usize;
        let mut counts = vec![0u16; max_bits + 1];
        for &l in lengths { if l > 0 { counts[l as usize] += 1; } }
        // offsets per length
        let mut offsets = vec![0u16; max_bits + 2];
        for i in 1..=max_bits { offsets[i + 1] = offsets[i] + counts[i]; }
        let mut symbols = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l > 0 {
                symbols[offsets[l as usize] as usize] = sym as u16;
                offsets[l as usize] += 1;
            }
        }
        Huffman { counts, symbols }
    }

    fn decode(&self, br: &mut BitReader) -> Result<u16, String> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..self.counts.len() {
            code |= br.bit()? as i32;
            let count = self.counts[len] as i32;
            if code - first < count {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("ज़िप: अमान्य Huffman कोड".to_string())
    }
}

const LEN_BASE: [u16; 29] = [3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258];
const LEN_EXTRA: [u16; 29] = [0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0];
const DIST_BASE: [u16; 30] = [1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577];
const DIST_EXTRA: [u16; 30] = [0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];

fn inflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut br = BitReader::new(data);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let last = br.bit()?;
        let btype = br.bits(2)?;
        match btype {
            0 => {
                // stored block
                br.align();
                if br.byte + 4 > br.data.len() { return Err("ज़िप: stored block अधूरा".to_string()); }
                let len = u16::from_le_bytes([br.data[br.byte], br.data[br.byte + 1]]) as usize;
                br.byte += 4; // skip LEN + NLEN
                if br.byte + len > br.data.len() { return Err("ज़िप: stored block आकार गलत".to_string()); }
                out.extend_from_slice(&br.data[br.byte..br.byte + len]);
                br.byte += len;
            }
            1 | 2 => {
                let (lit, dist) = if btype == 1 {
                    // fixed Huffman tables
                    let mut litlen = [0u16; 288];
                    for (i, v) in litlen.iter_mut().enumerate() {
                        *v = if i < 144 { 8 } else if i < 256 { 9 } else if i < 280 { 7 } else { 8 };
                    }
                    let distlen = [5u16; 30];
                    (Huffman::new(&litlen), Huffman::new(&distlen))
                } else {
                    read_dynamic_tables(&mut br)?
                };
                loop {
                    let sym = lit.decode(&mut br)?;
                    if sym == 256 { break; }
                    if sym < 256 {
                        out.push(sym as u8);
                    } else {
                        let si = (sym - 257) as usize;
                        if si >= LEN_BASE.len() { return Err("ज़िप: अमान्य length कोड".to_string()); }
                        let length = LEN_BASE[si] as usize + br.bits(LEN_EXTRA[si] as u32)? as usize;
                        let dsym = dist.decode(&mut br)? as usize;
                        if dsym >= DIST_BASE.len() { return Err("ज़िप: अमान्य distance कोड".to_string()); }
                        let distance = DIST_BASE[dsym] as usize + br.bits(DIST_EXTRA[dsym] as u32)? as usize;
                        if distance > out.len() { return Err("ज़िप: distance सीमा से बाहर".to_string()); }
                        let start = out.len() - distance;
                        for i in 0..length { let b = out[start + i]; out.push(b); }
                    }
                }
            }
            _ => return Err("ज़िप: अमान्य deflate block प्रकार".to_string()),
        }
        if last == 1 { break; }
    }
    Ok(out)
}

fn read_dynamic_tables(br: &mut BitReader) -> Result<(Huffman, Huffman), String> {
    let hlit = br.bits(5)? as usize + 257;
    let hdist = br.bits(5)? as usize + 1;
    let hclen = br.bits(4)? as usize + 4;
    const ORDER: [usize; 19] = [16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];
    let mut cl_lengths = [0u16; 19];
    for i in 0..hclen { cl_lengths[ORDER[i]] = br.bits(3)? as u16; }
    let cl_huff = Huffman::new(&cl_lengths);
    let mut lengths: Vec<u16> = Vec::with_capacity(hlit + hdist);
    while lengths.len() < hlit + hdist {
        let sym = cl_huff.decode(br)?;
        match sym {
            0..=15 => lengths.push(sym),
            16 => {
                let prev = *lengths.last().ok_or("ज़िप: repeat बिना पूर्व")?;
                let rep = br.bits(2)? + 3;
                for _ in 0..rep { lengths.push(prev); }
            }
            17 => { let rep = br.bits(3)? + 3; for _ in 0..rep { lengths.push(0); } }
            18 => { let rep = br.bits(7)? + 11; for _ in 0..rep { lengths.push(0); } }
            _ => return Err("ज़िप: अमान्य code-length कोड".to_string()),
        }
    }
    let lit_lengths = &lengths[..hlit];
    let dist_lengths = &lengths[hlit..hlit + hdist];
    Ok((Huffman::new(lit_lengths), Huffman::new(dist_lengths)))
}

// ── ZIP container ────────────────────────────────────────────────────────────

fn u16le(b: &[u8], o: usize) -> usize { u16::from_le_bytes([b[o], b[o + 1]]) as usize }
fn u32le(b: &[u8], o: usize) -> usize { u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize }

fn parse_zip(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    // Find End of Central Directory (signature 0x06054b50), scanning from the end.
    if data.len() < 22 { return Err("ज़िप: फ़ाइल बहुत छोटी".to_string()); }
    let mut eocd = None;
    let start = data.len().saturating_sub(65557); // max comment + EOCD
    for i in (start..=data.len() - 22).rev() {
        if &data[i..i + 4] == [0x50, 0x4b, 0x05, 0x06] { eocd = Some(i); break; }
    }
    let eocd = eocd.ok_or("ज़िप: EOCD नहीं मिला (अमान्य ज़िप)")?;
    let count = u16le(data, eocd + 10);
    let mut cd = u32le(data, eocd + 16);
    let mut entries = Vec::new();
    for _ in 0..count {
        if cd + 46 > data.len() || &data[cd..cd + 4] != [0x50, 0x4b, 0x01, 0x02] {
            return Err("ज़िप: central directory भ्रष्ट".to_string());
        }
        let method = u16le(data, cd + 10);
        let comp_size = u32le(data, cd + 20);
        let name_len = u16le(data, cd + 28);
        let extra_len = u16le(data, cd + 30);
        let comment_len = u16le(data, cd + 32);
        let lho = u32le(data, cd + 42);
        let name = String::from_utf8_lossy(&data[cd + 46..cd + 46 + name_len]).into_owned();
        // Local file header → find actual data offset
        if lho + 30 > data.len() || &data[lho..lho + 4] != [0x50, 0x4b, 0x03, 0x04] {
            return Err("ज़िप: local header भ्रष्ट".to_string());
        }
        let l_name = u16le(data, lho + 26);
        let l_extra = u16le(data, lho + 28);
        let data_start = lho + 30 + l_name + l_extra;
        let raw = &data[data_start..data_start + comp_size];
        let content = match method {
            0 => raw.to_vec(),
            8 => inflate(raw)?,
            m => return Err(format!("ज़िप: असमर्थित संपीडन विधि {m}")),
        };
        entries.push((name, content));
        cd += 46 + name_len + extra_len + comment_len;
    }
    Ok(entries)
}

fn build_zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    let mut offsets = Vec::new();
    for (name, content) in files {
        let offset = out.len() as u32;
        offsets.push(offset);
        let crc = crc32(content);
        let nb = name.as_bytes();
        let size = content.len() as u32;
        // local file header
        out.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]);
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0u16.to_le_bytes());  // flags
        out.extend_from_slice(&0u16.to_le_bytes());  // method = STORE
        out.extend_from_slice(&0u16.to_le_bytes());  // mod time
        out.extend_from_slice(&0u16.to_le_bytes());  // mod date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());  // compressed size
        out.extend_from_slice(&size.to_le_bytes());  // uncompressed size
        out.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());  // extra len
        out.extend_from_slice(nb);
        out.extend_from_slice(content);
        // central directory record
        central.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // method
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra
        central.extend_from_slice(&0u16.to_le_bytes()); // comment
        central.extend_from_slice(&0u16.to_le_bytes()); // disk
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(nb);
    }
    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);
    // End of central directory
    out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
    out.extend_from_slice(&0u16.to_le_bytes()); // disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len
    out
}

// ── Native functions ─────────────────────────────────────────────────────────

fn zip_likho(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("ज़िप_लिखो(): पहला तर्क पथ (वाक्य) होना चाहिए".to_string()),
    };
    let dict = match args.get(1) {
        Some(Value::Dict(d)) => d,
        _ => return Err("ज़िप_लिखो(): दूसरा तर्क कोश {नाम: सामग्री} होना चाहिए".to_string()),
    };
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut names: Vec<&String> = dict.keys().collect();
    names.sort();
    for name in names {
        let content = match &dict[name] {
            Value::Str(s) => s.clone().into_bytes(),
            other => format!("{other}").into_bytes(),
        };
        files.push((name.clone(), content));
    }
    let bytes = build_zip(&files);
    std::fs::write(&path, &bytes).map_err(|e| format!("ज़िप_लिखो(): लिख नहीं सका — {e}"))?;
    Ok(Value::Bool(true))
}

fn zip_padho(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("ज़िप_पढ़ो(): पथ (वाक्य) अपेक्षित".to_string()),
    };
    let data = std::fs::read(&path).map_err(|e| format!("ज़िप_पढ़ो(): पढ़ नहीं सका — {e}"))?;
    let entries = parse_zip(&data)?;
    let mut map = HashMap::new();
    for (name, content) in entries {
        map.insert(name, Value::Str(String::from_utf8_lossy(&content).into_owned()));
    }
    Ok(Value::Dict(map))
}

fn zip_suchi(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("ज़िप_सूची(): पथ (वाक्य) अपेक्षित".to_string()),
    };
    let data = std::fs::read(&path).map_err(|e| format!("ज़िप_सूची(): पढ़ नहीं सका — {e}"))?;
    let entries = parse_zip(&data)?;
    Ok(Value::List(entries.into_iter().map(|(n, _)| Value::Str(n)).collect()))
}

pub fn sampidan_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("ज़िप_लिखो", zip_likho),
        ("ज़िप_पढ़ो", zip_padho),
        ("ज़िप_सूची", zip_suchi),
    ];
    list
}
