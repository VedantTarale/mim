use std::collections::BTreeMap;

#[derive(Debug)]
enum Item { //creating an enum which represents the types of data present in the bencoded .torrent file
    Int(usize),
    String(Vec<u8>),
    List(Vec<Item>),
    Dict(BTreeMap<Vec<u8>, Item>),
}

//function to parse int ==> i<number>e
fn parse_int(str: &mut Vec<u8>) -> usize {
    let mut len: usize = 0;
    let mut int_string: String = String::new();
    for c in str.iter() {
        len += 1;
        if *c == b'i' {
            continue;
        }
        if *c == b'e' {
            break;
        }
        int_string.push(*c as char);
    }
    str.drain(0..len);
    int_string.parse::<usize>().unwrap()
}

//function to parse string ==> <length>:<literal>
fn parse_str(str: &mut Vec<u8>) -> Vec<u8> {
    let mut int_len: usize = 0;
    let mut int_string: String = String::new();
    for c in str.iter() {
        int_len += 1;
        if *c == b':' {
            break;
        }
        int_string.push(*c as char);
    }
    let len: usize = int_string.parse::<usize>().unwrap();
    str.drain(0..int_len);

    let s = str[..len].to_vec();
    let mut copy = str[len..].to_vec();
    str.clear();
    str.append(&mut copy);
    s
}

//funciton to parse list ==> l<list_items>e
fn parse_list(str: &mut Vec<u8>) -> Vec<Item> {
    str.drain(0..1);
    let mut list: Vec<Item> = Vec::<Item>::new();
    loop {
        match *str.get(0).unwrap() as char {
            'i' => list.push(Item::Int(parse_int(str))),
            'l' => list.push(Item::List(parse_list(str))),
            'd' => list.push(Item::Dict(parse_dict(str))),
            '0'..='9' => list.push(Item::String(parse_str(str))),
            'e' => break,
            _ => unreachable!(),
        }
    }
    str.drain(0..1);
    list
}

//function to parse dict ==> d<key_value_pairs>e
fn parse_dict(str: &mut Vec<u8>) -> BTreeMap<Vec<u8>, Item> {
    str.drain(0..1);
    let mut dict: BTreeMap<Vec<u8>, Item> = BTreeMap::new();
    loop {
        if *str.get(0).unwrap() == b'e' {
            break;
        }
        let s = parse_str(str);
        match *str.get(0).unwrap() as char {
            'i' => {
                dict.insert(s, Item::Int(parse_int(str)));
            }
            'l' => {
                dict.insert(s, Item::List(parse_list(str)));
            }
            'd' => {
                dict.insert(s, Item::Dict(parse_dict(str)));
            }
            '0'..='9' => {
                dict.insert(s, Item::String(parse_str(str)));
            }
            _ => unreachable!(),
        };
    }
    str.drain(0..1);
    dict
}

fn parse(str: &mut Vec<u8>) -> Vec<Item> {
    let mut tree: Vec<Item> = Vec::<Item>::new();
    while let Some(c) = str.get(0) {
        match *c {  //similar to switch case in C. matching the values to their bencoded start point
            b'i' => tree.push(Item::Int(parse_int(str))),
            b'l' => tree.push(Item::List(parse_list(str))),
            b'd' => tree.push(Item::Dict(parse_dict(str))),
            b'0'..=b'9' => tree.push(Item::String(parse_str(str))),
            _ => break,
        }
    }
    tree
}
#[tokio::main]
async fn main() {
    // get arguments from the terminal
    let args = std::env::args().collect::<Vec<String>>();   //args returns an iterator on whiich we use the collect method to turn it into a collection 
    let arg = if let Some(s) = args.get(1) {    //we only take the fist argument entered in the command line
        s
    } else {
        eprintln!("no torrent file specified");
        return;
    };

    // read and parse torrent file
    let mut bytes: Vec<u8> = match tokio::fs::read(arg).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{} {:?}", e, arg);
            return;
        }
    };
    
    let data = parse(&mut bytes);
    for entry in data {
        match entry{
            Item::Int(i) => println!("Integer: {}",i),
            Item::Dict(d) => {
                println!("Dictionary:");
                for (key, value) in d {
                    println!("\t- {:?} => {:?}", key, value);
                }
            },
            Item::String(s) =>  println!("String: {:?}", s),
            Item::List(l) => {
                println!("List:");
                for subitem in l {
                    println!("\t- {:?}", subitem);
                }
            },
        }
    }
}
