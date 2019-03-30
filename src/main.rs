//use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::iter::Peekable;

extern crate rustyline;

fn format_lexstack(stack: &Vec<LexItem>) -> String {
    let mut result = "".to_string();
    let stack_iter = stack.iter().rev();
    if stack_iter.len() == 0 {
        result = format!("None");
    }
    for i in stack_iter {
        match i {
            LexItem::Stack(s) => {
                result = format!("{}[ {}] ", result, format_lexstack(s));
            }
            _ => {
                result = format!("{}{:?} ", result, i);
            }
        }
    }
    return result;
}
fn format_expstack(stack: &Vec<Expectation>) -> String {
    let mut result = "".to_string();
    let stack_iter = stack.iter().rev();
    for i in stack_iter {
        result = format!("{}{:?} ", result, i);
    }
    return result;
}

fn format_parstack(stack: &Vec<Parameter>) -> String {
    let mut result = "".to_string();
    let stack_iter = stack.iter().rev();
    if stack_iter.len() == 0 {
        result = format!("None");
    }
    for i in stack_iter {
        match &i.value {
            LexItem::Stack(s) => {
                result = format!("{}({})[ {}] ", result, i.name, format_lexstack(&s));
            }
            _ => {
                result = format!("{}{:?} ", result, i);
            }
        }
    }
    return result;
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum LexItem {
    Word(String),
    OpenParen,
    CloseParen,
    Num(i64),
    #[derive(Ord)]
    Parameter(String),
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
    /*     fn get_stack(self: &Self) -> Option<Vec<LexItem>> {
        match self {
            LexItem::Stack(s) => {
                return Some(s);
            }s
            _ => {
                return None;
            }
        }
    } */
    fn get_arity(self: &Self) -> usize {
        match self {
            LexItem::Stack(s) => {
                let parameters = self.get_parameters();
                match parameters {
                    Some(mut p) => {
                        p.dedup();
                        return p.len();
                    }
                    None => {
                        return 0;
                    }
                }
            }
            _ => {
                return 0;
            }
        }
    }
    fn get_parameters(self: &Self) -> Option<Vec<LexItem>> {
        match self {
            LexItem::Stack(s) => {
                return Some(
                    s.iter()
                        .filter(|x| match x {
                            LexItem::Parameter(_) => true,
                            _ => false,
                        })
                        .cloned()
                        .collect(),
                );
            }
            _ => {
                return None;
            }
        }
    }
    fn get_expectations<'e>(
        self: &Self,
        exp: &'e mut Vec<Expectation>,
    ) -> &'e mut Vec<Expectation> {
        let presult = self.get_parameters();
        match presult {
            Some(mut plist) => {
                let mut stringp = Vec::new();
                plist.dedup();
                for p in plist.iter() {
                    match p {
                        LexItem::Parameter(s) => {
                            stringp.push(s);
                        }
                        _ => (),
                    }
                }
                stringp.sort();
                for s in stringp.iter() {
                    exp.push(Expectation::Any);
                }
            }
            None => {}
        }
        return exp;
    }
}

fn next_lexeme<T: Iterator<Item = char>>(mut it: &mut Peekable<T>) -> Option<LexItem> {
    let c = *(it.peek().unwrap());

    //println!("lex {}", c);

    match c {
        '0'...'9' | '-' => {
            it.next();
            match lex_number(c, &mut it) {
                Some(n) => {
                    return Some(LexItem::Num(n));
                }
                None => {
                    let a = lex_word(c, &mut it);
                    return Some(LexItem::Word(a));
                }
            }
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

fn lex_parameter<T: Iterator<Item = char>>(inc: char, iter: &mut Peekable<T>) -> String {
    let mut number = 0;
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    return format!("${}", number);
}

fn lex_number<T: Iterator<Item = char>>(inc: char, iter: &mut Peekable<T>) -> Option<i64> {
    let mut sign = 1;
    let mut c = inc;
    if inc == '-' {
        sign = -1;
        if *iter.peek().unwrap() == ' ' {
            return None;
        }
        c = iter.next().unwrap();
    }
    let nparse = c.to_string().parse::<i64>();
    match nparse {
        Ok(n) => {
            let mut number = n;
            while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
                number = number * 10 + digit;
                iter.next();
            }
            //println!("get_number {}", number * sign);
            return Some(number * sign);
        }
        Err(_) => {
            //println!("end of number");
            return None;
        }
    }
}

fn lex_word<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> String {
    let mut word = c.to_string();
    //println!("word c: {}", c);
    while let Some(&letter) = iter.peek() {
        //println!("word peek: {}", letter);
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
        Expectation::Word => match l {
            LexItem::Word(w) => {
                return Some(LexItem::Word(w));
            }
            _ => (),
        },

        _ => (),
    }
    return None;
}

#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    value: LexItem,
}

struct Call {
    name: String,
    //action: fn(call: &mut Call) -> (),
    arity: usize,
    arguments: Vec<Parameter>,
    expectations: Vec<Expectation>,
    results: Vec<LexItem>,
    substitution: Option<LexItem>,
}
impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.name, format_parstack(&self.arguments))
    }
}
struct CallStack {
    stack: Vec<Call>,
    words: HashMap<String, Word>,
}
impl CallStack {
    fn pushSearchWord(self: &mut Self, word: String) -> () {
        match self.words.get(&word) {
            Some(w) => {
                self.stack.push(make_call(w));
                println!("pushed Call: {}, Stack: {:?}", w.name, self.stack);
            }
            None => {
                let defword = self.words.get(&"define".to_string()).unwrap();

                let mut c = make_call(defword);
                c.expectations.pop();
                let parname = format!("${}", c.arguments.len() + 1);
                c.arguments.push(Parameter {
                    name: parname,
                    value: LexItem::Word(word.to_string()),
                });

                self.stack.push(c);

                println!("pushed Define {}, Stack: {:?}", word, self.stack);
            }
        }
    }
    fn pushWordCall(self: &mut Self, word: &mut Word) -> () {
        self.stack.push(make_call(word));

        return ();
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
        let addword = Word {
            name: "+".to_string(),
            arity: 2,
            //action: action_add,
            substitution: None,
            expectations: vec![Expectation::Num, Expectation::Num],
        };
        self.words.insert("+".to_string(), addword);

        let addword = Word {
            name: "-".to_string(),
            arity: 2,
            //action: action_add,
            substitution: None,
            expectations: vec![Expectation::Num, Expectation::Num],
        };
        self.words.insert("-".to_string(), addword);

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
                //println!("no call");
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
                        let parname = format!("${}", top_call.arguments.len() + 1);
                        top_call.arguments.insert(
                            0,
                            Parameter {
                                name: parname,
                                value: dataitem,
                            },
                        );
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
                match call.expectations.last() {
                    Some(e) => {
                        print!("expects: {}\n", format_expstack(&call.expectations));
                    }
                    None => {
                        print!("expects: None\n");
                    }
                }
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
                println!(
                    "apply {} with parameters : {}",
                    top_call.name,
                    format_parstack(&top_call.arguments)
                );
                match top_call.name.as_str() {
                    "+" => {
                        action_add(top_call);
                    }
                    "-" => {
                        action_subtract(top_call);
                    }
                    "if" => {
                        action_if(top_call);
                    }
                    "define" => {
                        action_define(&mut self.words, top_call);
                    }
                    "stack" | _ => {
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
    arity: usize,
    //action: fn(call: &mut Call) -> (),
    substitution: Option<LexItem>,
    expectations: Vec<Expectation>,
}
impl Word {
    fn make_word<'w>(
        name: String,
        arity: usize,
        //  action: fn(call: &mut Call) -> (),
        //expectations: &'w mut Vec<Expectation>,
        substitution: Option<LexItem>,
    ) -> Word {
        let mut word = Word {
            name: name,
            arity: arity,
            expectations: Vec::new(),
            // action: action_add,
            substitution: substitution,
        };

        return word;
    }
    fn setup_expectations(self: &mut Self) {
        let sub = &mut self.substitution;
        let mut e = &mut self.expectations;
        match sub {
            Some(s) => {
                let e2 = s.get_expectations(e);
                self.expectations = e2.to_vec();
            }
            None => {}
        }
    }
}
fn action_define(words: &mut HashMap<String, Word>, call: &mut Call) -> () {
    /*     println!(
        "define word \"{}\" argument {:?}",
        call.name, call.arguments
    ); */
    let first = call.arguments.pop().unwrap().value;
    match first {
        LexItem::Word(w) => {
            let value = call.arguments.pop().unwrap().value;
            words.insert(
                w.to_string(),
                Word::make_word(w.to_string(), 0, Some(value)),
            );
            call.expectations.pop();
        }
        _ => {
            println!("expected word");
        }
    }

    return ();
}
fn action_add(call: &mut Call) -> () {
    //println!("action_add arguments {:?}", call.arguments);
    let a = call.arguments.pop().unwrap().value.get_Num().unwrap();
    let b = call.arguments.pop().unwrap().value.get_Num().unwrap();
    call.results.push(LexItem::Num(a + b));
    //println!("action_add {} + {}", a, b);
}
fn action_subtract(call: &mut Call) -> () {
    //println!("action_add arguments {:?}", call.arguments);
    let a = call.arguments.pop().unwrap().value.get_Num().unwrap();
    let b = call.arguments.pop().unwrap().value.get_Num().unwrap();
    call.results.push(LexItem::Num(a - b));
    //println!("action_add {} + {}", a, b);
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
    //println!("action_if arguments {:?}", call.arguments);
    let conditional = call.arguments.pop().unwrap().value;
    let if_clause = call.arguments.pop().unwrap().value;

    let else_clause = call.arguments.pop().unwrap().value;

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

    //beta reduction
    let mut betasub = Vec::new();
    if call.arity > 0 {
        match sub.unwrap() {
            LexItem::Stack(s) => {
                for i in s.iter() {
                    let mut value = i.clone();
                    let mut foundvalue = false;
                    for a in call.arguments.iter() {
                        match i {
                            LexItem::Parameter(p) => {
                                if (*p == a.name) {
                                    value = a.value.clone();
                                    println!("reduced match p: {} to {:?}", *p, a.value);
                                }
                            }
                            _ => (),
                        }
                    }

                    betasub.push(value);
                }
            }
            _ => (),
        }
        call.results = betasub;
    } else {
        call.results = vec![sub.unwrap()];
    }
    println!("sub: {:?}", call.name);
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
fn eval<'o>(input: String, ostack: &'o mut Vec<LexItem>) -> &'o mut Vec<LexItem> {
    let cstack = &mut CallStack {
        stack: Vec::new(),
        words: HashMap::new(),
    };
    cstack.create_builtin_words();
    let mut istack: &mut Vec<LexItem> = &mut Vec::new();

    let lexstack: &mut Vec<LexItem> = &mut Vec::new();
    //add '+' builtin
    let mut loopcount = 0;
    if input.len() > 1 {
        println!("The first argument is {}", input);

        //lex all of the input
        let mut it = input.chars().peekable();
        while it.peek() != None {
            match next_lexeme(&mut it) {
                Some(lexeme) => {
                    lexstack.insert(0, lexeme);
                }
                _ => (),
            }
        }
        println!("lexstack: {}", format_lexstack(lexstack));
        istack = parse_stacks(lexstack, istack);
        //println!("istack: {:?}", istack);
        loop {
            loopcount = loopcount + 1;
            if loopcount > 100 {
                break;
            };
            println!("istack: {}", format_lexstack(istack));
            while !cstack.wantsData() {
                if cstack.len() > 0 {
                    let result = cstack.top_apply(istack);
                } else {
                    break;
                };
            }
            println!("cstack: {:?}", cstack.stack);
            if istack.len() == 0 {
                break;
            }
            match istack.pop().unwrap() {
                LexItem::Word(w) => {
                    let mut pw = &mut LexItem::Word(w);
                    let lexreturn = cstack.pushLexItem(&mut pw);
                    match lexreturn {
                        Some(LexItem::Word(w)) => {
                            cstack.pushSearchWord(w.to_string());
                        }
                        _ => {}
                    }
                }

                LexItem::Num(n) => {
                    if cstack.pushLexItem(&mut LexItem::Num(n)).is_none() {
                        println!("pushed expected item: {}", print_lexeme(&LexItem::Num(n)));
                    } else {
                        println!("output item: {}", print_lexeme(&LexItem::Num(n)));
                        ostack.push(LexItem::Num(n));
                    }
                }
                LexItem::Stack(mut s) => {
                    let stack = &mut LexItem::Stack(s);
                    let lexreturn = cstack.pushLexItem(stack);
                    match lexreturn {
                        Some(l) => {
                            let arity = l.get_arity();
                            let mut w =
                                Word::make_word("stack".to_string(), arity, Some(l.clone()));
                            w.setup_expectations();
                            cstack.pushWordCall(&mut w);
                            //istack.append(l);
                        }
                        _ => {}
                    }
                }
                _ => (),
            }
        }
    }
    println!("cstack: {:?}", cstack.stack);
    return ostack;
}
fn main() {
    let mut reader = Editor::<()>::new();
    if let Err(_) = reader.load_history("staplr_history.txt") {
        println!("No previous history.");
    }

    loop {
        let readline = reader.readline("staplr> ");
        //let args: Vec<_> = env::args().collect();
        let ostack: &mut Vec<LexItem> = &mut Vec::new();
        match readline {
            Ok(line) => {
                reader.add_history_entry(line.as_ref());
                eval(line.to_string(), ostack);
                println!("input was: {}", line.to_string());
            }
            _ => {
                break;
            }
        }
        println!("ostack: {}", format_lexstack(ostack));
    }
    reader.save_history("staplr_history.txt").unwrap();
    // println!("Hello, world! {}", match words.stack.get("define") { Some(fun) => fun, None=> "none" });
}
