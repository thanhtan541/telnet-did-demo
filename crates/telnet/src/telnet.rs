use std::io::{self, Read};
use tokio_util::{bytes::Buf, codec::Decoder};

pub struct TelnetCodec {
    current_line: Vec<u8>,
}

impl TelnetCodec {
    pub fn new() -> Self {
        TelnetCodec {
            current_line: Vec::with_capacity(1024),
        }
    }
}

#[derive(Debug)]
pub enum Item {
    ShowDID(Vec<u8>),
    VerifyDID(Vec<u8>),
    AssignRole(Vec<u8>),
    WhoAmI,
    ShowVP, // Show Verifiable Presentation
    CreateDID,
    Line(Vec<u8>),
    SE,
    DataMark,
    Break,
    InterruptProcess,
    AbortOutput,
    AreYouThere,
    GoAhead,
    SB,
    Will(u8),
    Wont(u8),
    Do(u8),
    Dont(u8),
}

impl Decoder for TelnetCodec {
    type Item = Item;
    type Error = io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if src.is_empty() {
                return Ok(None);
            }

            if src[0] == 0xff {
                let (res, consume) = try_parse_iac(src.chunk());
                src.advance(consume);

                match res {
                    ParseIacResult::Invalid(err) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, err));
                    }
                    ParseIacResult::NeedMore => return Ok(None),
                    ParseIacResult::Item(item) => return Ok(Some(item)),
                    ParseIacResult::NOP => { /* go around loop */ }
                    ParseIacResult::EraseCharacter => {
                        self.current_line.pop();
                    }
                    ParseIacResult::EraseLine => {
                        self.current_line.clear();
                    }
                    ParseIacResult::Escaped => {
                        self.current_line.push(0xff);
                    }
                }
            } else {
                let byte = src.get_u8();

                match byte {
                    10 => {
                        let line = self.current_line.to_vec();
                        self.current_line.clear();
                        let item = parse_line(line);

                        return Ok(item);
                    }
                    0..=31 => {
                        // ignore
                    }
                    _ => self.current_line.push(byte),
                }
            }
        }
    }
}

enum ParseIacResult {
    Invalid(String),
    NeedMore,
    Item(Item),
    NOP,
    EraseCharacter,
    EraseLine,
    Escaped,
}

fn try_parse_iac(bytes: &[u8]) -> (ParseIacResult, usize) {
    if bytes.len() < 2 {
        return (ParseIacResult::NeedMore, 0);
    }
    if bytes[0] != 0xff {
        unreachable!();
    }
    if is_three_byte_iac(bytes[1]) && bytes.len() < 3 {
        return (ParseIacResult::NeedMore, 0);
    }

    match bytes[1] {
        240 => (ParseIacResult::Item(Item::SE), 2),
        241 => (ParseIacResult::NOP, 2),
        242 => (ParseIacResult::Item(Item::DataMark), 2),
        243 => (ParseIacResult::Item(Item::Break), 2),
        244 => (ParseIacResult::Item(Item::InterruptProcess), 2),
        245 => (ParseIacResult::Item(Item::AbortOutput), 2),
        246 => (ParseIacResult::Item(Item::AreYouThere), 2),
        247 => (ParseIacResult::EraseCharacter, 2),
        248 => (ParseIacResult::EraseLine, 2),
        249 => (ParseIacResult::Item(Item::GoAhead), 2),
        250 => (ParseIacResult::Item(Item::SB), 2),
        251 => (ParseIacResult::Item(Item::Will(bytes[2])), 3),
        252 => (ParseIacResult::Item(Item::Wont(bytes[2])), 3),
        253 => (ParseIacResult::Item(Item::Do(bytes[2])), 3),
        254 => (ParseIacResult::Item(Item::Dont(bytes[2])), 3),
        255 => (ParseIacResult::Escaped, 2),
        cmd => (
            ParseIacResult::Invalid(format!("Unknown IAC command {}.", cmd)),
            0,
        ),
    }
}

fn is_three_byte_iac(byte: u8) -> bool {
    match byte {
        251..=254 => true,
        _ => false,
    }
}

// Mark: Decentralized Identifier v1.0
fn parse_line(line: Vec<u8>) -> Option<Item> {
    println!(
        "[Client] sent command in byte {:?}",
        String::from_utf8_lossy(&line)
    );
    // c#cdid == command: [c]reate did
    if line.to_vec() == b"c#cdid".to_vec() {
        return Some(Item::CreateDID);
    }

    // c#wai== command: [w]ho [a]m [i]
    if line.to_vec() == b"c#wai".to_vec() {
        return Some(Item::WhoAmI);
    }

    // c#svp == command: [s]how [v]erifiable [p]resenation
    if line.to_vec() == b"c#svp".to_vec() {
        return Some(Item::ShowVP);
    }

    // c#sdid == command: [s]show did
    if line.to_vec()[0..6] == b"c#sdid".to_vec() {
        let did = &line[6..];
        return Some(Item::ShowDID(did.to_vec()));
    }

    // c#ar == command: [a]ssign [r]ole
    if line.to_vec()[0..4] == b"c#ar".to_vec() {
        let role = &line[4..];
        return Some(Item::AssignRole(role.to_vec()));
    }

    // c#vdid == command: [v]erify did
    if line.to_vec()[0..6] == b"c#vdid".to_vec() {
        let did = &line[6..];
        return Some(Item::VerifyDID(did.to_vec()));
    }
    //Todo: Add command from client

    return Some(Item::Line(line));
}
