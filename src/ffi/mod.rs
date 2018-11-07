#![allow(non_camel_case_types)]
mod consts;
mod types;

use std::ptr;
use std::path::Path;
use std::ffi::CString;
use num_rational::Rational32;

use ffi::types::*;
use ffi::consts::*;

#[derive(Debug)]
pub struct TaggedValue {
    pub name: String,
    pub value: String,
    pub attributes: Vec<(String, String)>,
}

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

    /* UMID */
    pub file_source_package_uid: MXFumid,
    pub physical_source_package_uid: MXFumid,
    pub material_package_uid: MXFumid,

    /* UL */
    pub essence_container_label: MXFkey,
    pub picture_coding_label: MXFkey,

    /* Timstamps */
    pub clip_created: MXFTimestamp,

    /* Avid Tagged Values */
    pub user_comments: Vec<TaggedValue>,
    pub material_package_attributes: Vec<TaggedValue>,

    /* Avid Essence Type */
    pub essenc_type: AvidEssenceType,

    /* Avid Physical Pacakge Type */
    pub physical_package_type: AvidPhysicalPackageType,

    /* Integers */
    pub frame_layout: u8,
    pub stored_width: u32,
    pub stored_height: u32,
    pub display_width: u32,
    pub display_height: u32,
    pub track_duration: i64,
    pub segment_duration: i64,
    pub segment_offset: i64,
    pub start_timecode: i64,
    pub track_number: u32,
    pub channel_count: u32,
    pub quantization_bits: u32,
    pub clip_duration: i64,
    pub is_video: bool,
    pub num_audio_tracks: u32,
    pub num_video_tracks: u32,
}

impl AvidMXFInfo {
    pub fn from_file(filename: &Path) -> Result<AvidMXFInfo, String>  {
        unsafe {
            let mut info = AvidMXFInfo::default();
            let mut mxffile: *mut MXFFile  = ptr::null_mut();
            let mut headerpartition: *mut MXFPartition = ptr::null_mut();
            let mut datamodel: *mut MXFDataModel = ptr::null_mut();
            let mut headerdata: *mut MXFHeaderMetadata = ptr::null_mut();
            //let mut set: *mut MXFHeaderMetadata = ptr::null_mut();
            let mut preface_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut material_package_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut tagged_values: *mut MXFList = ptr::null_mut();
            let mut tagged_names: *mut MXFList = ptr::null_mut();
            let mut tagged_value: *mut uint16 = ptr::null_mut();
            let mut mxful = MXFkey::default();
            let mut llen: uint8 = 0;
            let mut len: uint64 = 0;
            let mob_name = vec![95, 80, 74, 0];

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
            check!(mxf_find_singular_set_by_key, "Could not read preface set.", headerdata, &G_PREFACE_SET_KEY, &mut preface_set);
            if mxf_have_item(preface_set, &G_PREFACE_PROJECTNAME_ITEM_KEY) == 1 {
                info.project_name = AvidMXFInfo::get_string_value(preface_set, &G_PREFACE_PROJECTNAME_ITEM_KEY);
                println!("Project Name {:?}", info.project_name);
            }
            if mxf_have_item(preface_set, &G_PREFACE_PROJECTEDITRATE_ITEM_KEY) == 1 {
                info.project_edit_rate = AvidMXFInfo::get_rational_value(preface_set, &G_PREFACE_PROJECTEDITRATE_ITEM_KEY);
                println!("Project Edit Rate {:?}", info.project_edit_rate);
            }

            /* Get material package and info */
            check!(mxf_find_singular_set_by_key, "Could not read material package set.", headerdata, &G_MATERIALPACKAGE_SET_KEY, &mut material_package_set);
            check!(mxf_get_umid_item, "Could not read package uid.", material_package_set, &G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY, &mut info.material_package_uid);
            if mxf_have_item(material_package_set, &G_GENERICPACKAGE_NAME_ITEM_KEY) == 1 {
                info.clip_name = AvidMXFInfo::get_string_value(material_package_set, &G_GENERICPACKAGE_NAME_ITEM_KEY);
                println!("Clip Name {:?}", info.clip_name);
            }
            if mxf_have_item(material_package_set, &G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY) == 1 {
                mxf_get_timestamp_item(material_package_set, &G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY, &mut info.clip_created);
                println!("Clip Created {:?}", info.clip_created);
            }

            /* Get the material package project name tagged value if not aleady set */
            if info.project_name.is_none() && mxf_have_item(material_package_set, &G_GENERICPACKAGE_MOBATTRIBUTELIST_ITEM_KEY) == 1 {
                check!(mxf_avid_read_string_mob_attributes, "Could not read mob attributes.", material_package_set, &mut tagged_names, &mut tagged_values);
                if mxf_avid_get_mob_attribute(mob_name.as_ptr(), tagged_names, tagged_values, &mut tagged_value) == 1 {
                    info.project_name = AvidMXFInfo::convert_string(tagged_value);
                }
                mxf_free_list(&mut tagged_names);
                mxf_free_list(&mut tagged_values);
            }

            /* Get the material package user comments */
            if mxf_have_item(material_package_set, &G_GENERICPACKAGE_USERCOMMENTS_ITEM_KEY) == 1 {
                info.user_comments = AvidMXFInfo::get_package_tagged_values(material_package_set, &G_GENERICPACKAGE_USERCOMMENTS_ITEM_KEY)?;
                println!("User Comments: {:?}", info.user_comments);
            }

            /* Get the material package attributes */
            if mxf_have_item(material_package_set, &G_GENERICPACKAGE_MOBATTRIBUTELIST_ITEM_KEY) == 1 {
                // TODO: FIX ME!
                info.material_package_attributes = AvidMXFInfo::get_package_tagged_values(material_package_set, &G_GENERICPACKAGE_MOBATTRIBUTELIST_ITEM_KEY)?;
                println!("Material package attributes: {:?}", info.material_package_attributes);
            }

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
            mxf_get_rational_item(set, mxfkey, &mut mxf_rational);
            // TODO: not working yet
            //Some(Rational32::new(mxf_rational.numerator, mxf_rational.denominator))
            None
        }
    }

    fn get_package_tagged_values(set: *mut MXFMetadataSet, mxfkey: *const MXFkey) -> Result<Vec<TaggedValue>, String> {
        unsafe {
            let mut element: *mut uint8 = ptr::null_mut();
            let mut tagged_value_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut tagged_values = Vec::new();
            let mut count: uint32 = 0;

            if mxf_have_item(set, mxfkey) != 1 { return Ok(tagged_values); }
            if mxf_get_array_item_count(set, mxfkey, &mut count) == 0 { return Ok(tagged_values); }
            /* Preallocate Vec */
            tagged_values.reserve(count as usize);

            for i in 0..count {
                /* Retrieve name and value */
                let mut name: *mut uint16 = ptr::null_mut();
                let mut value: *mut uint16 = ptr::null_mut();

                if mxf_get_array_item_element(set, mxfkey, i, &mut element) == 0 {
                    return Err("Tagged value: failed reading array element.".to_string());
                }
                if mxf_get_strongref((*set).header_metadata, element, &mut tagged_value_set) == 0 {
                    return Err("Tagged value: failed reading strongref.".to_string());
                }
                if mxf_avid_read_string_tagged_value(tagged_value_set, &mut name, &mut value) == 0 {
                    return Err("Tagged value: failed reading name and value.".to_string());
                }

                let mut val = TaggedValue {
                    name: {
                        AvidMXFInfo::convert_string(name)
                            .ok_or({
                                libc::free(name as *mut c_void);
                                libc::free(value as *mut c_void);
                                "Tagged value: name is not utf16.".to_string()
                            })?
                    },
                    value: {
                        AvidMXFInfo::convert_string(value)
                            .ok_or({
                                libc::free(name as *mut c_void);
                                libc::free(value as *mut c_void);
                                "Tagged value: value is not utf16.".to_string()
                            })?
                    },
                    attributes: Vec::new(),
                };
                libc::free(name as *mut c_void);
                libc::free(value as *mut c_void);

                /* Check for attributes */
                if mxf_have_item(tagged_value_set, &G_TAGGEDVALUE_TAGGEDVALUEATTRIBUTELIST_ITEM_KEY) == 1 {
                    let mut names: *mut MXFList = ptr::null_mut();
                    let mut values: *mut MXFList = ptr::null_mut();
                    let mut names_iter = MXFListIterator::default();
                    let mut values_iter = MXFListIterator::default();

                    if mxf_avid_read_string_tagged_values(tagged_value_set, &G_TAGGEDVALUE_TAGGEDVALUEATTRIBUTELIST_ITEM_KEY, &mut names, &mut values) == 0 {
                        return Err("Tagged value: failed reading attribute list.".to_string());
                    }
                    val.attributes.reserve(mxf_get_list_length(names) as usize);

                    mxf_initialise_list_iter(&mut names_iter, names);
                    mxf_initialise_list_iter(&mut values_iter, values);
                    while mxf_next_list_iter_element(&mut names_iter) != 0 && mxf_next_list_iter_element(&mut values_iter) != 0 {
                        val.attributes.push(
                            (
                                AvidMXFInfo::convert_string(mxf_get_iter_element(&mut names_iter) as *mut uint16)
                                .ok_or({
                                    mxf_free_list(&mut names);
                                    mxf_free_list(&mut values);
                                    "Tagged value: failed reading name of attribute.".to_string()
                                })?
                                ,
                                AvidMXFInfo::convert_string(mxf_get_iter_element(&mut values_iter) as *mut uint16)
                                .ok_or({
                                    mxf_free_list(&mut names);
                                    mxf_free_list(&mut values);
                                    "Tagged value: failed reading value of attribute.".to_string()
                                })?
                            )
                        );
                    }

                    mxf_free_list(&mut names);
                    mxf_free_list(&mut values);
                }

                tagged_values.push(val);
            }

            Ok(tagged_values)
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

            // NOTE: This should work without clone. Will fix it later.
            String::from_utf8(utf8_str.clone()).ok()
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
    fn mxf_get_umid_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, mxfumid: *mut MXFumid) -> c_int;
    fn mxf_get_timestamp_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, mxftimestamp: *mut MXFTimestamp) -> c_int;
    fn mxf_get_array_item_count(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, count: *mut uint32) -> c_int;
    fn mxf_get_array_item_element(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, index: uint32, element: *mut *mut uint8) -> c_int;
    fn mxf_get_strongref(hederdata: *mut MXFHeaderMetadata, value: *const uint8, dataset: *mut *mut MXFMetadataSet) -> c_int;

    /* mxf_labels_and_keys.h */
    fn mxf_is_op_atom(mxful: *const MXFkey) -> c_int;

    /* mxf_list.h */
    fn mxf_free_list(list: *mut *mut MXFList);
    fn mxf_get_list_length(list: *mut MXFList) -> size_t;
    fn mxf_initialise_list_iter(iter: *mut MXFListIterator, list: *const MXFList);
    fn mxf_next_list_iter_element(iter: *mut MXFListIterator) -> c_int;
    fn mxf_get_iter_element(iter: *mut MXFListIterator) -> *mut c_void;

    /* mxf_data_model.h */
    fn mxf_load_data_model(datamodel: *mut *mut MXFDataModel) -> c_int;
    fn mxf_finalise_data_model(datamodel: *mut MXFDataModel) -> c_int;

    /* mxf_avid.h */
    fn mxf_avid_get_mob_attribute(name: *const uint16, names: *const MXFList, values: *const MXFList, value: *mut *mut uint16) -> c_int;
    fn mxf_avid_read_string_mob_attributes(dataset: *mut MXFMetadataSet, names: *mut *mut MXFList, values: *mut *mut MXFList) -> c_int;
    fn mxf_avid_load_extensions(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_avid_read_string_tagged_values(dataset: *mut MXFMetadataSet, mxfkey: *const MXFkey, names: *mut *mut MXFList, values: *mut *mut MXFList) -> c_int;
    fn mxf_avid_read_string_tagged_value(dataset: *mut MXFMetadataSet, name: *mut *mut uint16, value: *mut *mut uint16) -> c_int;
    fn mxf_avid_read_filtered_header_metadata(mxffile: *mut MXFFile, skip_data_refs: c_int, headerdata: *mut MXFHeaderMetadata,
                                         header_byte_count: uint64, mxfkey: *const MXFkey, llen: uint8, len: uint64) -> c_int;
}
