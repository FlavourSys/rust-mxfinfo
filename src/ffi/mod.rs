#![allow(non_camel_case_types)]
extern crate libc;
extern crate num_rational;

mod consts;
mod types;

use std::ptr;
use std::path::Path;
use std::ffi::CString;
use self::num_rational::Rational32;

use ffi::types::*;

#[derive(Debug, Default)]
pub struct AvidMXFInfo {
    /* String values */
    pub project_name: Option<String>,
    pub clip_name: Option<String>,
    pub track_string: Option<String>,
    pub physical_package_name: Option<String>,
    pub physical_package_locator: Option<String>,

    /* Rational value */
    pub project_edit_rate: Option<Rational32>,
    pub edit_rate: Option<Rational32>,
    pub aspect_ratio: Option<Rational32>,
    pub audio_sampling_rate: Option<Rational32>,

    /*
    mxfTimestamp clipCreated;
    int64_t clipDuration;
    mxfUMID materialPackageUID;
    AvidTaggedValue *userComments;
    int numUserComments;
    AvidTaggedValue *materialPackageAttributes;
    int numMaterialPackageAttributes;
    int numVideoTracks;
    int numAudioTracks;

    /* track info */
    uint32_t trackNumber;
    int isVideo;
    int64_t trackDuration;
    int64_t segmentDuration;
    int64_t segmentOffset;
    int64_t startTimecode;

    /* file essence info */
    AvidEssenceType essenceType;
    mxfUMID fileSourcePackageUID;
    mxfUL essenceContainerLabel;

    /* picture info */
    mxfUL pictureCodingLabel;
    uint8_t frameLayout;
    uint32_t storedWidth;
    uint32_t storedHeight;
    uint32_t displayWidth;
    uint32_t displayHeight;

    /* sound info */
    uint32_t channelCount;
    uint32_t quantizationBits;

    /* physical source info */
    mxfUMID physicalSourcePackageUID;
    AvidPhysicalPackageType physicalPackageType;
    */
}

impl AvidMXFInfo {
    pub fn from_file(filename: &Path) -> Result<AvidMXFInfo, String>  {
        unsafe {
            let mut info = AvidMXFInfo::default();
            let mut mxffile: *mut MXFFile  = ptr::null_mut();
            let mut headerpartition: *mut MXFPartition = ptr::null_mut();
            let mut datamodel: *mut MXFDataModel = ptr::null_mut();
            let mut headerdata: *mut MXFHeaderMetadata = ptr::null_mut();
            let mut set: *mut MXFHeaderMetadata = ptr::null_mut();
            let mut preface_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut material_package_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut mxful = MXFkey::default();
            let mut llen: uint8 = 0;
            let mut len: uint64 = 0;

            /* Convert filepath into CString */
            let filename = filename.to_str().ok_or("Filename is not UTF-8.".to_string())?;
            let filename = CString::new(filename).map_err(|_| "Filename not CString compatible.".to_string())?;

            /* Open file */
            check!(mxf_disk_file_open_read, "Could not open file.", filename.as_ptr(), &mut mxffile);
            /* Read header and partition */
            check!(mxf_read_header_pp_kl, "Could not read header.", mxffile, &mut mxful, &mut llen, &mut len);
            check!(mxf_read_partition, "Could not read header partition.", mxffile, &mxful, &mut headerpartition);
            /* Check if OP-Atom */
            check!(mxf_is_op_atom, "Is not OP-Atom.", &headerpartition.as_ref().unwrap().operational_pattern);
            /* Load datamodel */
            check!(mxf_load_data_model, "Could not load datamodel.", &mut datamodel);
            check!(mxf_avid_load_extensions, "Could not load avid extensions.", datamodel);
            check!(mxf_finalise_data_model, "Could not finalize datamodel.", datamodel);
            check!(mxf_read_next_nonfiller_kl, "Could not read next nonfiller.", mxffile, &mut mxful, &mut llen, &mut len);
            check!(mxf_is_header_metadata, "Is not header metadata.", &mut mxful);
            check!(mxf_create_header_metadata, "Could not read header metadata.", &mut headerdata, datamodel);
            check!(mxf_avid_read_filtered_header_metadata, "Could not read filtered header metadata.", mxffile, 0,
                   headerdata, headerpartition.as_ref().unwrap().header_byte_count, &mxful, llen, len);

            /* Get preface and info */
            check!(mxf_find_singular_set_by_key, "Could not read preface set.", headerdata, &consts::G_PREFACE_SET_KEY, &mut preface_set);
            if mxf_have_item(preface_set, &consts::G_PREFACE_PROJECTNAME_ITEM_KEY) == 1 {
                info.project_name = AvidMXFInfo::get_string_value(preface_set, &consts::G_PREFACE_PROJECTNAME_ITEM_KEY);
            }
            if mxf_have_item(preface_set, &consts::G_PREFACE_PROJECTEDITRATE_ITEM_KEY) == 1 {
                info.project_edit_rate = AvidMXFInfo::get_rational_value(preface_set, &consts::G_PREFACE_PROJECTEDITRATE_ITEM_KEY);
            }

            /* Get material package and info */
            check!(mxf_find_singular_set_by_key, "Could not read material package set.", headerdata, &consts::G_MATERIALPACKAGE_SET_KEY, &mut material_package_set);

            return Ok(info);
        }
    }

    fn get_string_value(set: *mut MXFMetadataSet, mxfkey: *const MXFkey) -> Option<String> {
        unsafe {
            let mut utf16size: uint16 = 0;
            if mxf_get_utf16string_item_size(set, mxfkey, &mut utf16size) == 0 {
                return None;
            }
            let mut utf16str: Vec<uint16> = vec![0; utf16size as usize];
            let u_ptr = utf16str.as_mut_ptr();
            if mxf_get_utf16string_item(set, mxfkey, u_ptr) == 0 {
                return None;
            }

            AvidMXFInfo::convert_string(u_ptr)
        }
    }

    fn get_rational_value(set: *mut MXFMetadataSet, mxfkey: *const MXFkey) -> Option<Rational32> {
        unsafe {
            let mut mxf_rational = MXFRational::default();
            let mut mxf_rational = vec![0; 10];
            mxf_get_rational_item(set, mxfkey, mxf_rational.as_mut_ptr() as *mut MXFRational);
            // TODO: not working yet
            //Some(Rational32::new(mxf_rational.numerator, mxf_rational.denominator))
            None
        }
    }

    fn convert_string(utf16str: *const uint16) -> Option<String> {
        unsafe {
            let utf8_str = ptr::null_mut();
            let size = mxf_utf16_to_utf8(utf8_str, utf16str, 0);
            if size == std::u64::MAX {
                return None;
            }

            let mut utf8_str: Vec<u8> = vec![0; size as usize];
            mxf_utf16_to_utf8(utf8_str.as_mut_ptr(), utf16str, size);

            String::from_utf8(utf8_str).ok()
        }
    }
}

extern "C" {
    /* mxf_file.h */
    fn mxf_disk_file_open_read(filename: *const c_char, mxffile: *mut *mut MXFFile) -> c_int;

    /* mxf_partition.h*/
    fn mxf_read_header_pp_kl(mxffile: *mut MXFFile, mxfkey: *mut MXFkey, llen: *mut uint8, len: *mut uint64) -> c_int;
    fn mxf_read_partition(mxffile: *mut MXFFile, mxfkey: *const MXFkey, mxfpartition: *mut *mut MXFPartition) -> c_int;
    fn mxf_read_next_nonfiller_kl(mxffile: *mut MXFFile, mxfkey: *const MXFkey, llen: *mut uint8, len: *mut uint64) -> c_int;

    /* mxf_utils.h */
    fn mxf_utf16_to_utf8(u8_str: *mut u8, u16_str: *const uint16, u8_size: size_t) -> size_t;

    /* mxf_header_metadata.h */
    fn mxf_is_header_metadata(mxfkey: *const MXFkey) -> c_int;
    fn mxf_create_header_metadata(headerdata: *mut *mut MXFHeaderMetadata, datamodel: *const MXFDataModel) -> c_int;
    fn mxf_find_singular_set_by_key(headerdata: *mut MXFHeaderMetadata, mxfkey: *const MXFkey, dataset: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_have_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey) -> c_int;
    fn mxf_get_utf16string_item_size(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, size: *mut uint16) -> c_int;
    fn mxf_get_utf16string_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, value: *mut uint16) -> c_int;
    fn mxf_get_rational_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, mxffractional: *mut MXFRational) -> c_int;

    /* mxf_labels_and_keys.h */
    fn mxf_is_op_atom(mxful: *const MXFkey) -> c_int;

    /* mxf_data_model.h */
    fn mxf_load_data_model(datamodel: *mut *mut MXFDataModel) -> c_int;
    fn mxf_finalise_data_model(datamodel: *mut MXFDataModel) -> c_int;

    /* mxf_avid.h */
    fn mxf_avid_load_extensions(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_avid_read_filtered_header_metadata(mxffile: *mut MXFFile, skip_data_refs: c_int, headerdata: *mut MXFHeaderMetadata,
                                         header_byte_count: uint64, mxfkey: *const MXFkey, llen: uint8, len: uint64) -> c_int;
}
