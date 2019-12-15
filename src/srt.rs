use std::io::{self, Read};
use std::fmt;
use nom::{
  IResult,
  sequence::delimited,
  combinator::{map_res, opt},
  character::complete::{digit1, space0, crlf},
  bytes::complete::tag,
};

#[derive(Clone)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{},{}", self.hours, self.minutes, self.seconds, self.milliseconds)
    }
}

// 00:00:49,174
fn one_time(i: &str) -> IResult<&str, Time> {
    let get_u8 = map_res(digit1, |s: &str| s.parse::<u8>());
    let get_u16 = map_res(digit1, |s: &str| s.parse::<u16>());

    let (i, hours) = get_u8(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, minutes) = get_u8(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, seconds) = get_u8(i)?;
    let (i, _) = tag(",")(i)?;
    let (i, milliseconds) = get_u16(i)?;

    Ok((i, Time { hours, minutes, seconds, milliseconds }))
}

// 00:00:49,174 --> 00:00:52,593
fn time_and_arrow(i: &str) -> IResult<&str, (Time, Time)> {
    let (i, start) = one_time(i)?;
    let (i, _) = delimited(space0, tag("-->"), space0)(i)?;
    let (i, end) = one_time(i)?;

    Ok((i, (start, end)))
}

#[derive(Debug, Clone)]
pub struct SubTitle {
    pub index: u32,
    pub start: Time,
    pub end: Time,
    pub text: String,
}

fn until_empty_newline(i: &str) -> IResult<&str, &str> {
    let lines = i.lines();
    let mut count = 0;

    for line in lines {
        count += line.len() + 2; // for the crlf (it's wrong)
        if line.is_empty() { break }
    }

    let (text, rest) = i.split_at(count);
    Ok((rest, text.trim()))
}

// 1
// 00:00:49,174 --> 00:00:52,593
// - Is everything in place?
// - You're not to relieve me.
fn one_subtitle(i: &str) -> IResult<&str, SubTitle> {
    let integer = map_res(digit1, |s: &str| s.parse::<u32>());

    let (i, index) = integer(i)?;
    let (i, _) = crlf(i)?;
    let (i, (start, end)) = time_and_arrow(i)?;
    let (i, _) = crlf(i)?;
    let (i, text) = until_empty_newline(i)?;
    let text = text.to_owned();

    Ok((i, SubTitle { index, start, end, text }))
}

fn parser(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag("\u{feff}"))(i)
}

pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Vec<SubTitle>> {
    let mut input = String::new();
    reader.read_to_string(&mut input)?;

    let mut subtitles = Vec::new();
    let (mut input, _) = parser(&input[..]).unwrap();

    loop {
        let (i, sub) = one_subtitle(&input).unwrap(); // bad !
        subtitles.push(sub);
        if i.is_empty() { break }
        input = i;
    }

    Ok(subtitles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy_two_lines() {
        let content = r#"1
00:00:49,174 --> 00:00:52,593
- Is everything in place?
- You're not to relieve me.

2
00:00:52,844 --> 00:00:55,471
I know, but I felt like taking a shift.

"#;

        let subs = from_reader(content.as_bytes()).unwrap();
        let sub = &subs[0];

        assert_eq!(sub.index, 1);
        assert_eq!(sub.text, r#"- Is everything in place?
- You're not to relieve me."#);

        let sub = &subs[1];

        assert_eq!(sub.index, 2);
        assert_eq!(sub.text, r#"I know, but I felt like taking a shift."#);
    }
}
