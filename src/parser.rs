use crate::ast;
use crate::lexer::{Cursor, Token, TokenValue};
use crate::ast::{PosExpression, SExpression};
use crate::lexer::TokenStream;
use crate::symbol::Sym;

#[derive(Debug)]
pub enum ParserError{
    UnexpectedToken(Cursor),
    NoMatchingParser(Cursor),
    UnclosedParenthesis,
    //UnclosedParenthesis(Cursor),
}

pub type ParserResult = Result<Option<ast::PosExpression>, ParserError>;
//type Parser = fn(&mut TokenStream) -> ParserResult;

fn parse_iter(stream: &mut TokenStream, acc: Vec<PosExpression>) -> Result<Vec<PosExpression>, ParserError>{
    let mut acc = acc;
    match parse_s_expression(stream) {
        Ok(Some(e)) => {acc.push(e); parse_iter(stream, acc)},
        Ok(None) => Ok(acc),
        Err(e) => Err(e),
    }
}
pub fn parse(stream: &mut TokenStream) -> Result<PosExpression, ParserError> {
    let stack = parse_iter(stream, Vec::new())?;
    match stream.peek().unwrap() {
        Token{value: TokenValue::EOF, ..} => Ok(PosExpression{exp: SExpression::Block(std::rc::Rc::new(stack)), cursor: Cursor::new()}),
        Token{value: _, cursor} => Err(ParserError::NoMatchingParser(cursor.clone())),
    }
}

fn parse_s_expression(stream: &mut TokenStream) -> ParserResult {
    let parsers = [parse_atomic, parse_list, parse_block];
    for parser in parsers{
        match parser(stream){
            Ok(Some(data)) => return Ok(Some(data)),
            Err(e) => return Err(e),
            Ok(None) => {/* attempt next parser */}
        }
    }
    Ok(None)
}

fn parse_atomic(stream: &mut TokenStream) -> ParserResult{
    let stream = stream;
    match stream.next_if(|token| matches!(token.value, TokenValue::NumericToken(_)) ||matches!(token.value, TokenValue::Identifier(_))) {
        Some(Token {value: TokenValue::Identifier(ident), cursor}) => {
            Ok(Some(PosExpression{cursor, exp: SExpression::Symbol(Sym::intern(&ident))}))
        },
        Some(Token{value: TokenValue::NumericToken(i), cursor}) => {
            Ok(Some(PosExpression{cursor, exp: SExpression::Number(i)}))
        }
        _ => { Ok(None) }
    }
}

fn parse_list_iter(stream: &mut TokenStream, acc: Vec<ast::PosExpression>) -> Result<Vec<ast::PosExpression>, ParserError> {
    let mut acc = acc;
    match parse_s_expression(stream){
        Ok(Some(exp)) => {
            acc.push(exp);
            parse_list_iter(stream,acc)
        },
        Ok(None) => Ok(acc),
        Err(e) => Err(e)
    }
}

fn parse_list(stream: &mut TokenStream) -> ParserResult{
    match parse_listy(stream, TokenValue::ParenthesisOpen, TokenValue::ParenthesisClose)? {
        None => Ok(None),
        Some((acc, cursor)) => Ok(Some(PosExpression{exp: ast::SExpression::List(std::rc::Rc::new(acc)), cursor})),
    }
}

fn parse_block(stream: &mut TokenStream) -> ParserResult{
    match parse_listy(stream, TokenValue::BracketOpen, TokenValue::BracketClose)? {
        None => Ok(None),
        Some((acc, cursor)) => Ok(Some(PosExpression{cursor, exp: ast::SExpression::Block(std::rc::Rc::new(acc))})),
    }
}

fn parse_listy(stream: &mut TokenStream, open: TokenValue, close: TokenValue) -> Result<Option<(Vec<PosExpression>, Cursor)>, ParserError>{
    if stream.peek().unwrap().value != open {
        return Ok(None);
    }
    stream.next(); //discard open
    let inner = parse_list_iter(stream, Vec::new())?;
    if stream.peek().unwrap().value != close {
        return Err(ParserError::UnclosedParenthesis); //TODO: not generic enough
    }
    stream.next(); //discard close
    Ok(Some((inner, Cursor::new())))
}