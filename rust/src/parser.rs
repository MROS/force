use crate::lexer::Token;
use crate::Bondee;
use crate::CategoryAttribute;
use crate::DataType;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
struct Field {
    datatype: DataType,
    name: String,
}

#[derive(Debug)]
enum Linkees {
    All,
    Names(String),
}

#[derive(Debug)]
enum Linkee {
    All,
    Name(String),
}

#[derive(Debug)]
struct Category {
    name: String,
    fields: Vec<Field>,
    link_to: Linkees, // 可以鏈接到的分類名稱
}

#[derive(Debug)]
struct Link {
    from: String,
    to: Linkee,
}

type Categories = HashMap<String, Category>;
type Links = HashMap<(String, Linkee), Link>;

#[derive(Debug)]
pub struct Force {
    categories: Categories,
    links: Links,
}

#[derive(Debug)]
pub enum ForceError {
    NonExpect { expect: Token, fact: Token },
    NoMeet { expect: String, fact: Token },
}

pub type ForceResult<T> = Result<T, ForceError>;

pub struct Parser {
    tokens: Vec<Token>,
    count: usize,
    cur: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        return Parser {
            count: 0,
            cur: tokens[0].clone(),
            tokens: tokens,
        };
    }
    fn advance(&mut self) {
        self.count += 1;
        self.cur = self.tokens[self.count].clone()
    }
    fn eat(&mut self, expect: Token) -> ForceResult<()> {
        if self.cur == expect {
            self.advance();
            Ok(())
        } else {
            Err(ForceError::NonExpect {
                expect,
                fact: self.cur.clone(),
            })
        }
    }
    fn get_identifier(&mut self) -> ForceResult<String> {
        let ret = if let Token::Identifier(id) = &self.cur {
            Ok(id.clone())
        } else {
            Err(ForceError::NonExpect {
                expect: Token::Identifier("某個識別子".to_owned()),
                fact: self.cur.clone(),
            })
        };
        if let Ok(_) = ret {
            self.advance();
        }
        ret
    }
    fn parse_bondee(&mut self) -> ForceResult<Bondee> {
        self.eat(Token::LeftSquareBracket)?;
        match self.cur.clone() {
            Token::Star => {
                self.advance();
                self.eat(Token::RightSquareBracket)?;
                Ok(Bondee::All)
            }
            Token::Identifier(name) => {
                let mut choices = vec![name.to_string()];
                self.advance();
                loop {
                    match self.cur {
                        Token::RightSquareBracket => {
                            break;
                        }
                        _ => {
                            self.eat(Token::Comma)?;
                            let name = self.get_identifier()?;
                            choices.push(name);
                        }
                    }
                }
                self.eat(Token::RightSquareBracket)?;
                Ok(Bondee::Choices(choices))
            }
            _ => Err(ForceError::NoMeet {
                expect: "* 或識別子".to_owned(),
                fact: self.cur.clone(),
            }),
        }
    }
    fn parse_datatype(&mut self) -> ForceResult<DataType> {
        match self.cur {
            Token::Number => {
                self.advance();
                Ok(DataType::Number)
            }
            Token::OneLine => {
                self.advance();
                Ok(DataType::OneLine)
            }
            Token::Text => {
                self.advance();
                match self.cur.clone() {
                    Token::Regex(regex) => {
                        self.advance();
                        Ok(DataType::Text(Some(regex)))
                    }
                    _ => Ok(DataType::Text(None)),
                }
            }
            Token::Bond => {
                self.advance();
                let bondee = self.parse_bondee()?;
                Ok(DataType::Bond(bondee))
            }
            // Token::TaggedBond => {}
            _ => Err(ForceError::NoMeet {
                expect: "型別".to_owned(),
                fact: self.cur.clone(),
            }),
        }
    }
    pub fn parse(&mut self) -> ForceResult<Force> {
        let categories = self.parse_categories()?;
        return Ok(Force {
            categories,
            links: HashMap::new(),
        });
    }
    fn parse_categories(&mut self) -> ForceResult<Categories> {
        let mut categories = HashMap::new();
        loop {
            if let Token::End = self.cur {
                break;
            } else {
                let category = self.parse_category()?;
                categories.insert(category.name.clone(), category);
            }
        }
        return Ok(categories);
    }
    fn parse_category(&mut self) -> ForceResult<Category> {
        let name = self.get_identifier()?;
        let mut category = Category {
            name,
            fields: Vec::new(),
            link_to: Linkees::All, // NOTE: 先設置爲 ALL ，之後解析鍵結時再修改
        };
        self.eat(Token::LeftCurlyBrace)?;
        loop {
            if let Token::RightCurlyBrace = self.cur {
                break;
            } else {
                let datatype = self.parse_datatype()?;
                let name = self.get_identifier()?;
                category.fields.push(Field { datatype, name });
            }
        }
        self.eat(Token::RightCurlyBrace)?;
        Ok(category)
    }
}
