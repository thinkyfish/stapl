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
        '[' => {
            it.next();
            return Some(LexItem::OpenParen);
        }
        ']' => {
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
        if letter == ' ' || letter == ']' {
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
            value = "[".to_string();
        }
        LexItem::CloseParen => {
            value = "]".to_string();
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

fn parse_stacks<'i>(
    lex_input: &mut Vec<LexItem>,
    parsed_input: &'i mut Vec<LexItem>,
) -> &'i mut Vec<LexItem> {
    while let Some(itop) = lex_input.pop() {
        match itop {
            LexItem::OpenParen => {
                //println!("openbracket found");
                let mut newstack = &mut Vec::new();
                newstack = parse_stacks(lex_input, newstack);
                parsed_input.insert(0, LexItem::Stack(newstack.to_vec()));
            }
            LexItem::CloseParen => {
                //println!("closedbracket found");

                return parsed_input;
            }
            _ => {
                //println!("lexeme found: {}", print_lexeme(&itop));
                parsed_input.insert(0, itop);
                let p = parse_stacks(lex_input, parsed_input);
                return p;
            }
        }
    }
    return parsed_input;
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

#[derive(Debug, Copy, Clone)]
enum Expectation {
    Num,
    Literal,
    Stack,
    Any,
    Word,
}
/* impl fmt::Debug for Expectation{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output;
        match self {
            Num => {output = "Num";},
            Literal => { output = "Literal"; },
            Stack => { output = "Stack"


        }
        write!(f, "{{{}, a:{:?}}}", self.name, self.arguments)
    }
} */

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
    //action: fn(call: &mut Call) -> (),
    arity: u8,
    arguments: Vec<LexItem>,
    expectations: Vec<Expectation>,
    results: Vec<LexItem>,
    substitution: Option<LexItem>,
}
impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, a:{:?}}}", self.name, self.arguments)
    }
}
struct CallStack {
    stack: Vec<Call>,
    words: HashMap<String, Word>,
}
impl CallStack {
    fn pushWordCall(self: &mut Self, word: String) -> () {
        match self.words.get(&word) {
            Some(w) => {
                self.stack.push(make_call(w));
                println!("pushed Call: {}, Stack: {:?}", w.name, self.stack);
            }
            None => {
                let defword = self.words.get(&"define".to_string()).unwrap();

                let mut c = make_call(defword);
                c.expectations.pop();
                c.arguments.push(LexItem::Word(word.to_string()));

                self.stack.push(c);

                println!("pushed Define {}, Stack: {:?}", word, self.stack);
            }
        }
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
    fn create_builtin_words(self: &mut Self) -> () {
        let string1 = String::from("+");
        let addword = Word {
            name: "+".to_string(),
            arity: 2,
            //action: action_add,
            substitution: None,
            expectations: vec![Expectation::Num, Expectation::Num],
        };
        self.words.insert(string1, addword);

        let string2 = String::from("if");
        let ifword = Word {
            name: "if".to_string(),
            arity: 3,
            //action: action_if,
            substitution: None,
            expectations: vec![Expectation::Any, Expectation::Any, Expectation::Num],
        };
        self.words.insert(string2, ifword);

        let defword = Word {
            name: "define".to_string(),
            arity: 1,
            //action: CallStack::action_define(self), // change this to an option?
            substitution: None,
            expectations: vec![Expectation::Any, Expectation::Word],
        };
        self.words.insert("define".to_string(), defword);
    }

    fn pushLexItem<'l>(self: &mut Self, lexeme: &'l mut LexItem) -> Option<&'l mut LexItem> {
        let top_call: &mut Call;
        match self.stack.last_mut() {
            Some(call) => {
                top_call = call;
            }
            None => {
                return Some(lexeme);
            }
        };
        let e = &mut top_call.expectations;
        match e.last() {
            Some(&top_expectation) => {
                let d = check_expectation(top_expectation, lexeme.clone());
                match d {
                    Some(dataitem) => {
                        e.pop();
                        top_call.arguments.insert(0, dataitem);
                        return None;
                        //could put apply here
                    }
                    None => {}
                }
            }
            None => {}
        }

        /* println!(
            "pushed item: {}, Stack: {:?}",
            print_lexeme(top_call.arguments.last().unwrap()),
            self.stack
        ); */
        return Some(lexeme);
    }

    fn wantsData(self: &Self) -> bool {
        match self.stack.last() {
            Some(call) => {
                println!(
                    "wants data: {:?}, {} more",
                    call.expectations.last(),
                    call.expectations.len()
                );
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
                match top_call.name.as_str() {
                    "+" => {
                        action_add(top_call);
                    }
                    "if" => {
                        action_if(top_call);
                    }
                    "define" => {
                        action_define(&mut self.words, top_call);
                    }
                    _ => {
                        action_substitution(top_call);
                    }
                }
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
    //action: fn(call: &mut Call) -> (),
    substitution: Option<LexItem>,
    expectations: Vec<Expectation>,
}

fn make_word(
    name: String,
    arity: u8,
    //  action: fn(call: &mut Call) -> (),
    substitution: Option<LexItem>,
) -> Word {
    let word = Word {
        name: name,
        arity: arity,
        expectations: Vec::new(),
        // action: action_add,
        substitution: substitution,
    };

    return word;
}
fn action_define(words: &mut HashMap<String, Word>, call: &mut Call) -> () {
    println!(
        "define word \"{}\" argument {:?}",
        call.name, call.arguments
    );
    let first = call.arguments.pop().unwrap();
    match first {
        LexItem::Word(w) => {
            let value = call.arguments.pop().unwrap();
            words.insert(w.to_string(), make_word(w.to_string(), 0, Some(value)));
        }
        _ => {
            println!("expected word");
        }
    }

    return ();
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
    let conditional = call.arguments.pop().unwrap();
    let if_clause = call.arguments.pop().unwrap();

    let else_clause = call.arguments.pop().unwrap();

    if check_conditional(conditional) {
        call.results.push(if_clause);
    } else {
        call.results.push(else_clause);
    }
}

fn action_substitution(call: &mut Call) -> () {
    let sub = call.substitution.take();
    if sub.is_none() {
        return ();
    }
    call.results = vec![sub.unwrap()];
    println!("sub: {:?}", call.results);
    return ();
}
fn make_call(word: &Word) -> Call {
    let sub = word.substitution.clone();
    let a = Call {
        name: word.name.to_string(),
        //action: word.action,
        arity: word.arity,
        arguments: Vec::new(),
        expectations: word.expectations.to_vec(),
        results: Vec::new(),
        substitution: sub,
    };
    return a;
}

fn main() {
    let cstack = &mut CallStack {
        stack: Vec::new(),
        words: HashMap::new(),
    };
    cstack.create_builtin_words();
    let mut istack: &mut Vec<LexItem> = &mut Vec::new();
    let ostack: &mut Vec<LexItem> = &mut Vec::new();
    let lexstack: &mut Vec<LexItem> = &mut Vec::new();
    //add '+' builtin

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);

        //lex all of the input
        let mut it = (&args[1]).chars().peekable();
        while it.peek() != None {
            match next_lexeme(&mut it) {
                Some(lexeme) => {
                    lexstack.insert(0, lexeme);
                }
                _ => (),
            }
        }
        println!("lexstack: {:?}", lexstack);
        istack = parse_stacks(lexstack, istack);
        println!("istack: {:?}", istack);
        loop {
            println!("istack: {:?}", istack);
            while !cstack.wantsData() {
                if cstack.len() > 0 {
                    let result = cstack.top_apply(istack);
                } else {
                    break;
                };
            }
            println!("istack: {:?}", istack);
            if istack.len() == 0 {
                break;
            }
            match istack.pop().unwrap() {
                LexItem::Word(w) => {
                    let mut pw = &mut LexItem::Word(w);
                    let lexreturn = cstack.pushLexItem(&mut pw);
                    match lexreturn {
                        Some(LexItem::Word(w)) => {
                            cstack.pushWordCall(w.to_string());
                        }
                        _ => {}
                    }
                }

                LexItem::Num(n) => {
                    if cstack.pushLexItem(&mut LexItem::Num(n)).is_none() {
                        println!("pushed item: {}", print_lexeme(&LexItem::Num(n)));
                    } else {
                        println!("output item: {}", print_lexeme(&LexItem::Num(n)));
                        ostack.push(LexItem::Num(n));
                    }
                }
                LexItem::Stack(mut s) => {
                    let stack = &mut LexItem::Stack(s);
                    let lexreturn = cstack.pushLexItem(stack);
                    match lexreturn {
                        Some(LexItem::Stack(l)) => {
                            istack.append(l);
                        }
                        _ => {}
                    }
                }
                _ => (),
            }
        }
    }

    println!("ostack: {:?}", ostack);
    println!("The first argument is {}", args[1]);
    // println!("Hello, world! {}", match words.stack.get("define") { Some(fun) => fun, None=> "none" });
}
