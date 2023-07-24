use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::Path,
};

use walkdir::WalkDir;

type VersionType = u64;

const FFI_PREFIX: &str = "    pub mod FFI {\n";
const FFI_SUFFIX: &str = "\n    }";
const ROOT_PREFIX: &str = "pub mod root {\n";
const ROOT_SUFFIX: &str = "\n}\n";

fn read_file_to_string<P: AsRef<Path>>(path: P, expect: &str) -> String {
    let mut file: fs::File = fs::File::open(path).expect(expect);
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect(expect);
    buff
}

fn scan_ffi_src_head(dir: &str) -> (Vec<String>, VersionType) {
    let mut headers = Vec::new();
    let mut headers_buff = HashMap::new();
    for result in WalkDir::new(Path::new(dir)) {
        let dent = result.expect("Error happened when search headers");
        if !dent.file_type().is_file() {
            continue;
        }
        println!("has dent.path() {:?}", dent.path());
        let buff = read_file_to_string(dent.path(), "Couldn't open headers");
        let head_file_path = String::from(dent.path().to_str().unwrap());
        if !head_file_path.ends_with(".h") {
            continue;
        }
        headers.push(head_file_path.clone());
        headers_buff.insert(head_file_path, buff);
    }
    headers.sort();
    let hash_version = {
        let mut hasher = DefaultHasher::new();
        for name in &headers {
            let buff = headers_buff.get(name).unwrap();
            buff.hash(&mut hasher);
        }
        hasher.finish()
    };
    (headers, hash_version)
}

fn filter_by_namespace(buff: &str) -> String {
    let mut res = String::new();
    // pub mod root {?
    let a1 = buff.find(ROOT_PREFIX).unwrap() + ROOT_PREFIX.len();
    res.push_str(&buff[..a1]);
    // ?    pub mod FFI {
    let b1 = buff.find(FFI_PREFIX).unwrap();
    let b2 = (&buff[b1..]).find(FFI_SUFFIX).unwrap() + FFI_SUFFIX.len() + b1;
    res.push_str(&buff[b1..b2]);
    res.push_str(ROOT_SUFFIX);
    res
}

pub fn gen_ffi_code() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = format!(
        "{}/../interfaces",
        manifest_dir
    );
    let tar_file = format!(
        "{}/../rustpeer/src/interfaces.rs",
        manifest_dir
    );

    println!("\nFFI src dir path is {}\ntarget {}", src_dir, tar_file);

    let mut builder = bindgen::Builder::default()
        .clang_arg("-xc++")
        .clang_arg("-std=c++11")
        .clang_arg("-Wno-pragma-once-outside-header")
        .layout_tests(false)
        .derive_copy(false)
        .enable_cxx_namespaces()
        .disable_header_comment()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        });

    let (headers, _hash_version) = scan_ffi_src_head(&src_dir);
    for path in headers {
        builder = builder.header(path);
    }

    let bindings = builder.generate().unwrap();

    let buff = bindings.to_string();
    let buff = filter_by_namespace(&buff);
    let ori_buff = if std::path::Path::new(&tar_file).exists() {
        read_file_to_string(&tar_file, "Couldn't open rust ffi code file")
    } else {
        "".to_string()
    };
    if ori_buff == buff {
        println!("There is no need to overwrite rust ffi code file");
    } else {
        println!("Start generate rust code into {}\n", tar_file);
        let mut file = fs::File::create(tar_file).expect("Couldn't create rust ffi code file");
        file.write(buff.as_bytes()).unwrap();
    }
}

fn main() {
    gen_ffi_code();
}