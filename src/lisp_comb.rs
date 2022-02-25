use crate::{
    parser_comb::{any, character, from_fn, many, range, whitespace, Error, Get, Parser},
    LispObject,
};

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn string() -> impl Parser<Output = String> {
    character('"')
        .flat_map(|_| any().until(character('"')))
        .zip_left(character('"'))
        .map(|s| s.into_iter().collect())
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn ident() -> impl Parser<Output = String> {
    from_fn(move |input| {
        let mut first = character('_')
            .or(range('a'..='z'))
            .or(range('A'..='Z'))
            .map(Get::get);
        let mut other = many(
            character('_')
                .or(range('a'..='z'))
                .or(range('A'..='Z'))
                .or(range('0'..='9'))
                .map(Get::get),
        );

        let (first_char, rest): (char, _) = first.parse(input)?;
        let (parsed, rest): (Vec<char>, _) = other.parse(rest)?;

        Ok((
            [vec![first_char], parsed].concat().into_iter().collect(),
            rest,
        ))
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn number() -> impl Parser<Output = i32> {
    from_fn(move |input| {
        let mut parser = many(range('0'..='9'));

        let (parsed, rest) = parser.parse(input)?;
        if let Ok(n) = parsed.into_iter().collect::<String>().parse() {
            Ok((n, rest))
        } else {
            Err(Error)
        }
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn lisp_string() -> impl Parser<Output = LispObject> {
    string().map(LispObject::String)
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn lisp_ident() -> impl Parser<Output = LispObject> {
    ident().map(LispObject::Ident)
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn lisp_object() -> impl Parser<Output = LispObject> {
    from_fn(move |input| {
        lisp_string()
            .or(lisp_ident())
            .or(lisp_list())
            .map(Get::get)
            .parse(input)
    })
}

#[must_use = "parsers do nothing unless passed to [`parse`]"]
pub fn lisp_list() -> impl Parser<Output = LispObject> {
    character('(')
        .zip_left(many(whitespace()))
        .zip_right(many(lisp_object().zip_left(many(whitespace()))))
        .zip_left(many(whitespace()))
        .zip_left(character(')'))
        .zip_left(many(whitespace()))
        .map(LispObject::List)
}

#[cfg(test)]
mod tests {
    use crate::parser_comb::Error;

    use super::*;

    #[test]
    fn test_name() {
        let (parsed, rest) = string().parse(r#""hello""#).unwrap();
        assert_eq!(parsed, "hello");
        assert_eq!(rest, "");
        assert_eq!(Err(Error), string().parse(""));
    }

    #[test]
    fn test_ident() {
        let (parsed, rest) = ident().parse("hello").unwrap();
        assert_eq!(parsed, "hello");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_number() {
        let (parsed, rest) = number().parse("123").unwrap();
        assert_eq!(parsed, 123);
        assert_eq!(rest, "");
        assert_eq!(Err(Error), number().parse(""));
        assert_eq!(Err(Error), number().parse("asd"));
    }

    #[test]
    fn test_lisp_string() {
        let (parsed, rest) = lisp_string().parse(r#""ayo""#).unwrap();
        assert_eq!(parsed, LispObject::String("ayo".into()));
        assert_eq!(rest, "");
        assert_eq!(Err(Error), lisp_string().parse(""));
    }

    #[test]
    fn test_lisp_ident() {
        let (parsed, rest) = lisp_ident().parse("foo").unwrap();
        assert_eq!(parsed, LispObject::Ident("foo".into()));
        assert_eq!(rest, "");
        assert_eq!(Err(Error), lisp_ident().parse(""));
    }

    #[test]
    fn test_lisp_list() {
        let (parsed, rest) = lisp_list().parse("()").unwrap();
        assert_eq!(parsed, LispObject::List(vec![]));
        assert_eq!(rest, "");

        assert_eq!(Err(Error), lisp_list().parse(""));
    }

    #[test]
    fn test_lisp() {
        use LispObject::*;

        let (parsed, rest) = lisp_object()
            .parse(r#"(asd ("asdasd" asd ("asd") asd) "asdasd" ())"#)
            .unwrap();

        assert_eq!(
            parsed,
            List(vec![
                Ident("asd".into()),
                List(vec![
                    String("asdasd".into()),
                    Ident("asd".into()),
                    List(vec![String("asd".into())]),
                    Ident("asd".into())
                ]),
                String("asdasd".into()),
                List(vec![])
            ])
        );
        assert_eq!(rest, "");
    }
}
