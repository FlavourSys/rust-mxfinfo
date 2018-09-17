extern crate pkg_config;

fn main() {
    let lib_mxfinfo = pkg_config::probe_library("libMXF-1.0");
    if lib_mxfinfo.is_err() {
        panic!("Could not find MediaInfo via pkgconfig");
    } else {
        // panic!("{:?}", lib_mediainfo);
    }
}
