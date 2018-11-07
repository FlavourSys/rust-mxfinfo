extern crate libc;
extern crate num_rational;

#[macro_use]
mod macros;
mod ffi;

pub type MXFInfo = ffi::AvidMXFInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use num_rational::Rational32;

    #[test]
    fn can_retrieve_from_file() {
        let sample_path = PathBuf::from("samples");
        let filename = sample_path.join("domdom.mov.V159CD0127V.mxf");
        let mxf = MXFInfo::from_file(filename.as_path()).unwrap();
        assert_eq!(mxf.project_name, Some("dom".to_string()));
        assert_eq!(mxf.project_edit_rate, Some(Rational32::new(50, 1)));
        assert_eq!(mxf.clip_name, Some("domdom.mov".to_string()));
        assert_eq!(mxf.clip_created.year, 2017);
    }
}
