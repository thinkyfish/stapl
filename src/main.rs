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
                result = format!("{}'[ {}] ", result, format_lexstack(s));
            }
            LexItem::Lambda(s) => {
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
            LexItem::Lambda(s) => {
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
    Literal(String),
    Quote,
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
        let mut arity = 0;
        match self {
            LexItem::Lambda(s) => {
                let parameters = self.get_parameters();
                match parameters {
                    Some(mut p) => {
                        let mut max = 0;
                        let len = p.len();

                        for parameter in p {
                            if let Ok(number) = parameter.parse::<usize>() {
                                if number > max {
                                    max = number;
                                }
                            }
                        }

                        arity = max;
                    }
                    None => {}
                }
            }
            _ => {}
        }
        println!("get_arity {}", arity);
        return arity;
    }
    fn get_parameters(self: &Self) -> Option<Vec<String>> {
        match self {
            LexItem::Lambda(s) => {
                let mut parameters: Vec<String> = Vec::new();
                for lexeme in s {
                    if let LexItem::Parameter(p) = lexeme {
                        parameters.push(p.clone());
                    }
                }
                parameters.sort_unstable();
                parameters.dedup();
                println!("get_parameters {:?}", parameters);
                return Some(parameters);
            }
            _ => {
                return None;
            }
        }
    }
    fn get_expectations<'e>(self: &Self) -> Vec<Expectation> {
        let mut exp = Vec::new();

        if let Some(parameters) = self.get_parameters() {
            for s in parameters {
                exp.push(Expectation::NumStaLit);
            }
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
        '#' => {
            it.next();
            return Some(LexItem::Word("#".to_string()));
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
        '[' | '(' => {
            it.next();
            return Some(LexItem::OpenParen);
        }
        ']' | ')' => {
            it.next();
            return Some(LexItem::CloseParen);
        }
        '\'' => {
            it.next();
            return Some(LexItem::Quote);
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
    return format!("{}", number);
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
    let mut value = "".to_string();
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
            value = format_lexstack(s);
        }
        LexItem::Parameter(p) => {
            value = format!("${}", p);
        }
        _ => (),
    }
    return format!("{} ", value);
}

fn parse_stacks<'i>(
    lex_input: &mut Vec<LexItem>,
    parsed_input: &'i mut Vec<LexItem>,
) -> &'i mut Vec<LexItem> {
    let mut quoted = false;
    while let Some(itop) = lex_input.pop() {
        match itop {
            LexItem::OpenParen => {
                //println!("openbracket found");
                let mut newstack = &mut Vec::new();
                newstack = parse_stacks(lex_input, newstack);
                if quoted {
                    parsed_input.insert(0, LexItem::Stack(newstack.to_vec()));
                    quoted = false;
                } else {
                    parsed_input.insert(0, LexItem::Lambda(newstack.to_vec()));
                }
            }
            LexItem::CloseParen => {
                //println!("closedbracket found");

                return parsed_input;
            }
            LexItem::Quote => {
                quoted = true;
                println!("Parse: found quote");
            }
            /*             LexItem::Parameter(p) => {
                //println!("lexeme found: {}", print_lexeme(&itop));
                parsed_input.insert(0, LexItem::Parameter(p));
                parsed_input.insert(0, LexItem::Word("$".to_string()));
                let p = parse_stacks(lex_input, parsed_input);
                return p;

            } */
            LexItem::Num(n) => {
                let mut lexeme = LexItem::Num(n);
                if quoted {
                    let lexeme = LexItem::Literal(n.to_string());
                    quoted = false;
                }
                //println!("lexeme found: {}", print_lexeme(&itop));
                parsed_input.insert(0, lexeme);
                let p = parse_stacks(lex_input, parsed_input);

                return p;
            }
            LexItem::Word(w) => {
                if quoted {
                    let lexeme = LexItem::Literal(w.to_string());
                    parsed_input.insert(0, lexeme);
                    quoted = false;
                } else {
                    //println!("lexeme found: {}", print_lexeme(&itop));
                    parsed_input.insert(0, LexItem::Word(w));
                }
                let p = parse_stacks(lex_input, parsed_input);

                return p;
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

#[derive(Debug, Copy, Clone)]
enum Expectation {
    Num,
    Literal,
    Stack,
    Lambda,
    Any,
    Word,
    Parameter,
    NumStaLit,
}

#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    value: LexItem,
}

struct Call {
    name: String,
    action: fn(call: &mut Call) -> (),
    arity: usize,
    arguments: Vec<Parameter>,
    expectations: Vec<Expectation>,
    results: Vec<LexItem>,
    substitution: Option<LexItem>,
}
impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, args:{}, result:{}",
            self.name,
            format_parstack(&self.arguments),
            format_lexstack(&self.results)
        )
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

                self.stack.push(c);
                println!(
                    "pushed Define {}, Stack: {:?}",
                    word.to_string(),
                    self.stack
                );

                self.pushLexItem(&mut LexItem::Word(word));
            }
        }
    }
    fn pushWordCall(self: &mut Self, word: &mut Word) -> () {
        //println!("pushWordCall {:?}", word);
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
            action: action_add,
            substitution: None,
            expectations: vec![Expectation::Num, Expectation::Num],
        };
        self.words.insert("+".to_string(), addword);

        let addword = Word {
            name: "-".to_string(),
            arity: 2,
            action: action_subtract,
            substitution: None,
            expectations: vec![Expectation::Num, Expectation::Num],
        };
        self.words.insert("-".to_string(), addword);

        let string2 = String::from("if");
        let ifword = Word {
            name: "if".to_string(),
            arity: 3,
            action: action_if,
            substitution: None,
            expectations: vec![Expectation::Any, Expectation::Any, Expectation::Num],
        };
        self.words.insert(string2, ifword);

        let defword = Word {
            name: "define".to_string(),
            arity: 1,
            action: action_none, // change this to an option?
            substitution: None,
            expectations: vec![Expectation::Any, Expectation::Word],
        };
        self.words.insert("define".to_string(), defword);

        let defextract = Word {
            name: "#".to_string(),
            arity: 2,
            action: action_extract, // change this to an option?
            substitution: None,
            expectations: vec![Expectation::Stack, Expectation::Num],
        };
        self.words.insert("#".to_string(), defextract);
    }

    fn pushLexItem<'l>(self: &mut Self, lexeme: &'l mut LexItem) -> Option<&'l mut LexItem> {
        if let Some(top_call) = self.stack.last_mut() {
            let e = &mut top_call.expectations;
            match e.last() {
                Some(&top_expectation) => {
                    //let d = check_expectation(top_expectation, lexeme.clone());

                    //let dataitem = lexeme;
                    let mut expectation_match = true;
                    match (&top_expectation, &lexeme) {
                        (Expectation::Num, LexItem::Num(_))
                        | (Expectation::NumStaLit, LexItem::Num(_)) => (),
                        (Expectation::Word, LexItem::Word(_)) => (),
                        (Expectation::Parameter, LexItem::Parameter(_)) => (),
                        (Expectation::Stack, LexItem::Stack(_))
                        | (Expectation::NumStaLit, LexItem::Stack(_)) => (),
                        (Expectation::Stack, LexItem::Literal(_))
                        | (Expectation::NumStaLit, LexItem::Literal(_)) => (),
                        (Expectation::Lambda, LexItem::Lambda(_)) => (),
                        (Expectation::Any, _) => (),
                        _ => {
                            expectation_match = false;
                        }
                    };

                    if expectation_match {
                        e.pop();
                        let parname = format!("{}", top_call.arguments.len() + 1);
                        top_call.arguments.insert(
                            0,
                            Parameter {
                                name: parname,
                                value: lexeme.to_owned(),
                            },
                        );
                        return None;
                    }
                    //could put apply here
                }
                None => {}
            }
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
                    "define" => {
                        //action_define(&mut self.words, top_call);
                        let first = top_call.arguments.pop().unwrap().value;
                        match first {
                            LexItem::Word(w) => {
                                let value = top_call.arguments.pop().unwrap().value;
                                let mut newword = Word::make_word(
                                    w.to_string(),
                                    0,
                                    action_substitution,
                                    Some(value),
                                );
                                let n2 = newword.setup_expectations();
                                self.words.insert(w.to_string(), newword);
                                top_call.expectations.pop();
                            }
                            _ => {
                                println!("expected word");
                            }
                        }
                    }

                    _ => {
                        (top_call.action)(top_call);
                    }
                }

                result.append(&mut top_call.results);
                //result.insert(0, LexItem::Stack(top_call.results.clone()));
                println!("call result {}", format_lexstack(result));
                self.stack.pop();
                return true;
            }
            _ => {
                return false;
            }
        }
    }
}

//#[derive(Debug)]
struct Word {
    name: String,
    arity: usize,
    action: fn(call: &mut Call) -> (),
    substitution: Option<LexItem>,
    expectations: Vec<Expectation>,
}
impl Word {
    fn make_word<'w>(
        name: String,
        arity: usize,
        action: fn(call: &mut Call) -> (),
        //expectations: &'w mut Vec<Expectation>,
        substitution: Option<LexItem>,
    ) -> Word {
        let mut expectations: Vec<Expectation> = Vec::new();
        let sub = substitution.clone();
        match substitution {
            Some(s) => {
                expectations = s.get_expectations();
                println!(
                    "made word {},arity {}, {}",
                    name,
                    expectations.len(),
                    print_lexeme(&s)
                );
            }
            None => (),
        }
        let mut word = Word {
            name: name,
            arity: expectations.len(),
            expectations: expectations,
            action: action,
            substitution: sub,
        };

        return word;
    }
    fn setup_expectations(self: &mut Self) -> &mut Self {
        let sub = &mut self.substitution;
        let mut e = &mut self.expectations;
        match sub {
            Some(s) => {
                let e2 = s.get_expectations();
                self.expectations = e2;
            }
            None => {}
        }
        return self;
    }
}
fn action_none(call: &mut Call) -> () {
    return ();
}

fn action_extract(call: &mut Call) {
    let index = call.arguments.pop().unwrap().value.get_Num().unwrap() as i64;
    let s = call.arguments.pop().unwrap().value;
    if let LexItem::Stack(s) = s {
        if s.len() > 0 {
            if index > 0 && index <= s.len() as i64 {
                call.results
                    .push(s.get((s.len() as i64 - index) as usize).unwrap().clone());
            } else if index < 0 && index >= -(s.len() as i64) {
                call.results
                    .push(s.get((-(index + 1)) as usize).unwrap().clone());
            }
        }
    }
}
fn action_add(call: &mut Call) -> () {
    println!("action_add arguments {:?}", call.arguments);
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
fn action_parameter(call: &mut Call) -> Option<LexItem> {
    if let Some(l) = call.arguments.pop() {
        if let LexItem::Parameter(p) = l.value {
            for a in call.arguments.iter() {
                let mut value;
                if p == a.name {
                    value = a.value.clone();
                    //println!("reduced match p: {} to {:?}", *p, a.value);
                    return Some(value); //should only happen once ||X/  \X||
                }
            }
        }
    }
    return None;
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

fn parameter_reduce<'a>(a: &'a Parameter, l: &'a LexItem) -> Option<&'a LexItem> {
    match l {
        LexItem::Parameter(p) => {
            if a.name == *p {
                return Some(&a.value);
            } else {
                return None;
            }
        }
        LexItem::Stack(s) => {
            return None;
        }
        _ => {
            return None;
        }
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
            LexItem::Lambda(s) => {
                let mut curstack = Vec::new();
                curstack.push(s);

                while let Some(current) = curstack.pop() {
                    for mut value in current {
                        for a in call.arguments.iter() {
                            if let Some(p) = parameter_reduce(a, &value) {
                                //if let LexItem::Stack(s) = p {
                                // curstack.push(s.to_vec());
                                //} else {
                                value = p.clone();
                                //}
                            }
                        }
                        betasub.push(value);
                    }
                }
            }
            _ => (),
        }
        call.results = betasub;
    } else {
        match sub.unwrap() {
            LexItem::Lambda(s) => {
                call.results = s;
            }
            mut s => {
                call.results = vec![s];
            }
        }
    }

    println!("sub: {:?}", call);
    return ();
}

fn make_call(word: &Word) -> Call {
    let sub = word.substitution.clone();
    let a = Call {
        name: word.name.to_string(),
        action: word.action,
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
        println!("istack: {:?}", format_lexstack(istack));
        loop {
            loopcount = loopcount + 1;
            if loopcount > 100 {
                println!("exceeded maximum loops");
                break;
            };
            while !cstack.wantsData() {
                if cstack.len() > 0 {
                    let result = cstack.top_apply(istack);
                    println!("istack: {}", format_lexstack(istack));
                } else {
                    break;
                };
            }
            println!("cstack: {:?}", cstack.stack);
            println!("istack: {:?}", format_lexstack(istack));
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
                        let c = cstack.stack.last_mut();
                        if let Some(top_call) = c {
                            top_call.results.push(LexItem::Num(n));
                        } else {
                            println!("output item: {}", print_lexeme(&LexItem::Num(n)));
                            ostack.push(LexItem::Num(n));
                        }
                    }
                }
                LexItem::Lambda(mut s) => {
                    let stack = &mut LexItem::Lambda(s);
                    let lexreturn = cstack.pushLexItem(stack);
                    match lexreturn {
                        Some(l) => {
                            let arity = l.get_arity();
                            let mut w = Word::make_word(
                                "lambda".to_string(),
                                arity,
                                action_substitution,
                                Some(l.clone()),
                            );
                            w.setup_expectations();
                            cstack.pushWordCall(&mut w);
                            //istack.append(l);
                        }
                        _ => {}
                    }
                }
                LexItem::Stack(mut s) => {
                    if let Some(l) = cstack.pushLexItem(&mut LexItem::Stack(s)) {
                        //println!("output item: {}", format_lexstack(&s));
                        ostack.push(l.clone());
                    }
                }
                LexItem::Parameter(p) => {
                    let c = cstack.stack.last_mut();
                    if let Some(top_call) = c {
                        match action_parameter(top_call) {
                            Some(lexeme) => {
                                istack.push(lexeme);
                            }
                            None => (),
                        }
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
        let readline = reader.readline("STAPLr> ");
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
