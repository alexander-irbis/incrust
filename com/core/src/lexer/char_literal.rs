use nom::{
    Context::*,
    Err::*,
    ErrorKind::*,
    types::CompleteByteSlice as Slice,
};

use crate::container::pst::{self, ErrorKind::*};

pub fn char_literal(input: Slice) -> nom::IResult<Slice, pst::CharLiteral, pst::ErrorKind> {
    let (next, _) = complete!(input, char!('\''))
        .map_err(|_| Error(Code(input, Custom(NotRecognized))))?;

    let (next, string) = recognize!(next, check_char)
        .map_err(|_| Failure(Code(input, Custom(IncorrectCharLiteral))))?;

    let (output, _) = char!(next, '\'')
        .map_err(|_| Failure(Code(input, Custom(UnclosedCharLiteral))))?;

    Ok((output, pst::CharLiteral(&string[..])))
}

fn check_char(input: Slice) -> nom::IResult<Slice, (), pst::ErrorKind> {
    // TODO number encoded symbols, e.g. \x00
    let (output, _) = fold_many0!(input,
        alt!(
            is_not!(br#"'\"#) |
            alt!(
                tag!(r#"\\"#) |
                tag!(r#"\'"#) |
                tag!(r#"\n"#) |
                tag!(r#"\r"#) |
                tag!(r#"\t"#) |
                tag!(r#"\"#)
            )
        ),
        (),
        |_a, _i| ()
    )
        .map_err(|_| Failure(Code(input, Custom(IncorrectCharLiteral))))?;

    Ok((output, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &[u8] = &[];

    fn good(sample: &str) {
        let sample = Slice(sample.as_bytes());
        assert_eq!(
            Ok((Slice(EMPTY), pst::CharLiteral(&sample[1..sample.len() - 1]))),
            char_literal(sample),
        );
    }

    fn not_recognized(sample: &str) {
        let sample = Slice(sample.as_bytes());
        assert_eq!(
            Err(Error(Code(sample, Custom(NotRecognized)))),
            char_literal(sample),
        );
    }

    fn _incorrect(sample: &str) {
        let sample = Slice(sample.as_bytes());
        assert_eq!(
            Err(Failure(Code(sample, Custom(IncorrectCharLiteral)))),
            char_literal(sample),
        );
    }

    fn unclosed(sample: &str) {
        let sample = Slice(sample.as_bytes());
        assert_eq!(
            Err(Failure(Code(sample, Custom(UnclosedCharLiteral)))),
            char_literal(sample),
        );
    }

    #[test]
    fn test() {
        // NB: the character itself is parsed in the next stage, so `'string'` is good now.

        good(r#"''"#);
        good(r#"' '"#);
        good(r#"'string'"#);
        good(r#"'a\nb'"#);
        good(r#"'0'"#);
        good(r#"'0.0'"#);
        good(r#"'a\rb'"#);
        good(r#"'a\tb'"#);
        good(r#"'a\\b'"#);
        good(r#"'a\'b'"#);
        good(r#"'a"b'"#);
        good(r#"'\\'"#);
        good(r#"'\ \\'"#);
        good(r#"'{{ expression }}'"#);
        good(r#"'{% statement %}'"#);
        good(r#"'{# comment #}'"#);

        not_recognized(r#""#);
        not_recognized(r#""""#);
        not_recognized(r#"plain text"#);
        not_recognized(r#"{{ expression }}"#);
        not_recognized(r#"\'"#);
        not_recognized(r#"\''"#);
        not_recognized(r#"\'\'"#);

        unclosed(r#"'"#);
        unclosed(r#"'\'"#);
        unclosed(r#"'\\\'"#);
        unclosed(r#"'with text"#);
        unclosed(r#"'with text\'"#);
    }
}