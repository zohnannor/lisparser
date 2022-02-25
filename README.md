# Lisp syntax parser combinator

```rust
use lisparser::LispObject::{*, self};
use lisparser::lisp_comb::lisp_object;
use lisparser::{parse, Parser};

let parsed = parse(
    lisp_object(),
    r#"(asd ("asdasd" asd ("asd") asd) "asdasd" ())"#,
)
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
```

TODO:

- [ ] Keywords
- [ ] Other types
- [ ] Documentation
- [ ] More tests?
- [ ] Error reporting
- [ ] Lexer/Parser variant + benches

## Credits

Inspired by [davidpdrsn/json-parser](https://github.com/davidpdrsn/json-parser)

See also:

- [Parser combinator](https://en.wikipedia.org/wiki/Parser_combinator) on wiki.
- [nom](https://lib.rs/nom) - parser combinator library for rust
