use std::collections::HashMap;
use std::iter::Peekable;

#[derive(Debug)]
enum GrammarItem {
    Product,
    Sum,
    Number(i64),
    Paren,
}

#[derive(Debug)]
struct ParseNode {
    children: Vec<ParseNode>,
    entry: GrammarItem,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            children: Vec::new(),
            entry: GrammarItem::Paren,
        }
    }
}
#[derive(Debug, Clone)]
enum LexItem {
    Word(String),
    OpenParen,
    CloseParen,
    Num(i64),
}

fn next_lexeme<T: Iterator<Item = char>>(mut it: &mut Peekable<T>) -> Option<LexItem> {
    let c = *(it.peek().unwrap());

    //println!("lex {}", c);

    match c {
        '0'...'9' | '-' => {
            it.next();
            let n = get_number(c, &mut it);

            return Some(LexItem::Num(n));
        }
        'A'...'Z' | 'a'...'z' => {
            it.next();
            let a = get_word(c, &mut it);
            return Some(LexItem::Word(a));
        }
        // '+' | '*' => {
        //     result.push(LexItem::Op(c));
        //     it.next();
        // }
        // '(' | ')' | '[' | ']' | '{' | '}' => {
        //     result.push(LexItem::Paren(c));
        //     it.next();
        // }
        ' ' | '\n' | '\t' => {
            it.next();
            return None;
        }
        '(' => {
            it.next();
            return Some(LexItem::OpenParen);
        }
        ')' => {
            it.next();
            return Some(LexItem::CloseParen);
        }
        _ => {
            return None;
        }
    }
}

fn get_number<T: Iterator<Item = char>>(inc: char, iter: &mut Peekable<T>) -> i64 {
    let mut sign = 1;
    let mut c = inc;
    if inc == '-' {
        sign = -1;
        c = iter.next().unwrap();
    }
    let mut number = c
        .to_string()
        .parse::<i64>()
        .expect("The caller should have passed a digit.");
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    //println!("get_number {}", number * sign);
    number * sign
}

fn get_word<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> String {
    let mut word = c.to_string();
    while let Some(&letter) = iter.peek() {
        if letter == ' ' || letter == ')' {
            break;
        }

        word.push(letter);
        iter.next();
    }
    //println!("get_word {}", word);
    word
}

fn print_lexeme(token: &LexItem) -> String {
    let mut value: String;
    match token {
        LexItem::Word(w) => {
            value = w.to_string();
        }
        LexItem::Num(n) => {
            value = n.to_string();
        }
        LexItem::OpenParen => {
            value = "(".to_string();
        }
        LexItem::CloseParen => {
            value = ")".to_string();
        } //LexItem::WhiteSpace => {descriptor = "WhiteSpace"; value = " ".to_string();}
    }
    return format!("{} ", value);
}

fn matching(c: char) -> char {
    match c {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("should have been a parenthesis!"),
    }
}

pub struct DataStack {
    stack: Vec<(Option<LexItem>, u8)>,
}
impl DataStack {
    fn push_data(self: &mut Self, l: LexItem) {
        let mut current_arity = self.peek_arity();
        if self.stack.len() > 0 {
            let (d, _) = &self.stack[self.stack.len() - 1];
            match d {
                None => {
                    self.stack.pop();
                }
                _ => (),
            }
        }
        if current_arity > 0 {
            current_arity = current_arity - 1
        }
        println!("push_data: {}, arity: {}", print_lexeme(&l), current_arity);
        self.stack.push((Some(l), current_arity));
    }
    fn push_arity(self: &mut Self, a: u8) {
        println!("push_arity : {}", a);
        self.stack.push((None, a));
    }
    fn pop_data(self: &mut Self) -> Option<(LexItem, u8)> {
        let mut data = self.stack.pop();
        if data.is_some() {
            loop {
                let (d, a) = data.unwrap();
                if d.is_some() {
                    return Some((d.unwrap(), a));
                }
                data = self.stack.pop();
                if data.is_none() {
                    break;
                }
            }
        }
        return None;
    }

    fn peek_arity(self: &Self) -> u8 {
        if self.stack.len() > 0 {
            let (l, arity) = &self.stack[self.stack.len() - 1];

            match l {
                Some(item) => {
                    //self.print();
                    println!("peek_arity: {}, arity:{}", print_lexeme(item), arity);
                    return *arity;
                }
                _ => {
                    return *arity;
                }
            }
        } else {
            return 0;
        }
    }
    fn print(self: &Self) -> () {
        for data in &self.stack {
            let (d, a) = data;
            match d {
                Some(lex) => {
                    println!("Data Value: {}, arity: {}", print_lexeme(&lex), a);
                }
                _ => {
                    println!("Data Value: EMPTY, arity: {}", a);
                }
            };
        }
    }
}

pub struct Word {
    arity: u8,
    action: fn(w: &Self, dstack: &mut DataStack) -> (),
    substitution: Option<Vec<LexItem>>,
}

use std::env;

impl Word {
    fn action_add(self: &Self, dstack: &mut DataStack) -> () {
        let (a, a_arity) = dstack.pop_data().expect("action_add: a not found");
        let (b, b_arity) = dstack.pop_data().unwrap();

        match a {
            LexItem::Num(num_a) => match b {
                LexItem::Num(num_b) => {
                    println!("action_add: {} + {} = {}", num_a, num_b, num_a + num_b);
                    dstack.push_data(LexItem::Num(num_a + num_b));
                }
                _ => (),
            },
            _ => (),
        }
    }
}
fn action_arity_0(w: &Word, dstack: &mut DataStack) -> () {}

fn main() {
    let mut words: HashMap<String, Word> = HashMap::new();
    let mut dstack = DataStack { stack: Vec::new() };
    let mut astack: Vec<String> = Vec::new();
    let string1 = String::from("add");
    let testword = Word {
        arity: 2,
        action: Word::action_add,
        substitution: None,
    };
    words.insert(string1, testword);
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);
        let mut it = (&args[1]).chars().peekable();
        while it.peek() != None {
            let lexeme = next_lexeme(&mut it);
            match &lexeme {
                Some(LexItem::Word(w)) => match words.get(w) {
                    Some(word) => {
                        println!("Found Word: {}", w);
                        astack.push(w.to_string());
                        dstack.push_arity(word.arity);
                    }
                    _ => {
                        println!("{},{}", "Word", w);
                    }
                },
                Some(LexItem::Num(n)) => {
                    dstack.push_data(LexItem::Num(*n));
                    //println!("{},{}","Num",n.to_string());
                    while dstack.peek_arity() == 0 {
                        match astack.pop() {
                            Some(fname) => {
                                println!("popped function {}", fname);
                                match words.get(&fname) {
                                    Some(word) => {
                                        (word.action)(word, &mut dstack);
                                    }
                                    _ => (),
                                }
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                }
                Some(LexItem::OpenParen) => {
                    println!("{},{}", "OpenParen", "(");
                }
                Some(LexItem::CloseParen) => {
                    println!("{},{}", "CloseParen", ")");
                }
                None => (),
            }
        }
    }

    dstack.print();

    // println!("Hello, world! {}", match words.stack.get("define") { Some(fun) => fun, None=> "none" });
}
