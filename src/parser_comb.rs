use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error;

/// Main parsing function.
///
/// Pass any parser to it and get parsed value.
///
/// # Examples
///
/// ```
/// use lisparser::parser_comb::parse;
///
/// // assert_eq!(parse(parser, input), );
/// ```
///
/// # Errors
///
/// This function will return an error if parser will meet EOF.
pub fn parse<P: Parser>(mut parser: P, input: &str) -> Result<P::Output, Error> {
    let (parsed, rest) = parser.parse(input)?;
    if rest.is_empty() {
        Ok(parsed)
    } else {
        Err(Error)
    }
}

pub trait Parser: Sized {
    type Output;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error>;

    fn or<P: Parser>(self, parser: P) -> Or<Self, P> {
        Or {
            first: self,
            second: parser,
        }
    }

    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> T,
    {
        Map { parser: self, f }
    }

    fn flat_map<F, P>(self, f: F) -> FlatMap<Self, F>
    where
        F: FnMut(Self::Output) -> P,
        P: Parser,
    {
        FlatMap { parser: self, f }
    }

    fn zip_left<P>(self, parser: P) -> ZipLeft<Self, P> {
        ZipLeft {
            left: self,
            right: parser,
        }
    }
    fn zip_right<P>(self, parser: P) -> ZipRight<Self, P> {
        ZipRight {
            left: self,
            right: parser,
        }
    }

    fn until<P>(self, parser: P) -> Until<Self, P> {
        Until {
            parser: self,
            until: parser,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Until<P, Q> {
    parser: P,
    until: Q,
}

impl<P, Q> Parser for Until<P, Q>
where
    P: Parser,
    Q: Parser,
{
    type Output = Vec<P::Output>;

    fn parse<'s>(&mut self, mut input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        if input.is_empty() {
            return Err(Error);
        }

        let mut parsed = vec![];
        while let Err(..) = self.until.parse(input) {
            let (c, rest) = self.parser.parse(input)?;
            parsed.push(c);
            input = rest;
        }
        Ok((parsed, input))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipLeft<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Parser for ZipLeft<P, Q>
where
    P: Parser,
    Q: Parser,
{
    type Output = P::Output;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        let (parsed, rest) = self.left.parse(input)?;
        let (_, rest) = self.right.parse(rest)?;
        Ok((parsed, rest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipRight<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Parser for ZipRight<P, Q>
where
    P: Parser,
    Q: Parser,
{
    type Output = Q::Output;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        let (_, rest) = self.left.parse(input)?;
        let (parsed, rest) = self.right.parse(rest)?;
        Ok((parsed, rest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatMap<P, F> {
    parser: P,
    f: F,
}

impl<P, Q, F> Parser for FlatMap<P, F>
where
    P: Parser,
    F: FnMut(P::Output) -> Q,
    Q: Parser,
{
    type Output = Q::Output;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        let (parsed, rest) = self.parser.parse(input)?;
        (self.f)(parsed).parse(rest)
    }
}

// pub fn zip<'s, P, Q>(
//     mut left: P,
//     mut right: Q,
//     input: &'s str,
// ) -> Result<((P::Output, &'s str), (Q::Output, &'s str)), Error>
// where
//     P: Parser,
//     Q: Parser,
// {
//     left.parse(input)
//         .into_iter()
//         .zip(right.parse(input))
//         .next()
//         .ok_or(Error)
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<P, F> {
    parser: P,
    f: F,
}

impl<P, F, T> Parser for Map<P, F>
where
    P: Parser,
    F: FnMut(P::Output) -> T,
{
    type Output = T;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        let (parsed, rest) = self.parser.parse(input)?;
        Ok(((self.f)(parsed), rest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Or<P, Q> {
    first: P,
    second: Q,
}

impl<P, Q> Parser for Or<P, Q>
where
    P: Parser,
    Q: Parser,
{
    type Output = Either<P::Output, Q::Output>;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        if let Ok((parsed, rest)) = self.first.parse(input) {
            Ok((Either::A(parsed), rest))
        } else {
            let (parsed, rest) = self.second.parse(input)?;
            Ok((Either::B(parsed), rest))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub trait Get<T> {
    fn get(self) -> T;
}

impl<T> Get<T> for T {
    fn get(self) -> T {
        self
    }
}

impl<A, B> Get<B> for Either<A, B>
where
    A: Get<B>,
    B: Get<B>,
{
    #[inline]
    fn get(self) -> B {
        match self {
            Either::A(a) => a.get(),
            Either::B(b) => b.get(),
        }
    }
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn character(c: char) -> impl Parser<Output = char> {
    from_fn(move |input| {
        input.chars().next().map_or(Err(Error), |ch| {
            if ch == c {
                Ok((c, &input[1..]))
            } else {
                Err(Error)
            }
        })
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn many<P: Parser>(mut parser: P) -> impl Parser<Output = Vec<P::Output>> {
    from_fn(move |mut input| {
        // if input.is_empty() {
        //     return Err(Error);
        // }

        let mut parsed = vec![];
        while let Ok((ch, rest)) = parser.parse(input) {
            parsed.push(ch);
            input = rest;
        }
        Ok((parsed, input))
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn any() -> impl Parser<Output = char> {
    from_fn(|input| {
        input
            .chars()
            .next()
            .map_or(Err(Error), |c| Ok((c, &input[1..])))
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn whitespace() -> impl Parser<Output = ()> {
    character(' ')
        .or(character('\n'))
        .or(character('\t'))
        .map(|_| ())
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn one_of(chars: &str) -> impl Parser<Output = char> + '_ {
    from_fn(move |input| {
        if chars.is_empty() {
            return Err(Error);
        }

        input.chars().next().map_or(Err(Error), |c| {
            if chars.contains(c) {
                Ok((c, &input[1..]))
            } else {
                Err(Error)
            }
        })
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn range(r: RangeInclusive<char>) -> impl Parser<Output = char> {
    from_fn(move |input| {
        if r.is_empty() {
            return Err(Error);
        }

        input.chars().next().map_or(Err(Error), |c| {
            if r.contains(&c) {
                Ok((c, &input[1..]))
            } else {
                Err(Error)
            }
        })
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromFn<F> {
    f: F,
}

impl<T, F> Parser for FromFn<F>
where
    F: FnMut(&str) -> Result<(T, &str), Error>,
{
    type Output = T;

    fn parse<'s>(&mut self, input: &'s str) -> Result<(Self::Output, &'s str), Error> {
        (self.f)(input)
    }
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn from_fn<F, T>(f: F) -> FromFn<F>
where
    F: FnMut(&str) -> Result<(T, &str), Error>,
{
    FromFn { f }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_until() {
        let mut parser = any().until(character('!'));
        let (parsed, rest) = parser.parse("hello!").unwrap();
        assert_eq!(parsed, &['h', 'e', 'l', 'l', 'o']);
        assert_eq!(rest, "!");
        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_zip_left() {
        let mut parser = character('a').zip_left(character('b'));

        let (parsed, rest) = parser.parse("ab").unwrap();
        assert_eq!(parsed, 'a');
        assert_eq!(rest, "");
        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_zip_right() {
        let mut parser = character('a').zip_right(character('b'));

        let (parsed, rest) = parser.parse("ab").unwrap();
        assert_eq!(parsed, 'b');
        assert_eq!(rest, "");
        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_flat_map() {
        let mut parser = any().flat_map(|c| {
            assert_eq!(c, 'a');
            character('b')
        });

        let (parsed, rest) = parser.parse("ab").unwrap();
        assert_eq!(parsed, 'b');
        assert_eq!(rest, "");
        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_map() {
        let mut parser = character('a').map(|c| c.to_ascii_uppercase());

        let (parsed, rest) = parser.parse("a").unwrap();
        assert_eq!(parsed, 'A');
        assert_eq!(rest, "");
        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_or() {
        let mut parser = character('a').or(character('b'));

        let (parsed, rest) = parser.parse("a").unwrap();
        assert_eq!(parsed, Either::A('a'));
        assert_eq!(rest, "");

        let (parsed, rest) = parser.parse("b").unwrap();
        assert_eq!(parsed, Either::B('b'));
        assert_eq!(rest, "");

        assert_eq!(Err(Error), parser.parse(""));
    }

    #[test]
    pub fn test_character() {
        assert_eq!(Err(Error), parse(character('2'), "12"));

        let (c, rest) = character('1').parse("12").unwrap();
        assert_eq!(('1', "2"), (c, rest));
        assert_eq!(Ok(('2', "")), character('2').parse(rest));

        assert_eq!(Err(Error), parse(character('2'), ""));
    }

    #[test]
    pub fn test_many() {
        let (parsed_ones, rest1) = many(character('1')).parse("1111222").unwrap();
        let (parsed_twos, rest2) = many(character('2')).parse(rest1).unwrap();
        assert_eq!(parsed_ones, &['1'; 4]);
        assert_eq!(rest1, "222");
        assert_eq!(parsed_twos, &['2'; 3]);
        assert_eq!(rest2, "");

        assert_eq!(Ok((vec![], "")), many(character('1')).parse(""));
    }

    #[test]
    pub fn test_any() {
        let input = "()";
        let (parsed, rest) = any().parse(input).unwrap();
        assert_eq!(parsed, '(');
        assert_eq!(rest, ")");

        let input = "";
        assert_eq!(Err(Error), any().parse(input));
    }

    #[test]
    pub fn test_whitespace() {
        let mut parser = many(whitespace());
        let (parsed, rest) = parser.parse("   \n    \tasdf").unwrap();
        assert_eq!(parsed, &[(); 9]);
        assert_eq!(rest, "asdf");
        assert_eq!(Ok((vec![], "")), parser.parse(""));
    }

    #[test]
    pub fn test_one_of() {
        let mut parser = many(one_of("123"));
        let (parsed, rest) = parser.parse("2231235").unwrap();
        assert_eq!(parsed, &['2', '2', '3', '1', '2', '3']);
        assert_eq!(rest, "5");

        assert_eq!(Ok((vec![], "")), parser.parse(""));
        assert_eq!(Err(Error), one_of("").parse("123"));
    }

    #[test]
    pub fn test_range() {
        let mut parser = many(range('a'..='z'));
        let (parsed, rest) = parser.parse("hello!").unwrap();
        assert_eq!(parsed, &['h', 'e', 'l', 'l', 'o']);
        assert_eq!(rest, "!");

        assert_eq!(Ok((vec![], "")), parser.parse(""));
        assert_eq!(Err(Error), range('a'..='a').parse("123"));
    }
}
