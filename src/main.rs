use lazy_static::lazy_static;
use regex::Regex;
use std::env::args;
use std::process::ExitCode;

struct MyNum {
    mant: u64,
    exp: u8,
}

#[derive(PartialEq, Debug)]
struct Fract {
    numer: u64,
    denom: u64,
}

enum FType {
    Num,
    Den,
}

fn main() -> ExitCode {
    if args().len() <= 1 {
        eprintln!("E{:04}: {}", 10, "No argument found");
        return ExitCode::from(10);
    }

    if args().len() != 2 {
        eprintln!("E{:04}: {}", 12, format!("Too many arguments ({})", args().len() - 1));
        return ExitCode::from(12);
    }

    let my_arg = args().skip(1).next().unwrap();
    let my_opt = get_fract(&my_arg);

    if let Err((ecd, emsg)) = my_opt {
        eprintln!("E{:04}: {}", ecd, emsg);
        return ExitCode::from(ecd);
    }

    let my_mfr = my_opt.unwrap();

    println!("{}/{}", my_mfr.numer, my_mfr.denom);
    ExitCode::SUCCESS
}

fn get_fract(inp_fract: &String) -> Result<Fract, (u8, String)> {
    lazy_static! { static ref RX_FRACT1: Regex = Regex::new(r"(?xms)\A ([^/]+)           \z").unwrap(); }
    lazy_static! { static ref RX_FRACT2: Regex = Regex::new(r"(?xms)\A ([^/]+) / ([^/]+) \z").unwrap(); }

    let (inp_nom, inp_den);

    if let Some(s) = RX_FRACT1.captures(&inp_fract) {
        inp_nom = s[1].to_string();
        inp_den = "1".to_string();
    }
    else if let Some(s) = RX_FRACT2.captures(&inp_fract) {
        inp_nom = s[1].to_string();
        inp_den = s[2].to_string();
    }
    else {
        return Err((14, format!("Could not parse fraction")));
    }

    let val_num = get_num(FType::Num, &inp_nom)?;
    let val_den = get_num(FType::Den, &inp_den)?;

    let exp_p10 =
        if val_num.exp > val_den.exp {
            val_num.exp - val_den.exp
        }
        else {
            val_den.exp - val_num.exp
        };

    let opt_p10 = 10_u64.checked_pow(exp_p10.into());

    if opt_p10.is_none() {
        return Err((16, format!("p10 overflow for 10 ^ {}", exp_p10)));
    }

    let val_p10 = opt_p10.unwrap();

    let mfr_dat =
        if val_num.exp > val_den.exp {
            let opt_den = val_den.mant.checked_mul(val_p10);

            if opt_den.is_none() {
                return Err((18, format!("Denominator overflow: {} * {}", val_den.mant, val_p10)));
            }

            Fract{ numer: val_num.mant, denom: opt_den.unwrap() }
        }
        else {
            let opt_num = val_num.mant.checked_mul(val_p10);

            if opt_num.is_none() {
                return Err((20, format!("Numerator overflow: {} * {}", val_num.mant, val_p10)));
            }

            Fract{ numer: opt_num.unwrap(), denom: val_den.mant }
        };

    Ok(get_norm(&mfr_dat)?)
}

fn get_num(stype: FType, sval: &String) -> Result<MyNum, (u8, String)> {
    let label = match stype { FType::Num => "Numerator", FType::Den => "Denominator" };

    lazy_static! { static ref RX_NUM1: Regex = Regex::new(r"(?xms)\A \d+               \z").unwrap(); }
    lazy_static! { static ref RX_NUM2: Regex = Regex::new(r"(?xms)\A (\d+) [,\.] (\d+) \z").unwrap(); }

    let gn_mant: String;
    let gn_exp: u8;

    if RX_NUM1.find(&sval).is_some() {
        gn_mant = sval.to_string();
        gn_exp  = 0;
    }
    else if let Some(s) = RX_NUM2.captures(&sval) {
        let p1 = s[1].to_string();
        let p2 = s[2].to_string();

        gn_mant = p1 + &p2;
        gn_exp  = u8::try_from(p2.len()).unwrap_or(0);
    }
    else {
        return Err((22, format!("Can't parse {} = '{}'", label, sval)));
    }

    if let Ok(v) = gn_mant.parse::<u64>() {
        return Ok(MyNum{ mant: v, exp: gn_exp });
    }

    Err((24, format!("Integer overflow {} = '{}'", label, sval)))
}

fn get_norm(fr: &Fract) -> Result<Fract, (u8, String)> {
    if fr.denom == 0 {
        return Err((26, "Division by zero".to_string()));
    }

    if fr.numer == 0 {
        return Ok(Fract{ numer: 0, denom: 1 });
    }

    // Calculate gcd using the Euclid algorithm
    // https://en.wikipedia.org/wiki/Euclidean_algorithm

    let mut xa = fr.numer;
    let mut xb = fr.denom;

    while xb > 0 {
        let tmp = xb;
        xb = xa % xb;
        xa = tmp;
    }

    Ok(Fract{ numer: fr.numer / xa, denom: fr.denom / xa})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0010() {
        let result = get_norm(&Fract{ numer: 486, denom: 12 });
        assert_eq!(result, Ok(Fract{ numer: 81, denom: 2 }));
    }

    #[test]
    fn test_0020() {
        let result = get_norm(&Fract{ numer: 96, denom: 4 });
        assert_eq!(result, Ok(Fract{ numer: 24, denom: 1 }));
    }

    #[test]
    fn test_0030() {
        let result = get_norm(&Fract{ numer: 0, denom: 3 });
        assert_eq!(result, Ok(Fract{ numer: 0, denom: 1 }));
    }

    #[test]
    fn test_0040() {
        let result = get_norm(&Fract{ numer: 0, denom: 0 });

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 26);
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0050() {
        let result = get_fract(&"100000000000000000000/3".to_string());

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 24); // Integer overflow Numerator
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0060() {
        let result = get_fract(&"3/100000000000000000000".to_string());

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 24); // Integer overflow Denominator
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0070() {
        let result = get_fract(&"3/10000000000000000000".to_string());

        if let Ok(_) = result {
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0080() {
        let result = get_fract(&"0,000000000000001/1000000000000000000".to_string());

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 18); // Denominator overflow
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0090() {
        let result = get_fract(&"1000000000000000000/0,000000000000001".to_string());

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 20); // Numerator overflow
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_0100() {
        let result = get_fract(&"35,6/12".to_string());
        assert_eq!(result, Ok(Fract{ numer: 89, denom: 30 }));
    }

    #[test]
    fn test_0110() {
        let result = get_fract(&"smdjfklsjkdf".to_string());

        if let Err((ecd, _)) = result {
            assert_eq!(ecd, 22); // Can't parse
        }
        else {
            assert!(false);
        }
    }
}
