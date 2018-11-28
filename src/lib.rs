extern crate chrono;
extern crate libc;
extern crate num_rational;
extern crate num_traits;

#[macro_use]
mod macros;
mod ffi;

pub type MXFInfo = ffi::AvidMXFInfo;
pub type MXFKey = ffi::MXFKey;
pub type MXFUmid = ffi::MXFUmid;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use num_rational::Rational32;
    use std::path::PathBuf;

    #[test]
    fn can_retrieve_from_video_file() {
        let sample_path = PathBuf::from("samples");
        let filename = sample_path.join("domdom.mov.V159CD0127V.mxf");
        let mxf = MXFInfo::from_file(filename.as_path()).unwrap();
        let mpuid = MXFUmid::new(
            0x06, 0x0a, 0x2b, 0x34, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x0f, 0x00, 0x13, 0x00,
            0x00, 0x00, 0x59, 0xcd, 0x01, 0x27, 0x87, 0x7c, 0x06, 0x63, 0x06, 0x0e, 0x2b, 0x34,
            0x7f, 0x7f, 0x2a, 0x80,
        );
        let fpuid = MXFUmid::new(
            0x06, 0x0a, 0x2b, 0x34, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x0f, 0x00, 0x13, 0x00,
            0x00, 0x00, 0x59, 0xcd, 0x01, 0x27, 0x89, 0xdf, 0x06, 0x63, 0x06, 0x0e, 0x2b, 0x34,
            0x7f, 0x7f, 0x2a, 0x80,
        );
        let essence_label = MXFKey::new(
            0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x0a, 0x0d, 0x01, 0x03, 0x01, 0x02, 0x11,
            0x02, 0x00,
        );
        let picture_label = MXFKey::new(
            0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x0a, 0x04, 0x01, 0x02, 0x02, 0x71, 0x12,
            0x00, 0x00,
        );

        assert_eq!(mxf.project_name, Some("dom".to_string()), "project name");
        assert_eq!(mxf.project_edit_rate, Some(Rational32::new(50, 1)));
        assert_eq!(mxf.clip_name, Some("domdom.mov".to_string()), "clip name");
        assert_eq!(
            mxf.clip_created,
            Some(NaiveDate::from_ymd(2017, 9, 28).and_hms(14, 3, 19)),
            "created date"
        );
        assert_eq!(mxf.track_duration, Some(49), "track duration");
        assert_eq!(mxf.clip_duration, Some(49), "clip duration");
        assert_eq!(mxf.frame_layout, Some(0), "frame layout");
        assert_eq!(mxf.stored_width, Some(1280), "stored width");
        assert_eq!(mxf.stored_height, Some(720), "stored height");
        assert_eq!(
            mxf.material_package_uid,
            Some(mpuid),
            "material package uid"
        );
        assert_eq!(mxf.file_source_package_uid, Some(fpuid), "file package uid");
        assert_eq!(mxf.start_timecode, 179999, "start timecode");
        assert_eq!(
            mxf.picture_coding_label,
            Some(picture_label),
            "picture coding label"
        );
        assert_eq!(
            mxf.essence_container_label,
            Some(essence_label),
            "essence label"
        );
        assert_eq!(mxf.is_renderfile(), false, "render file");
        assert_eq!(
            mxf.clip_edit_rate,
            Some(Rational32::new(50, 1)),
            "clip edit rate"
        );
        assert_eq!(mxf.video_track_count, 1, "video track count");
        assert_eq!(mxf.audio_track_count, 2, "audio track count");
        assert_eq!(mxf.track_number, Some(1), "track number");
        assert_eq!(
            mxf.physical_package_name,
            Some("domdom.mov".to_string()),
            "physical package name"
        );
    }

    #[test]
    fn can_retrieve_from_audio_file() {
        let sample_path = PathBuf::from("samples");
        let filename = sample_path.join("domdom.mov.A159CD0127A.mxf");
        let mxf = MXFInfo::from_file(filename.as_path()).unwrap();

        assert_eq!(mxf.channel_count, Some(1), "channel count");
        assert_eq!(mxf.quantization_bits, Some(24), "quantization bits");
    }
}
