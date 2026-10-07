#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Ok(s)=core::str::from_utf8(data){if let Ok(path)=pfx_vector_svg::parse_path(s){let tol=pfx_vector_core::Tolerance::default();let _=pfx_vector_core::path_length(&path,tol);let _=pfx_vector_core::flatten_path(&path,tol);let _=pfx_vector_svg::serialize_path(&path,pfx_vector_svg::SerializeOptions::default());}}
});
