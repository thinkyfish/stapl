use std::collections::HashMap;
use std::env;
use std::fmt;
use std::iter::Peekable;

#[derive(Debug, Clone)]
enum LexItem {
    Word(String),
    OpenParen,
    CloseParen,
    Num(i64),
    Parameter(i64),
    Stack(Vec<LexItem>),
    Lambda(Vec<LexItem>),
}

impl LexItem {
    fn get_Num(self: &Self) -> Option<i64> {
        match self {
            LexItem::Num(n) => {
                return Some(*n);
            }
            _ => {
                return None;
            }
        }
    }
    fn get_Word(self: &Self) -> Option<String> {
        match self {
            LexItem::Word(w) => {
                return Some(w.to_string());
            }
            _ => {
                return None;
            }
        }
    }
}

fn next_lexeme<T: Iterator<Item = char>>(mut it: &mut Peekable<T>) -> Option<LexItem> {
    let c = *(it.peek().unwrap());

    //println!("lex {}", c);

    match c {
        '0'...'9' | '-' => {
            it.next();
            let n = lex_number(c, &mut it);

            return Some(LexItem::Num(n));
        }
        'A'...'Z' | 'a'...'z' | '+' => {
            it.next();
            let a = lex_word(c, &mut it);
            return Some(LexItem::Word(a));
        }
        '$' => {
            it.next();
            let p = lex_parameter(c, &mut it);
            return Some(LexItem::Parameter(p));
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

fn lex_parameter<T: Iterator<Item = char>>(inc: char, iter: &mut Peekable<T>) -> i64 {
    let mut number = 0;
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    return number;
}

fn lex_number<T: Iterator<Item = char>>(inc: char, iter: &mut Peekable<T>) -> i64 {
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

fn lex_word<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> String {
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
        LexItem::Stack(s) => {
            value = "[stack]".to_string();
        }
        LexItem::Lambda(l) => {
            value = "[Lambda]".to_string();
        }
        LexItem::Parameter(p) => {
            value = format!("${}", p);
        }
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

#[derive(Copy, Clone)]
enum Expectation {
    Num,
    Literal,
    Stack,
    Any,
}

fn check_expectation(e: Expectation, l: LexItem) -> Option<LexItem> {
    match e {
        Expectation::Num => match l {
            LexItem::Num(n) => {
                return Some(l);
            }

            _ => (),
        },
        Expectation::Any => {
            return Some(l);
        }
        _ => (),
    }
    return None;
}

struct Call {
    name: String,
    action: fn(call: &mut Call) -> (),
    arity: u8,
    arguments: Vec<LexItem>,
    expectations: Vec<Expectation>,
    results: Vec<LexItem>,
}
impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, a:{:?}}}", self.name, self.arguments)
    }
}
struct CallStack {
    stack: Vec<Call>,
}
impl CallStack {
    fn pushWordCall(self: &mut Self, word: &Word) -> () {
        self.stack.push(make_call(word));
        println!("pushed Call: {}, Stack: {:?}", word.name, self.stack);
    }
    fn len(self: &Self) -> usize {
        return self.stack.len();
    }
    fn getExpectation(self: &Self) -> Option<&Expectation> {
        let top_call: &Call;
        match self.stack.last() {
            Some(call) => {
                top_call = call;
            }
            None => {
                return None;
            }
        };

        return top_call.expectations.last();
    }

    fn pushLexItem(self: &mut Self, lexeme: LexItem) -> bool {
        let top_call: &mut Call;
        match self.stack.last_mut() {
            Some(call) => {
                top_call = call;
            }
            None => {
                return false;
            }
        };
        let e = &mut top_call.expectations;
        match e.last() {
            Some(&top_expectation) => {
                let d = check_expectation(top_expectation, lexeme);
                match d {
                    Some(dataitem) => {
                        e.pop();
                        top_call.arguments.push(dataitem);
                        //could put apply here
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        println!(
            "pushed item: {}, Stack: {:?}",
            print_lexeme(top_call.arguments.last().unwrap()),
            self.stack
        );
        return true;
    }

    fn wantsData(self: &Self) -> bool {
        match self.stack.last() {
            Some(call) => {
                println!("wants data: {}", call.expectations.len());
                return call.expectations.len() > 0;
            }
            None => {
                return false;
            }
        };
    }
    fn top_apply(self: &mut Self, result: &mut Vec<LexItem>) -> bool {
        match self.stack.last_mut() {
            Some(top_call) => {
                (top_call.action)(top_call);
                result.append(&mut top_call.results);
                self.stack.pop();
                return true;
            }
            _ => {
                return false;
            }
        }
    }
}

struct Word {
    name: String,
    arity: u8,
    action: fn(call: &mut Call) -> (),
    substitution: Option<Vec<LexItem>>,
    expectations: Vec<Expectation>,
}

fn make_word(name: String, arity: u8, substitution: Option<Vec<LexItem>>) -> Word {
    let word = Word {
        name: name,
        arity: arity,
        expectations: Vec::new(),
        action: action_add,
        substitution: substitution,
    };

    return word;
}

fn action_add(call: &mut Call) -> () {
    println!("action_add arguments {:?}", call.arguments);
    let a = call.arguments.pop().unwrap().get_Num().unwrap();
    let b = call.arguments.pop().unwrap().get_Num().unwrap();
    call.results.push(LexItem::Num(a + b));
    println!("action_add {} + {}", a, b);
}
fn check_conditional(l: LexItem) -> bool {
    match l {
        LexItem::Num(n) => {
            if n == 0 {
                return false;
            } else {
                return true;
            }
        }
        _ => return false,
    }
}
fn action_if(call: &mut Call) -> () {
    println!("action_if arguments {:?}", call.arguments);
    let else_clause = call.arguments.pop().unwrap();
    let if_clause = call.arguments.pop().unwrap();
    let conditional = call.arguments.pop().unwrap();
    if check_conditional(conditional) {
        call.results.push(if_clause);
    } else {
        call.results.push(else_clause);
    }
}
fn make_call(word: &Word) -> Call {
    let a = Call {
        name: "+".to_string(),
        action: word.action,
        arity: word.arity,
        arguments: Vec::new(),
        expectations: word.expectations.to_vec(),
        results: Vec::new(),
    };
    return a;
}

fn main() {
    let mut words: HashMap<String, Word> = HashMap::new();
    let cstack = &mut CallStack { stack: Vec::new() };
    let istack: &mut Vec<LexItem> = &mut Vec::new();
    let ostack: &mut Vec<LexItem> = &mut Vec::new();

    //add '+' builtin
    let string1 = String::from("+");
    let addword = Word {
        name: "+".to_string(),
        arity: 2,
        action: action_add,
        substitution: None,
        expectations: vec![Expectation::Num, Expectation::Num],
    };
    words.insert(string1, addword);

    let string2 = String::from("if");
    let ifword = Word {
        name: "if".to_string(),
        arity: 3,
        action: action_if,
        substitution: None,
        expectations: vec![Expectation::Num, Expectation::Any, Expectation::Any],
    };
    words.insert(string2, ifword);

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);

        //lex all of the input
        let mut it = (&args[1]).chars().peekable();
        while it.peek() != None {
            match next_lexeme(&mut it) {
                Some(lexeme) => {
                    istack.insert(0, lexeme);
                }
                _ => (),
            }
        }

        println!("istack: {:?}", istack);
        while istack.len() > 0 || cstack.len() > 0 {
            println!("istack: {:?}", istack);
            while !cstack.wantsData() {
                if cstack.len() > 0 {
                    let result = cstack.top_apply(istack);
                } else {
                    break;
                };
            }

            match istack.pop().unwrap() {
                LexItem::Word(w) => match words.get(&w) {
                    Some(word) => {
                        cstack.pushWordCall(word);
                    }
                    _ => {
                        ostack.push(LexItem::Word(w));
                        println!("{},{}", "Word", ostack.last().unwrap().get_Word().unwrap());
                    }
                },
                LexItem::Num(n) => {
                    if cstack.pushLexItem(LexItem::Num(n)) {
                        println!("pushed item: {}", print_lexeme(&LexItem::Num(n)));
                    } else {
                        ostack.push(LexItem::Num(n));
                    }
                }
                LexItem::OpenParen => {
                    println!("{},{}", "OpenParen", "(");
                }
                LexItem::CloseParen => {
                    println!("{},{}", "CloseParen", ")");
                }
                _ => (),
            }
        }
    }
    println!("ostack: {:?}", ostack);
    println!("The first argument is {}", args[1]);
    // println!("Hello, world! {}", match words.stack.get("define") { Some(fun) => fun, None=> "none" });
}
