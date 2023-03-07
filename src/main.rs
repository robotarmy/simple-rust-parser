use std::collections::HashMap;
#[derive(Debug, Clone, Copy)]
enum Operations {
    Accumulate,
    StoreTimestamp,
    Push,
    StoreAttributes,
    Skip,
}
#[derive(Debug, Clone, Copy)]
enum States {
    Timestamp,
    TimestampOrEol,
    Key,
    ValueOrValueQuote,
    Value,
    ValueQuote,
    ValueEndQuote,
    Eol,
    Ws,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
struct ParseState {
    op: Operations,
    cs: States,
}

#[derive(Debug)]
struct Datum {
    timestamp: i64,
    attributes: HashMap<String, String>
}

struct VM {
    acc: String,
    stack: Vec<String>
}

fn initial_state() -> ParseState {
    return ParseState { cs: States::Unknown, op: Operations::Skip };
}

fn next_state(input: char, this_state: &ParseState) -> ParseState {
    return match this_state.cs {
        States::Unknown | States::Eol | States::Timestamp | States::TimestampOrEol => {
            match input {
                '\n' => {
                    ParseState { cs: States::Eol, op: Operations::Skip }
                }
                '0'..='9' => {
                    ParseState { cs: States::Timestamp, op: Operations::Accumulate }
                }
                ' ' => {
                    ParseState { cs: States::Ws, op: Operations::StoreTimestamp }
                }
                _ => {
                    println!("Unexpected input");
                    ParseState { cs: States::Unknown, op: Operations::Skip }
                }
            }
        }
        States::Ws => {
            match input {
                ' ' => {
                    ParseState { cs: States::Ws, op: Operations::Skip }
                }
                _ => {
                    ParseState { cs: States::Key, op: Operations::Accumulate }
                }
            }
        }
        States::Key => {
            match input {
                '=' => {
                    ParseState { cs: States::ValueOrValueQuote, op: Operations::Push }
                }
                _ => {
                    ParseState { cs: States::Key, op: Operations::Accumulate }
                }
            }
        }
        States::ValueOrValueQuote => {
            match input {
                '"' => {
                    ParseState { cs: States::ValueQuote, op: Operations::Skip }
                }
                _ => {
                    ParseState { cs: States::Value, op: Operations::Accumulate }
                }
            }
        }
        States::Value |
        States::ValueQuote |
        States::ValueEndQuote => {
            match input {
                '"' => {
                    ParseState { cs: States::ValueEndQuote, op: Operations::Skip }
                }
                '\n' => {
                    ParseState { cs: States::TimestampOrEol, op: Operations::StoreAttributes }
                }
                ',' => {
                    ParseState { cs: States::Ws, op: Operations::Push }
                }
                _ => {
                    ParseState { cs: this_state.cs, op: Operations::Accumulate }
                }
            }
        }
    };
}

fn execute_state(vm: &mut VM, datum: &mut Datum, parse_state: ParseState, input: char) {
    let mut push_fn = || {
        let str = vm.acc.to_string();
        vm.stack.push(str);
        vm.acc.clear();
    };
    match parse_state.op {
        Operations::Skip => {
            // no operation
        }
        Operations::StoreTimestamp => {
            datum.timestamp = vm.acc.parse::<i64>().unwrap();
            vm.acc.clear();
        }
        Operations::Accumulate => {
            vm.acc.push(input);
        }
        Operations::Push => push_fn(),
        Operations::StoreAttributes => {
            push_fn(); // clear accumulator
            while vm.stack.len() >= 2 {
                let value = vm.stack.pop().unwrap();
                let key = vm.stack.pop().unwrap();
                datum.attributes.insert(key, value);
            }
        }
    }
}

fn main() {
    let line = String::from("\n
1234  pretty=きれいな, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1235  noisy=うるさい, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1236  spacious=ひろい, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1257  salmon=しゃけ, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1258  bean=まめ, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1259  near=ちかい, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
1260  fish=さかな, mem=23234M, cpu=2131, cat=\"hat\", boat=goat\n
");

    let mut datum = Datum {
        timestamp: -1,
        attributes: Default::default(),
    };

    let mut vm = VM {
        acc: Default::default(),
        stack: Default::default()
    };

    let mut data: Vec<Datum> = Default::default();
    let mut s1 = initial_state();
    for c in line.chars() {
        s1 = next_state(c, &s1);
        execute_state(&mut vm,&mut datum, s1, c);
        if c == '\n' {
            data.push(datum);
            datum = Datum {
                timestamp: -1,
                attributes: Default::default(),
            }
        }
    }
    while let Some(pdat) = data.pop() {
        println!("{:?}", pdat);
    }
}
