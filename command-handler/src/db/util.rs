use rand::prelude::*;
use std::str::FromStr;
use std::u128;

pub fn new_iban() -> String {
    let mut rng = thread_rng();
    let mut digits = String::from("0");
    for _i in 0..=8 {
        digits.push_str(rng.gen_range(0, 10).to_string().as_ref());
    }
    open_iban(&digits[..])
}

fn open_iban(digits: &str) -> String {
    let check_nr = get_check_nr(digits);
    format!("NL{}OPEN{}", check_nr, digits)
}

fn get_check_nr(digits: &str) -> String {
    let string_for_check = format!("24251423{}232100", digits);
    let check_nr = (98 - u128::from_str(string_for_check.as_ref()).unwrap() % 97).to_string();
    if check_nr.len() == 1 {
        format!("0{}", check_nr)
    } else {
        check_nr
    }
}

pub fn new_token() -> String {
    let mut rng = thread_rng();
    let mut digits = String::new();
    for _i in 0..=19 {
        digits.push_str(rng.gen_range(0, 10).to_string().as_ref());
    }
    digits
}

pub fn invalid_from(from: &str) -> bool {
    if "cash" == from {
        false
    } else {
        !valid_open_iban(from)
    }
}

pub fn valid_open_iban(iban: &str) -> bool {
    if iban.len() != 18 {
        false
    } else {
        iban == open_iban(&iban[8..])
    }
}

#[cfg(test)]
mod test {
    use crate::db::util::invalid_from;
    use crate::db::util::new_iban;
    use crate::db::util::new_token;
    use crate::db::util::open_iban;

    #[test]
    fn test_new_iban() {
        let nib = new_iban();
        println!("Iban: {}", nib);
        assert_eq!(18, nib.len());
    }

    #[test]
    fn test_new_token() {
        let tkn = new_token();
        println!("Token: {}", tkn);
        assert_eq!(20, tkn.len());
    }

    #[test]
    fn test_invalid_from() {
        let nib = new_iban();
        println!("Iban: {}", nib);
        assert_eq!(false, invalid_from(&nib));
        assert_eq!(true, invalid_from(&String::from("bla")));
        assert_eq!(true, invalid_from(&String::from("NL83OPEN0104642752")));
        assert_eq!(false, invalid_from(&String::from("NL66OPEN0000000000")));
    }
}
