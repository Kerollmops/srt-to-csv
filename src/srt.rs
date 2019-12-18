use std::time::Duration;
use std::fmt;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, space0, line_ending},
    combinator::{map_res, opt},
    multi::fold_many0,
    sequence::delimited,
};

#[derive(Debug, Clone)]
pub struct SubTitle {
    pub index: u32,
    pub start: Time,
    pub end: Time,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl Time {
    pub fn to_duration(&self) -> Duration {
        let mut millis = self.milliseconds as u64;
        millis += self.seconds as u64 * 1000;
        millis += self.minutes as u64 * 60_000;
        millis += self.hours as u64 * 3_600_000;
        Duration::from_millis(millis)
    }
}

impl fmt::Display for Time {
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

fn until_empty_newline(i: &str) -> IResult<&str, &str> {
    let bytes = i.as_bytes();
    match bytes.windows(4).position(|x| &x[0..2] == b"\n\n" || x == b"\r\n\r\n") {
        Some(pos) if &bytes[pos..pos + 2] == b"\n\n" => {
            let (taken, rest) = i.split_at(pos + 2);
            Ok((rest, taken))
        },
        Some(pos) => {
            let (taken, rest) = i.split_at(pos + 4);
            Ok((rest, taken))
        }
        None => Ok(("", i)),
    }
}

// 1
// 00:00:49,174 --> 00:00:52,593
// - Is everything in place?
// - You're not to relieve me.
fn one_subtitle(i: &str) -> IResult<&str, SubTitle> {
    let integer = map_res(digit1, |s: &str| s.parse::<u32>());

    let (i, index) = integer(i)?;
    let (i, _) = line_ending(i)?;
    let (i, (start, end)) = time_and_arrow(i)?;
    let (i, _) = line_ending(i)?;
    let (i, text) = until_empty_newline(i)?;
    let text = text.trim().to_owned();

    Ok((i, SubTitle { index, start, end, text }))
}

fn opt_byte(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag("\u{feff}"))(i)
}

pub fn from_str(input: &str) -> Result<Vec<SubTitle>, ()> {
    let (input, _) = opt_byte(input).unwrap();

    let (input, subtitles) = fold_many0(
        one_subtitle,
        Vec::new(),
        |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        }
    )(input).unwrap();

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

        let subs = from_str(content).unwrap();
        let sub = &subs[0];

        assert_eq!(sub.index, 1);
        assert_eq!(sub.text, r#"- Is everything in place?
- You're not to relieve me."#);

        let sub = &subs[1];

        assert_eq!(sub.index, 2);
        assert_eq!(sub.text, r#"I know, but I felt like taking a shift."#);
    }
}
