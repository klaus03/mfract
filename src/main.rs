use lazy_static::lazy_static;
use regex::Regex;
use std::env::args;
use std::process::ExitCode;

fn main() -> ExitCode {
    if args().len() <= 1 {
        eprintln!("E0010: No argument found");
        return ExitCode::from(10);
    }

    if args().len() != 2 {
        eprintln!("E0020: Too many arguments ({})", args().len() - 1);
        return ExitCode::from(20);
    }

    let inp_fract = args().skip(1).next().unwrap();

    lazy_static! { static ref RX_FRACT1: Regex = Regex::new(r"(?xms)\A ([^/]+)           \z").unwrap(); }
    lazy_static! { static ref RX_FRACT2: Regex = Regex::new(r"(?xms)\A ([^/]+) / ([^/]+) \z").unwrap(); }

    let c1 = RX_FRACT1.captures_iter(&inp_fract).next();
    let c2 = RX_FRACT2.captures_iter(&inp_fract).next();

    let (inp_nom, inp_den, inp_err);

    if let Some(s) = c1 {
        inp_err = false;
        inp_nom = s[1].to_string();
        inp_den = String::from("1");
    }
    else if let Some(s) = c2 {
        inp_err = false;
        inp_nom = s[1].to_string();
        inp_den = s[2].to_string();
    }
    else {
        inp_err = true;
        inp_nom = String::from("???");
        inp_den = String::from("???");
    }

    if inp_err {
        eprintln!("E0030: Could not parse fraction");
        return ExitCode::from(30);
    }

    let val_nom;
    let val_den;

    if let Ok(v) = inp_nom.parse::<u64>() {
        val_nom = v;
    }
    else {
        eprintln!("E0040: Could not parse nom = '{}'", inp_nom);
        return ExitCode::from(40);
    }

    if let Ok(v) = inp_den.parse::<u64>() {
        val_den = v;
    }
    else {
        eprintln!("E0050: Could not parse den = '{}'", inp_den);
        return ExitCode::from(50);
    }

    let res = fnorm(val_nom, val_den);

    if let Err(msg) = res {
        eprintln!("E0060: Failure in fnorm({}, {}) --> '{}'", val_nom, val_den, msg);
        return ExitCode::from(60);
    }

    let (out_nom, out_den) = res.unwrap();

    println!("{}/{}", out_nom, out_den);

    ExitCode::SUCCESS
}

fn fnorm(nom: u64, den: u64) -> Result<(u64, u64), String> {
    if den == 0 {
        return Err("division by zero".to_string());
    }

    if nom == 0 {
        return Ok((0, 1));
    }

    // Calculate gcd using the Euclid algorithm
    // https://en.wikipedia.org/wiki/Euclidean_algorithm

    let mut xa = nom;
    let mut xb = den;

    while xb > 0 {
        let tmp = xb;
        xb = xa % xb;
        xa = tmp;
    }

    Ok((nom / xa, den / xa))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0010() {
        let result = fnorm(486, 12);
        assert_eq!(result, Ok((81, 2)));
    }

    #[test]
    fn test_0020() {
        let result = fnorm(96, 4);
        assert_eq!(result, Ok((24, 1)));
    }

    #[test]
    fn test_0030() {
        let result = fnorm(0, 3);
        assert_eq!(result, Ok((0, 3)));
    }

    #[test]
    fn test_0040() {
        let result = fnorm(0, 0);

        if let Err(msg) = &result {
            assert_eq!(msg[0 .. 4], "divi".to_string());
        }
        else {
            assert!(false);
        }
    }
}
