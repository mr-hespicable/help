use compiler::to_file;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

const CDIRNAME: &str = "binary_ops";

#[test]
fn binary() {
    let dir_string = format!("./tests/test_c_files/{}/", CDIRNAME);
    let test_c_dir = Path::new(&dir_string);
    for dir in fs::read_dir(test_c_dir).unwrap() {
        if dir.as_ref().unwrap().path().is_file() {
            let rawpath: PathBuf = dir.unwrap().path();
            let path_str: Option<&str> = rawpath.to_str();
            let ext = path_str
                .as_ref()
                .unwrap()
                .rsplitn(2, ".")
                .collect::<Vec<_>>()[0];

            if ext == "c" {
                match path_str.unwrap().rsplitn(2, "/").collect::<Vec<_>>()[0]
                    .splitn(2, "_")
                    .collect::<Vec<_>>()[0]
                {
                    "valid" => {
                        valid_test(path_str);
                    }
                    "invalid" => {
                        invalid_test(path_str);
                    }
                    _ => panic!("Invalid prefix to test c file"),
                }
            }
        }
    }

    fn valid_test(path_str: Option<&str>) {
        let r = to_file(path_str.unwrap());
        dbg![path_str, &r];
        match r {
            Ok(_) => {},
            Err(e) => panic!("{:?}", e)
        }
    }

    fn invalid_test(path_str: Option<&str>) {
        let r = to_file(path_str.unwrap());
        dbg![path_str, &r];
        match r {
            Ok(o) => panic!("{:?}", o),
            Err(_) => {},
        }
    }
    // to_file();
}
