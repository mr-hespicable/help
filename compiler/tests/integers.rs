use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use compiler::to_file;

#[test]
fn integers() {
    let test_c_dir = Path::new("./tests/test_c_files/integers");
    for dir in fs::read_dir(test_c_dir).unwrap() {
        if dir.as_ref().unwrap().path().is_file() {
            let rawpath: PathBuf = dir.unwrap().path();
            let path_str: Option<&str> = rawpath.to_str();

            let ext = path_str
                .as_ref()
                .unwrap()
                .rsplitn(2, ".")
                .collect::<Vec<_>>()[0];

            dbg![ext];
            if ext == "c" {
                match path_str.unwrap().rsplitn(2, "/").collect::<Vec<_>>()[0]
                    .splitn(2, "_")
                    .collect::<Vec<_>>()[0]
                {
                    "valid" => {
                        dbg![&path_str];
                        valid_test(path_str);
                    }
                    "invalid" => {
                        dbg![&path_str];
                        invalid_test(path_str);
                    }
                    _ => panic!("Invalid prefix to test c file"),
                }
            }
        }
    }

    fn valid_test(path_str: Option<&str>) {
        let r = to_file(path_str.unwrap());
        dbg![&r];
        match r {
            Ok(_) => {},
            Err(e) => panic!("{:?}", e)
        }
    }
    
    fn invalid_test(path_str: Option<&str>) {
        let r = to_file(path_str.unwrap());
        dbg![&r];
        match r {
            Ok(o) => panic!("{:?}", o),
            Err(_) => {},
        }
    }
    // to_file();
}
