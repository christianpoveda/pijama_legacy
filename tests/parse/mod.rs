use std::include_str;

use pijama::ast::{self, BinOp::*, Node::*, UnOp};
use pijama::parser::parse;
use pijama::ty::{Binding, Ty};
use pijama::LangResult;

#[test]
fn name() -> LangResult<()> {
    let input = include_str!("name.pj");
    let result = parse(input)?;
    let expected = vec![
        Name(ast::Name("x")),
        Name(ast::Name("foo")),
        Name(ast::Name("foo_bar")),
    ];

    assert_eq!(expected[0], result[0], "single letter");
    assert_eq!(expected[1], result[1], "word");
    assert_eq!(expected[2], result[2], "snake case");
    Ok(())
}

#[test]
fn literal() -> LangResult<()> {
    let input = include_str!("literal.pj");
    let result = parse(input)?;
    let expected = vec![
        Literal(ast::Literal::Number(0)),
        Literal(ast::Literal::Number(-1)),
        Literal(ast::Literal::Number(14142)),
        Literal(ast::Literal::True),
        Literal(ast::Literal::False),
        Literal(ast::Literal::Unit),
    ];

    assert_eq!(expected[0], result[0], "integer");
    assert_eq!(expected[1], result[1], "negative integer");
    assert_eq!(expected[2], result[2], "large integer");
    assert_eq!(expected[3], result[3], "true");
    assert_eq!(expected[4], result[4], "false");
    assert_eq!(expected[5], result[5], "unit");
    Ok(())
}

#[test]
fn binary_op() -> LangResult<()> {
    let input = include_str!("bin_op.pj");
    let result = parse(input)?;
    let expected = vec![
        BinaryOp(Plus, box Name(ast::Name("a")), box Name(ast::Name("b"))),
        BinaryOp(
            Plus,
            box BinaryOp(Plus, box Name(ast::Name("a")), box Name(ast::Name("b"))),
            box Name(ast::Name("c")),
        ),
        BinaryOp(
            Plus,
            box Name(ast::Name("a")),
            box BinaryOp(Plus, box Name(ast::Name("b")), box Name(ast::Name("c"))),
        ),
    ];

    assert_eq!(expected[0], result[0], "simple");
    assert_eq!(expected[1], result[1], "left-associative");
    assert_eq!(expected[2], result[2], "brackets");
    Ok(())
}

#[test]
fn unary_op() -> LangResult<()> {
    let input = include_str!("un_op.pj");
    let result = parse(input)?;
    let expected = vec![
        UnaryOp(UnOp::Minus, box Name(ast::Name("x"))),
        UnaryOp(UnOp::Not, box Name(ast::Name("x"))),
        UnaryOp(UnOp::Not, box UnaryOp(UnOp::Not, box Name(ast::Name("x")))),
        UnaryOp(UnOp::Not, box Name(ast::Name("x"))),
    ];

    assert_eq!(expected[0], result[0], "minus");
    assert_eq!(expected[1], result[1], "not");
    assert_eq!(expected[2], result[2], "double");
    assert_eq!(expected[3], result[3], "brackets");
    Ok(())
}

#[test]
fn let_bind() -> LangResult<()> {
    let input = include_str!("let_bind.pj");
    let result = parse(input)?;
    let expected = vec![
        LetBind(ast::Name("x"), box Name(ast::Name("y"))),
        LetBind(
            ast::Name("x"),
            box BinaryOp(Plus, box Name(ast::Name("y")), box Name(ast::Name("z"))),
        ),
    ];

    assert_eq!(expected[0], result[0], "simple");
    assert_eq!(expected[1], result[1], "bind to bin op");
    Ok(())
}

#[test]
fn cond() -> LangResult<()> {
    let input = include_str!("cond.pj");
    let result = parse(input)?;
    let expected = vec![
        Cond(
            vec![Name(ast::Name("x"))],
            vec![Name(ast::Name("y"))],
            vec![Name(ast::Name("z"))],
        ),
        Cond(
            vec![Name(ast::Name("u")), Name(ast::Name("v"))],
            vec![Name(ast::Name("w")), Name(ast::Name("x"))],
            vec![Name(ast::Name("y")), Name(ast::Name("z"))],
        ),
    ];

    assert_eq!(expected[0], result[0], "simple blocks");
    assert_eq!(expected[1], result[1], "long blocks");
    Ok(())
}

#[test]
fn call() -> LangResult<()> {
    let input = include_str!("call.pj");
    let result = parse(input)?;
    let expected = vec![
        Call(ast::Name("x"), vec![]),
        Call(ast::Name("x"), vec![Name(ast::Name("y"))]),
        Call(
            ast::Name("x"),
            vec![Name(ast::Name("y")), Name(ast::Name("z"))],
        ),
    ];

    assert_eq!(expected[0], result[0], "nullary call");
    assert_eq!(expected[1], result[1], "unary call");
    assert_eq!(expected[2], result[2], "binary call");
    Ok(())
}

#[test]
fn fn_def() -> LangResult<()> {
    let input = include_str!("fn_def.pj");
    let result = parse(input)?;
    let expected = vec![
        FnDef(ast::Name("foo"), vec![], vec![], None),
        FnDef(
            ast::Name("foo"),
            vec![Binding {
                name: ast::Name("x"),
                ty: Ty::Int,
            }],
            vec![Name(ast::Name("x"))],
            None,
        ),
        FnDef(
            ast::Name("foo"),
            vec![],
            vec![Call(ast::Name("foo"), vec![])],
            Some(Ty::Unit),
        ),
        FnDef(
            ast::Name("foo"),
            vec![
                Binding {
                    name: ast::Name("x"),
                    ty: Ty::Int,
                },
                Binding {
                    name: ast::Name("y"),
                    ty: Ty::Int,
                },
            ],
            vec![Name(ast::Name("x")), Name(ast::Name("y"))],
            None,
        ),
    ];

    assert_eq!(expected[0], result[0], "nullary def");
    assert_eq!(expected[1], result[1], "unary def");
    assert_eq!(expected[2], result[2], "recursive def");
    assert_eq!(expected[3], result[3], "long body");
    Ok(())
}