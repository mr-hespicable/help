use std::env;

pub fn main() {
    // let r = compiler::to_file("./tests/test_c_files/binary_ops/valid/long_one.c");
    // match r {
    //     Ok(_) => {},
    //     Err(e) => panic!("{:?}", e)
    // }

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let r = compiler::to_file(&args[1]);
        match r {
            Ok(_) => {}
            Err(e) => panic!("{:?}", e),
        }
    }
}
