use std::iter::Peekable;
use crate::value::numeric::Numeric;

pub mod langchars {
    pub const COMMENT: char= ';';
    pub const PARENTHESIS_OPEN: char = '(';
    pub const PARENTHESIS_CLOSE: char = ')';
    pub const BRACKET_OPEN: char = '[';
    pub const BRACKET_CLOSE: char = ']';
    //const WHITESPACES: [char; 2] = [' ', '\t'];
    pub const SPACE: char = ' ';
    pub const TAB: char = '\t';
    pub const NEW_LINE: char = '\n';

    //disallowed in identifiers
    pub const NON_IDENTIFIER_CHARS: [char; 7] = [PARENTHESIS_OPEN, PARENTHESIS_CLOSE, BRACKET_OPEN, BRACKET_CLOSE, SPACE, TAB, NEW_LINE];
}

#[derive(Clone, Debug)]
pub struct Cursor{
    line: usize,
    column: usize,
    abs_position: usize,
    _reach: Option<usize>,
}

impl Cursor{
    pub(crate) fn new() -> Cursor {
        Cursor{abs_position: 0, line:1, column: 1, _reach: None}
    }
    fn next_column(&self) -> Cursor {
        self.next_columns(1)
    }
    fn next_columns(&self, n: usize) -> Cursor {
        let mut clone = self.clone();
        clone.column+=n;
        clone.abs_position +=n;
        clone
    }

    fn next_line(&self) -> Cursor{
        let mut clone = self.clone();
        clone.line+=1;
        clone.column=1;
        clone.abs_position +=1;
        clone
    }
}

pub struct Lexer<'a>{
    txt_buffer: &'a str,
}
/*
pub enum Keyword{

}
 */

#[derive(Debug, PartialEq)]
pub enum TokenValue{
    Identifier(String),
    NumericToken(Numeric),
    //IntToken(i32),
    //StringLiteral(String),
    ParenthesisOpen,
    ParenthesisClose,
    BracketOpen,
    BracketClose,
    EOF,
}

#[derive(Debug)]
pub struct Token{
    pub cursor: Cursor,
    pub value: TokenValue,
}

pub struct TokenIterator<'a> {
    lexer: Lexer<'a>,
    cursor: Cursor
}


impl<'t> Lexer<'t>{
    pub fn from_file_path(_path: &str) -> Lexer {
        todo!()
    }

    pub fn from_text(data: &str) -> Lexer {
        Lexer{txt_buffer: data}
    }

    // `abs_position` is a byte offset into the buffer; slicing from it is O(1)
    // instead of the O(n) `chars().nth(i)` walk from the start of the text.
    fn char_at(&self, i: usize) -> Option<char> {
        self.txt_buffer.get(i..)?.chars().next()
    }

    fn char_at_cursor(&self, cursor: &Cursor) -> Option<char>{
        self.char_at(cursor.abs_position)
    }

    fn is_identifier_char(c: char) -> bool {
        //kind an anti-pattern but yolo
        !langchars::NON_IDENTIFIER_CHARS.contains(&c)
    }

    fn possible_identifier_upgrade(input: &String) -> Option<TokenValue> {
        //TODO: this is AWFUL, refactor when BigInt is added
        input.parse::<i32>()
            .map(|v| Some(TokenValue::NumericToken(Numeric::Integer(v)))).unwrap_or_else(
                |_| input.parse::<f64>()
                    .map(|v| Some(TokenValue::NumericToken(Numeric::Floating(v))))
                    .unwrap_or(None)
        )
    }

    fn read_identifier(&self, cursor: &Cursor) -> (TokenValue, Cursor) {
        let ident = self.txt_buffer
            .get(cursor.abs_position..)
            .unwrap_or("")
            .chars()
            .take_while(|c| Lexer::is_identifier_char(*c))
            .collect::<String>()
            ;
        let len = ident.len(); // bytes: abs_position is a byte offset
        (TokenValue::Identifier(ident),cursor.next_columns(len))
    }

    pub fn skip_comment(&self, start: &Cursor) -> Cursor {
        let mut cursor = start.clone();
        if let Some(c) = self.char_at_cursor(&cursor) {
            if c != langchars::COMMENT{ panic!("not a comment start");}
        }
        cursor = cursor.next_column();
        while let Some(char) = self.char_at_cursor(&cursor){
            if char == langchars::NEW_LINE{
                break;
            }
            // comments may contain multi-byte characters
            cursor = cursor.next_columns(char.len_utf8());
        }
        cursor
    }

    pub fn next_token(&self, cursor: &Cursor) -> (Token, Cursor){
        if let Some(char) = self.char_at_cursor(&cursor){
            match char{
                langchars::SPACE | langchars::TAB => { self.next_token(&cursor.next_column()) }
                langchars::NEW_LINE => { self.next_token(&cursor.next_line()) }
                langchars::PARENTHESIS_OPEN => { (Token{cursor: cursor.next_column(), value: TokenValue::ParenthesisOpen}, cursor.next_column()) }
                langchars::PARENTHESIS_CLOSE => { (Token{cursor: cursor.next_column(), value: TokenValue::ParenthesisClose}, cursor.next_column()) }
                langchars::BRACKET_OPEN => { (Token{cursor: cursor.next_column(), value: TokenValue::BracketOpen}, cursor.next_column()) }
                langchars::BRACKET_CLOSE => { (Token{cursor: cursor.next_column(), value: TokenValue::BracketClose}, cursor.next_column()) }
                langchars::COMMENT => {self.next_token(&self.skip_comment(cursor))}
                _ => {
                    let (ident_token, after_cursor) = self.read_identifier(cursor);
                    let TokenValue::Identifier(i) = ident_token else {panic!("didn't receive identifier")};
                    let value = Lexer::possible_identifier_upgrade(&i).unwrap_or(TokenValue::Identifier(i));
                    (
                        Token{
                        cursor: Cursor{ _reach: Some(after_cursor.abs_position-cursor.abs_position), ..*cursor},
                        value
                    },
                        after_cursor
                    )
                }
            }
        }
        else{
            (Token{cursor: cursor.clone(), value: TokenValue::EOF}, cursor.clone())
        }
    }
}

impl<'t> Iterator for TokenIterator<'t>{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let (token, next_cursor) = self.lexer.next_token(&self.cursor);
        self.cursor = next_cursor;
        Some(token)
    }
}

impl <'t>IntoIterator for Lexer<'t>{
    type Item = Token;
    type IntoIter = Peekable<TokenIterator<'t>>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator{lexer: self, cursor: Cursor::new()}.peekable()
        //Peekable{TokenIterator{lexer: self, cursor: Cursor::new()}}
    }
}


pub type TokenStream<'a> = Peekable<TokenIterator<'a>>;