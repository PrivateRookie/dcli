use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "extend.pest"]
pub struct ExtendParser;

pub struct ParamLine {
    pub ty: String,
    pub name: String,
    pub default: Option<String>,
    pub help: Option<String>,
}

pub enum Param {
    Raw {
        name: String,
        default: Option<String>,
    },
    Str {
        name: String,
        default: Option<String>,
    },
    StrUnList {
        name: String,
        default: Option<Vec<String>>,
    },
    StrList {
        name: String,
        default: Option<Vec<String>>,
        max: usize,
    },
    Int {
        name: String,
        default: Option<String>,
    },
    IntUnList {
        name: String,
        default: Option<Vec<i64>>,
    },
    IntList {
        name: String,
        default: Option<Vec<i64>>,
        max: usize,
    },
    Float {
        name: String,
        default: Option<String>,
    },
    FloatUnList {
        name: String,
        default: Option<Vec<f64>>,
    },
    FloatList {
        name: String,
        default: Option<Vec<f64>>,
        max: usize,
    },
}

#[test]
fn test_parse() {
    fn test(name: &str, input: &str) {
        let mut ret =
            ExtendParser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}\n{}", name, e));
        let line = ret.next().unwrap();
        let mut inners = line.into_inner();
        // skip white space after "-- param"
        let param = inners.next().unwrap().into_inner();
        dbg!(inners);
    }

    test("param_str", "-- param name: str = '32rookie' // OK");
    test(
        "param_str_list_0",
        "-- param addr: [str] = ['12', '23', 'gx',]",
    );
    test(
        "param_str_list_1",
        "-- param addr: [str; 10] = ['12', '23', 'gx', ]",
    );
    test("param_int", "-- param age: int = 10 // YES");
    test("param_int_list_0", "-- param some: [int] = [12, +23, -23,]");
    test(
        "param_int_list_1",
        "-- param some: [int; 3] = [12, +23, -23,]",
    );
    test("param_float", "-- param score: float= 60.0 // YES");
    test(
        "param_float_list_0",
        "-- param score: [float] = [ 60.0, -10.2, +12.1,] // YES",
    );
    test(
        "param_float_list_1",
        "-- param score: [float; 3] = [ 60.0, -10.1, +12.1,] // YES",
    );
    test(
        "param_raw",
        r"-- param        magic: raw = #Date()\## // insert direct into sql",
    );
}
