#![allow(non_camel_case_types)]
mod consts;
pub mod types;

use std::ptr;
use std::path::Path;
use std::ffi::CString;
use num_rational::Rational32;
use num_traits::Zero;

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
    pub clip_edit_rate: Option<Rational32>,
    pub aspect_ratio: Option<Rational32>,
    pub audio_sampling_rate: Option<Rational32>,

    /* UMID */
    pub file_source_package_uid: MXFUmid,
    pub physical_source_package_uid: MXFUmid,
    pub material_package_uid: MXFUmid,

    /* UL */
    pub essence_container_label: MXFKey,
    pub picture_coding_label: MXFKey,

    /* Timstamps */
    pub clip_created: MXFTimestamp,

    /* Avid Tagged Values */
    pub user_comments: Vec<TaggedValue>,
    pub material_package_attributes: Vec<TaggedValue>,

    /* Avid Essence Type */
    pub essence_type: AvidEssenceType,

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
    pub audio_track_count: u32,
    pub video_track_count: u32,
}

impl AvidMXFInfo {
    pub fn from_file(filename: &Path) -> Result<AvidMXFInfo, String>  {
        unsafe {
            let mut info = AvidMXFInfo::default();
            let mut mxffile: *mut MXFFile  = ptr::null_mut();
            let mut headerpartition: *mut MXFPartition = ptr::null_mut();
            let mut datamodel: *mut MXFDataModel = ptr::null_mut();
            let mut headerdata: *mut MXFHeaderMetadata = ptr::null_mut();
            let mut preface_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut material_package_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut file_source_package_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut material_package_track_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut descriptor_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut sequence_set: *mut MXFMetadataSet = ptr::null_mut();
            let mut tagged_values: *mut MXFList = ptr::null_mut();
            let mut tagged_names: *mut MXFList = ptr::null_mut();
            let mut tagged_value: *mut uint16 = ptr::null_mut();
            let mut package_uid = MXFUmid::default();
            let mut datadef = MXFKey::default();
            let mut array_iter = MXFArrayItemIterator::default();
            let mut mxful = MXFKey::default();
            let mut llen: uint8 = 0;
            let mut len: uint64 = 0;
            let mut avid_resolution_id = 0;
            let mut track_number = 0;
            let mut max_duration = 0;
            let mut max_edit_rate = Rational32::new(25, 1);
            let mut track_duration = 0;
            let mut segment_duration = 0;
            let mut segment_offset = 0;
            let mut edit_rate = None;
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
            if !mxf_have_item(preface_set, &G_PREFACE_PROJECTNAME_ITEM_KEY).is_zero() {
                info.project_name = AvidMXFInfo::get_string_value(preface_set, &G_PREFACE_PROJECTNAME_ITEM_KEY);
            }
            if !mxf_have_item(preface_set, &G_PREFACE_PROJECTEDITRATE_ITEM_KEY).is_zero() {
                info.project_edit_rate = AvidMXFInfo::get_rational_value(preface_set, &G_PREFACE_PROJECTEDITRATE_ITEM_KEY);
            }

            /* Get essence label */
            info.essence_container_label = (*(mxf_get_list_element(&mut (*headerpartition).essence_containers, 0) as *mut MXFKey)).clone();

            /* Get material package and info */
            check!(mxf_find_singular_set_by_key, "Could not read material package set.", headerdata, &G_MATERIALPACKAGE_SET_KEY, &mut material_package_set);
            check!(mxf_get_umid_item, "Could not read package uid.", material_package_set, &G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY, &mut info.material_package_uid);
            if !mxf_have_item(material_package_set, &G_GENERICPACKAGE_NAME_ITEM_KEY).is_zero() {
                info.clip_name = AvidMXFInfo::get_string_value(material_package_set, &G_GENERICPACKAGE_NAME_ITEM_KEY);
            }
            if !mxf_have_item(material_package_set, &G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY).is_zero() {
                mxf_get_timestamp_item(material_package_set, &G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY, &mut info.clip_created);
            }

            /* Get the material package project name tagged value if not aleady set */
            if info.project_name.is_none() && mxf_have_item(material_package_set, &G_GENERICPACKAGE_MOBATTRIBUTELIST_ITEM_KEY).is_zero() {
                check!(mxf_avid_read_string_mob_attributes, "Could not read mob attributes.", material_package_set, &mut tagged_names, &mut tagged_values);
                if !mxf_avid_get_mob_attribute(mob_name.as_ptr(), tagged_names, tagged_values, &mut tagged_value).is_zero() {
                    info.project_name = AvidMXFInfo::convert_string(tagged_value);
                }
                mxf_free_list(&mut tagged_names);
                mxf_free_list(&mut tagged_values);
            }

            /* Get the material package user comments */
            if !mxf_have_item(material_package_set, &G_GENERICPACKAGE_USERCOMMENTS_ITEM_KEY).is_zero() {
                info.user_comments = AvidMXFInfo::get_package_tagged_values(material_package_set, &G_GENERICPACKAGE_USERCOMMENTS_ITEM_KEY)?;
            }

            /* Get the top level file source package and info */
            check!(mxf_uu_get_top_file_package, "Could not read top level file pacakge", headerdata, &mut file_source_package_set);
            check!(mxf_get_umid_item, "Could not read file source package uid", file_source_package_set, &G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY, &mut info.file_source_package_uid);

            /* Get the fiel source package essence descriptor info */
            check!(mxf_get_strongref_item, "Could not read descriptor set.", file_source_package_set, &G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY, &mut descriptor_set);
            if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_GENERICPICTUREESSENCEDESCRIPTOR_SET_KEY).is_zero() {
                /* Image Aspect Ratio */
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_ASPECTRATIO_ITEM_KEY).is_zero() {
                    info.aspect_ratio = AvidMXFInfo::get_rational_value(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_ASPECTRATIO_ITEM_KEY);
                }

                /* Frame Layout */
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_FRAMELAYOUT_ITEM_KEY).is_zero() {
                    mxf_get_uint8_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_FRAMELAYOUT_ITEM_KEY, &mut info.frame_layout);
                }

                /* Stored witdth and height */
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDHEIGHT_ITEM_KEY).is_zero() {
                    mxf_get_uint32_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDHEIGHT_ITEM_KEY, &mut info.stored_height);
                }
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDWIDTH_ITEM_KEY).is_zero() {
                    mxf_get_uint32_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDWIDTH_ITEM_KEY, &mut info.stored_width);
                }

                /* Avid Resolution ID */
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_RESOLUTIONID_ITEM_KEY).is_zero() {
                    mxf_get_int32_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_RESOLUTIONID_ITEM_KEY, &mut avid_resolution_id);
                }

                /* Picture essence coding label */
                if !mxf_have_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_PICTUREESSENCECODING_ITEM_KEY).is_zero() {
                    mxf_get_ul_item(descriptor_set, &G_GENERICPICTUREESSENCEDESCRIPTOR_PICTUREESSENCECODING_ITEM_KEY, &mut info.picture_coding_label);
                }
            } else if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_GENERICSOUNDESSENCEDESCRIPTOR_SET_KEY).is_zero() {
                /* Audio Sampling Rate */
                if !mxf_have_item(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_AUDIOSAMPLINGRATE_ITEM_KEY).is_zero() {
                    info.audio_sampling_rate = AvidMXFInfo::get_rational_value(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_AUDIOSAMPLINGRATE_ITEM_KEY);
                }

                /* Quantization bits */
                if !mxf_have_item(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_QUANTIZATIONBITS_ITEM_KEY).is_zero() {
                    mxf_get_uint32_item(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_QUANTIZATIONBITS_ITEM_KEY, &mut info.quantization_bits);
                }

                /* Channel Count */
                if !mxf_have_item(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_CHANNELCOUNT_ITEM_KEY).is_zero() {
                    mxf_get_uint32_item(descriptor_set, &G_GENERICSOUNDESSENCEDESCRIPTOR_CHANNELCOUNT_ITEM_KEY, &mut info.channel_count);
                }
            }

            /* Get the material track referencing the file source package and info */
            check!(mxf_uu_get_package_tracks, "Could not read material track reference.", material_package_set, &mut array_iter);
            while !mxf_uu_next_track(headerdata, &mut array_iter, &mut material_package_track_set).is_zero() {
                check!(mxf_uu_get_track_datadef, "Could not read track data reference.", material_package_track_set, &mut datadef);
                /* Some Avid files have a weak reference to a data definition instead of a UL */
                if mxf_is_picture(&mut datadef).is_zero() && mxf_is_sound(&mut datadef).is_zero() && mxf_is_timecode(&mut datadef).is_zero() {
                    if mxf_avid_get_data_def(headerdata, &mut datadef, &mut datadef).is_zero() {
                        continue;
                    }
                }
                /* Skip non-video and non-audio tracks */
                if mxf_is_picture(&mut datadef).is_zero() && mxf_is_sound(&mut datadef).is_zero() {
                    continue;
                }
                /* Track counts */
                if !mxf_is_picture(&mut datadef).is_zero() {
                    info.video_track_count += 1;
                }
                if !mxf_is_sound(&mut datadef).is_zero() {
                    info.audio_track_count += 1;
                }
                /* Track number */
                if !mxf_have_item(material_package_track_set, &G_GENERICTRACK_TRACKNUMBER_ITEM_KEY).is_zero() {
                    check!(mxf_get_uint32_item, "Could not read track number.", material_package_track_set, &G_GENERICTRACK_TRACKNUMBER_ITEM_KEY, &mut track_number);
                } else {
                    track_number = 0;
                }
                /* Edit rate */
                edit_rate = AvidMXFInfo::get_rational_value(material_package_track_set, &G_TRACK_EDITRATE_ITEM_KEY);
                /* Assume the project edit rate is the video edit rate if not set */
                if info.project_edit_rate.is_none() && !mxf_is_picture(&mut datadef).is_zero() {
                    info.project_edit_rate = edit_rate;
                }
                /* Track duration */
                check!(mxf_uu_get_track_duration, "Could not read track duration.", material_package_track_set, &mut track_duration);
                if edit_rate.is_some() && AvidMXFInfo::compare_length(&max_edit_rate, max_duration, &edit_rate.unwrap(), track_duration) <= 0 {
                    max_edit_rate = edit_rate.unwrap();
                    max_duration = track_duration;
                }
                /* Get info from this track if it refrences the file source package through a
                 * source clip */
                check!(mxf_get_strongref_item, "Could not read generic track sequence.", material_package_track_set, &G_GENERICTRACK_SEQUENCE_ITEM_KEY, &mut sequence_set);
                if mxf_is_subclass_of((*(*sequence_set).header_metadata).datamodel, &mut (*sequence_set).key, &G_SOURCECLIP_SET_KEY).is_zero() {
                    /* Iterate through sequence comonents */
                    let mut count = 0;
                    check!(mxf_get_array_item_count, "Could not read structural component array count.", sequence_set, &G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, &mut count);
                    for i in 0..count {
                        let mut elem = ptr::null_mut();
                        let mut source_clip_set = ptr::null_mut();

                        check!(mxf_get_array_item_element, "Could not read sequence array element.", sequence_set, &G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, i, &mut elem);
                        if mxf_get_strongref((*sequence_set).header_metadata, elem, &mut source_clip_set).is_zero() {
                            /* Dark set not registered in dictionary */
                            continue;
                        }

                        check!(mxf_get_length_item, "Could not read length of item.", source_clip_set, &G_STRUCTURALCOMPONENT_DURATION_ITEM_KEY, &mut segment_duration);
                        if !mxf_is_subclass_of((*(*source_clip_set).header_metadata).datamodel, &mut (*source_clip_set).key, &G_ESSENCEGROUP_SET_KEY).is_zero() {
                            let mut choices_count = 0;
                            let mut final_idx = 0;

                            /* Is an essence group - iterate through choices */
                            check!(mxf_get_array_item_count, "Could not read array essence group choices array count.", source_clip_set,  &G_ESSENCEGROUP_CHOICES_ITEM_KEY, &mut choices_count);
                            for j in 0..choices_count {
                                final_idx = j;
                                check!(mxf_get_array_item_element, "Could not read essence group choices.", source_clip_set, &G_ESSENCEGROUP_CHOICES_ITEM_KEY, j, &mut elem);
                                if mxf_get_strongref((*source_clip_set).header_metadata, elem, &mut source_clip_set).is_zero() {
                                    /* Dark set not registered in dictionary */
                                    continue;
                                }

                                if !mxf_is_subclass_of((*(*source_clip_set).header_metadata).datamodel, &mut (*source_clip_set).key, &G_SOURCECLIP_SET_KEY).is_zero() {
                                    check!(mxf_get_umid_item, "Could not read source package id.", source_clip_set, &G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY, &mut package_uid);
                                    if !mxf_equals_umid(&mut info.file_source_package_uid, &mut package_uid).is_zero() {
                                        /* Found source clip referencing file source package */
                                        break;
                                    }
                                }
                            }
                            if final_idx < choices_count {
                                /* Found source clip referencing file source package */
                                break;
                            }
                        } else if !mxf_is_subclass_of((*(*source_clip_set).header_metadata).datamodel, &mut (*source_clip_set).key, &G_SOURCECLIP_SET_KEY).is_zero() {
                            check!(mxf_get_umid_item, "Could not read source package id.", source_clip_set, &G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY, &mut package_uid);
                            if !mxf_equals_umid(&mut info.file_source_package_uid, &mut package_uid).is_zero() {
                                /* Found source clip referencing file source package */
                                break;
                            }
                        }

                        segment_offset += segment_duration;
                    }
                } else {
                    check!(mxf_get_umid_item, "Could not read source package id.", sequence_set, &G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY, &mut package_uid);
                    check!(mxf_get_length_item, "Could not read segment duration.", sequence_set, &G_STRUCTURALCOMPONENT_DURATION_ITEM_KEY, &mut segment_duration);
                    segment_offset = 0;
                }

                if !mxf_equals_umid(&info.file_source_package_uid, &package_uid).is_zero() {
                    info.is_video = !mxf_is_picture(&mut datadef).is_zero();
                    info.clip_edit_rate = edit_rate;
                    info.track_duration = track_duration;
                    info.track_number = track_number;
                }
            }

            info.clip_duration = AvidMXFInfo::convert_length(&info.project_edit_rate.expect("Project edit rate was empty."), &max_edit_rate, max_duration);

            /* Get the physical source package and info */
            let mut list: *mut MXFList = ptr::null_mut();
            let mut list_iter = MXFListIterator::default();
            check!(mxf_find_set_by_key, "Could not find source package set.", headerdata, &G_SOURCEPACKAGE_SET_KEY, &mut list);
            mxf_initialise_list_iter(&mut list_iter, list);
            while !mxf_next_list_iter_element(&mut list_iter).is_zero() {
                let set = mxf_get_iter_element(&mut list_iter) as (*mut MXFMetadataSet);

                /* The Physical source package is the source package that references a physical
                 * descriptor */
                if !mxf_have_item(set, &G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY).is_zero() {
                    if !mxf_get_strongref_item(set, &G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY, &mut descriptor_set).is_zero() {
                        /* Get first physical package network locator */
                        if !mxf_have_item(descriptor_set, &G_GENERICDESCRIPTOR_LOCATORS_ITEM_KEY).is_zero() {
                            check!(mxf_initialise_array_item_iterator, "Could not initialize array item iterator.", descriptor_set, &G_GENERICDESCRIPTOR_LOCATORS_ITEM_KEY, &mut array_iter);
                            let mut element: *mut uint8 = ptr::null_mut();
                            let mut length = 0;
                            let mut locator_set = ptr::null_mut();
                            while !mxf_next_array_item_element(&mut array_iter, &mut element, &mut length).is_zero() {
                                check!(mxf_get_strongref, "Could not read locator set.", headerdata, element, &mut locator_set);
                                if !mxf_is_subclass_of((*headerdata).datamodel, &mut (*locator_set).key, &G_NETWORKLOCATOR_SET_KEY).is_zero() {
                                    info.physical_package_locator = AvidMXFInfo::get_string_value(locator_set, &G_NETWORKLOCATOR_URLSTRING_ITEM_KEY);
                                }
                            }
                        }

                        /* NOTE: Some descriptors could be dark and so we don't assume we can
                         * dereference */
                        if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_PHYSICALDESCRIPTOR_SET_KEY).is_zero() {
                            if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_TAPEDESCRIPTOR_SET_KEY).is_zero() {
                                info.physical_package_type = AvidPhysicalPackageType::TapePhysType;
                            } else if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_IMPORTDESCRIPTOR_SET_KEY).is_zero() {
                                info.physical_package_type = AvidPhysicalPackageType::ImportPhysType;
                            } else if !mxf_is_subclass_of(datamodel, &mut (*descriptor_set).key, &G_RECORDINGDESCRIPTOR_SET_KEY).is_zero() {
                                info.physical_package_type = AvidPhysicalPackageType::RecordingPhysType;
                            } else {
                                info.physical_package_type = AvidPhysicalPackageType::UnkownPhysType;
                            }

                            check!(mxf_get_umid_item, "Could not read physical source package uid.", set, &G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY, &mut info.physical_source_package_uid);
                            if !mxf_have_item(set, &G_GENERICPACKAGE_NAME_ITEM_KEY).is_zero() {
                                info.physical_package_name = AvidMXFInfo::get_string_value(set, &G_GENERICPACKAGE_NAME_ITEM_KEY);
                            }

                            break;
                        }
                    }
                }
            }
            mxf_free_list(&mut list);

            /* Get the start timecode
             * the source timecode is calculated using the SourceClip::start_position in the file
             * source package in conjunction with the TimecodeComponent in the referenced physical
             * source package */
            let mut has_timecode = false;
            let mut track_set = ptr::null_mut();
            check!(mxf_uu_get_package_tracks, "Could not read package tracks.", file_source_package_set, &mut array_iter);
            while !has_timecode && !mxf_uu_next_track(headerdata, &mut array_iter, &mut track_set).is_zero() {
                /* Skip non-timecode tracks */
                check!(mxf_uu_get_track_datadef, "Could not read datadef of trackset.", track_set, &mut datadef);
                /* Some Avid files have a weak reference to a data definition instead of a UL */
                if mxf_is_picture(&mut datadef).is_zero() && mxf_is_sound(&mut datadef).is_zero() && mxf_is_timecode(&mut datadef).is_zero() {
                    if mxf_avid_get_data_def(headerdata, &mut datadef, &mut datadef).is_zero() {
                        continue;
                    }
                }

                if mxf_is_picture(&mut datadef).is_zero() && mxf_is_sound(&mut datadef).is_zero() {
                    continue;
                }

                /* Get the timecode component */
                let mut source_clip_set = ptr::null_mut();
                if !AvidMXFInfo::get_single_track_component(track_set, &G_SOURCECLIP_SET_KEY, &mut source_clip_set) {
                    continue;
                }

                let package_edit_rate = AvidMXFInfo::get_rational_value(track_set, &G_TRACK_EDITRATE_ITEM_KEY);
                let mut package_start_pos = 0;
                check!(mxf_get_position_item, "Could not read track start position.", source_clip_set, &G_SOURCECLIP_STARTPOSITION_ITEM_KEY, &mut package_start_pos);
                /* Get the package referenced by the source clip */
                let mut source_package_id = MXFUmid::default();
                let mut ref_source_package_set = ptr::null_mut();
                check!(mxf_get_umid_item, "Could not read source package id.", source_clip_set, &G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY, &mut source_package_id);
                if !mxf_equals_umid(&mut MXFUmid::default(), &mut source_package_id).is_zero() || mxf_uu_get_referenced_package((*source_clip_set).header_metadata, &mut source_package_id, &mut ref_source_package_set).is_zero() {
                    /* Either at the end of the chain or doesn't have the referenced package */
                    continue;
                }

                /* find the timecode component in the physical source package and calculate the
                 * start timecode */
                check!(mxf_uu_get_package_tracks, "Could not read package track iterator.", ref_source_package_set, &mut array_iter);
                while !mxf_uu_next_track(headerdata, &mut array_iter, &mut track_set).is_zero() {
                    /* Skip non-timecode tracks */
                    check!(mxf_uu_get_track_datadef, "Could not read track datadef.", track_set, &mut datadef);

                    /* Some Avid files have a weak reference to a data definition instead of a UL */
                    if mxf_is_picture(&mut datadef).is_zero() && mxf_is_sound(&mut datadef).is_zero() && mxf_is_timecode(&mut datadef).is_zero() {
                        if mxf_avid_get_data_def(headerdata, &mut datadef, &mut datadef).is_zero() {
                            continue;
                        }
                    }
                    if mxf_is_timecode(&mut datadef).is_zero() {
                        continue;
                    }

                    /* Get the timecode component */
                    let mut time_code_component_set = ptr::null_mut();
                    if !AvidMXFInfo::get_single_track_component(track_set, &G_TIMECODECOMPONENT_SET_KEY, &mut time_code_component_set) {
                        continue;
                    }

                    /* Get the start timecode and rouded timecode base for the timecode component.
                     */
                    let mut start_timecode = 0;
                    let mut timecode_base = 0;
                    check!(mxf_get_position_item, "Could not read timecode track position item.", time_code_component_set, &G_TIMECODECOMPONENT_STARTTIMECODE_ITEM_KEY, &mut start_timecode);
                    check!(mxf_get_uint16_item, "Could not read timecode base item.", time_code_component_set, &G_TIMECODECOMPONENT_ROUNDEDTIMECODEBASE_ITEM_KEY, &mut timecode_base);

                    if info.clip_edit_rate.is_none() || package_edit_rate.is_none() {
                        return Err("Clip edit rate is empty for timecode calculation.".to_string());
                    }
                    let c_edit_rate = info.clip_edit_rate.as_ref().unwrap();
                    let p_edit_rate = package_edit_rate.as_ref().unwrap();

                    let mut start_pos = package_start_pos;
                    if start_timecode > 0 {
                        let tmp_timecode_base = (p_edit_rate.numer().clone() as f64 / p_edit_rate.denom().clone() as f64 + 0.5) as u16;
                        if tmp_timecode_base == timecode_base {
                            start_pos += start_timecode;
                        } else if tmp_timecode_base == 2 * timecode_base {
                            start_pos += 2 * start_timecode;
                        } else {
                            /* TODO: Complete support for different timecode and track edit rates.
                             * */
                        }
                    }


                    info.start_timecode = ((start_pos * c_edit_rate.numer().clone() as i64 * p_edit_rate.denom().clone() as i64) as f64 /
                        (c_edit_rate.denom().clone() * p_edit_rate.numer().clone()) as f64 + 0.5) as i64;

                    has_timecode = true;
                    break;
                }
            }

            return Ok(info);
        }
    }

    pub fn is_renderfile(&self) -> bool { self.physical_package_name == Some("Precompute Source Mob".to_string()) }

    fn get_string_value(set: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> Option<String> {
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

    fn get_rational_value(set: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> Option<Rational32> {
        unsafe {
            let mut mxf_rational = MXFRational::default();
            if mxf_get_rational_item(set, mxfkey, &mut mxf_rational).is_zero() || mxf_rational.denominator.is_zero() {
                None
            } else {
                Some(Rational32::new(mxf_rational.numerator, mxf_rational.denominator))
            }
        }
    }

    fn get_package_tagged_values(set: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> Result<Vec<TaggedValue>, String> {
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
                if !mxf_have_item(tagged_value_set, &G_TAGGEDVALUE_TAGGEDVALUEATTRIBUTELIST_ITEM_KEY).is_zero() {
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
                                AvidMXFInfo::convert_string(mxf_get_iter_element(&mut names_iter) as *const uint16)
                                .ok_or({
                                    mxf_free_list(&mut names);
                                    mxf_free_list(&mut values);
                                    "Tagged value: failed reading name of attribute.".to_string()
                                })?
                                ,
                                AvidMXFInfo::convert_string(mxf_get_iter_element(&mut values_iter) as *const uint16)
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

    fn get_single_track_component(track_set: *mut MXFMetadataSet, mxfkey: *const MXFKey, source_clip_set: *mut *mut MXFMetadataSet) -> bool {
        unsafe {
            let mut sequence_set = ptr::null_mut();
            let mut component_set = ptr::null_mut();
            let mut component_count = 0;
            let mut value = ptr::null_mut();

            if mxf_get_strongref_item(track_set, &G_GENERICTRACK_SEQUENCE_ITEM_KEY, &mut sequence_set).is_zero() {
                return false;
            }

            if !mxf_set_is_subclass_of(sequence_set, &G_SEQUENCE_SET_KEY).is_zero() {
                /* Is a sequence so we get the first component */
                if mxf_get_array_item_count(sequence_set, &G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, &mut component_count).is_zero() {
                    return false;
                }
                if component_count != 1 {
                    return false;
                }

                if mxf_get_array_item_element(sequence_set, &G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, 0, &mut value).is_zero() {
                    return false;
                }
                if mxf_get_strongref((*track_set).header_metadata, value, &mut component_set).is_zero() {
                    return false;
                }
            } else {
                /* Something other than a sequence */
                component_set = sequence_set;
            }

            if mxf_set_is_subclass_of(component_set, mxfkey).is_zero() {
                /* not a component set key component */
                return false;
            }

            (*source_clip_set) = component_set;

            return true;
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

    fn convert_length(target_edit_rate: &Rational32, edit_rate: &Rational32, length: int64) -> int64 {
        ((length * target_edit_rate.numer().clone() as i64 * edit_rate.denom().clone() as i64) as f64 / (target_edit_rate.denom() * edit_rate.numer()) as f64 + 0.5) as int64
    }

    fn compare_length(edit_rate_a: &Rational32, length_a: int64, edit_rate_b: &Rational32, length_b: int64) -> int64 {
        length_a - AvidMXFInfo::convert_length(edit_rate_a, edit_rate_b, length_b)
    }
}

extern "C" {
    /* mxf_file.h */
    fn mxf_disk_file_open_read(filename: *const c_char, mxffile: *mut *mut MXFFile) -> c_int;

    /* mxf_partition.h*/
    fn mxf_read_header_pp_kl(mxffile: *mut MXFFile, mxfkey: *mut MXFKey, llen: *mut uint8, len: *mut uint64) -> c_int;
    fn mxf_read_partition(mxffile: *mut MXFFile, mxfkey: *const MXFKey, mxfpartition: *mut *mut MXFPartition) -> c_int;
    fn mxf_read_next_nonfiller_kl(mxffile: *mut MXFFile, mxfkey: *const MXFKey, llen: *mut uint8, len: *mut uint64) -> c_int;

    /* mxf_utils.h */
    fn mxf_utf16_to_utf8(u8_str: *mut u8, u16_str: *const uint16, u8_size: size_t) -> size_t;
    fn mxf_equals_umid(umid_a: *const MXFUmid, umid_b: *const MXFUmid) -> c_int;

    /* mxf_header_metadata.h */
    fn mxf_is_header_metadata(mxfkey: *const MXFKey) -> c_int;
    fn mxf_create_header_metadata(headerdata: *mut *mut MXFHeaderMetadata, datamodel: *const MXFDataModel) -> c_int;
    fn mxf_find_singular_set_by_key(headerdata: *mut MXFHeaderMetadata, mxfkey: *const MXFKey, dataset: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_find_set_by_key(headerdata: *mut MXFHeaderMetadata, mxfkey: *const MXFKey, list: *mut *mut MXFList) -> c_int;
    fn mxf_have_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> c_int;
    fn mxf_get_utf16string_item_size(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, size: *mut uint16) -> c_int;
    fn mxf_get_utf16string_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut uint16) -> c_int;
    fn mxf_get_rational_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, mxffractional: *mut MXFRational) -> c_int;
    fn mxf_get_uint8_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut uint8) -> c_int;
    fn mxf_get_uint16_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut uint16) -> c_int;
    fn mxf_get_uint32_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut uint32) -> c_int;
    fn mxf_get_int32_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut int32) -> c_int;
    fn mxf_get_umid_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, mxfumid: *mut MXFUmid) -> c_int;
    fn mxf_get_timestamp_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, mxftimestamp: *mut MXFTimestamp) -> c_int;
    fn mxf_get_array_item_count(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, count: *mut uint32) -> c_int;
    fn mxf_get_array_item_element(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, index: uint32, element: *mut *mut uint8) -> c_int;
    fn mxf_next_array_item_element(iter: *mut MXFArrayItemIterator, value: *mut *mut uint8, len: *mut uint32) -> c_int;
    fn mxf_initialise_array_item_iterator(set: *mut MXFMetadataSet, mxfkey: *const MXFKey, iter: *mut MXFArrayItemIterator) -> c_int;
    fn mxf_get_strongref(hederdata: *mut MXFHeaderMetadata, value: *const uint8, dataset: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_get_strongref_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, value: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_get_ul_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, mxful: *mut MXFKey) -> c_int;
    fn mxf_get_length_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, val: *mut MXFLength) -> c_int;
    fn mxf_get_position_item(datset: *mut MXFMetadataSet, mxfkey: *const MXFKey, pos: *mut int64) -> c_int;
    fn mxf_set_is_subclass_of(set: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> c_int;


    /* mxf_labels_and_keys.h */
    fn mxf_is_op_atom(mxful: *const MXFKey) -> c_int;

    /* mxf_list.h */
    fn mxf_free_list(list: *mut *mut MXFList);
    fn mxf_get_list_length(list: *mut MXFList) -> size_t;
    fn mxf_get_list_element(list: *mut MXFList, index: size_t) -> *mut c_void;
    fn mxf_initialise_list_iter(iter: *mut MXFListIterator, list: *const MXFList);
    fn mxf_next_list_iter_element(iter: *mut MXFListIterator) -> c_int;
    fn mxf_get_iter_element(iter: *mut MXFListIterator) -> *mut c_void;

    /* mxf_data_model.h */
    fn mxf_load_data_model(datamodel: *mut *mut MXFDataModel) -> c_int;
    fn mxf_finalise_data_model(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_is_subclass_of(datamodel: *mut MXFDataModel, setkey: *mut MXFKey, mxfkey: *const MXFKey) -> c_int;

    /* mxf_avid.h */
    fn mxf_avid_get_mob_attribute(name: *const uint16, names: *const MXFList, values: *const MXFList, value: *mut *mut uint16) -> c_int;
    fn mxf_avid_read_string_mob_attributes(dataset: *mut MXFMetadataSet, names: *mut *mut MXFList, values: *mut *mut MXFList) -> c_int;
    fn mxf_avid_load_extensions(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_avid_read_string_tagged_values(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey, names: *mut *mut MXFList, values: *mut *mut MXFList) -> c_int;
    fn mxf_avid_read_string_tagged_value(dataset: *mut MXFMetadataSet, name: *mut *mut uint16, value: *mut *mut uint16) -> c_int;
    fn mxf_avid_read_filtered_header_metadata(mxffile: *mut MXFFile, skip_data_refs: c_int, headerdata: *mut MXFHeaderMetadata,
                                         header_byte_count: uint64, mxfkey: *const MXFKey, llen: uint8, len: uint64) -> c_int;
    fn mxf_avid_get_data_def(headerdata: *mut MXFHeaderMetadata, uuid: *mut MXFKey, datadef: *mut MXFKey) -> c_int;

    /* mxf_uu_metadata.h */
    fn mxf_uu_get_top_file_package(headerdata: *mut MXFHeaderMetadata, dataset: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_uu_get_package_tracks(set: *mut MXFMetadataSet, iter: *mut MXFArrayItemIterator) -> c_int;
    fn mxf_uu_next_track(headerdata: *mut MXFHeaderMetadata, iter: *mut MXFArrayItemIterator, set: *mut *mut MXFMetadataSet) -> c_int;
    fn mxf_uu_get_track_datadef(set: *mut MXFMetadataSet, key: *mut MXFKey) -> c_int;
    fn mxf_uu_get_track_duration(set: *mut MXFMetadataSet, duration: *mut MXFLength) -> c_int;
    fn mxf_uu_get_track_duration_at_rate(set: *mut MXFMetadataSet, edit_rate: *mut MXFRational, duration: *mut MXFLength) -> c_int;
    fn mxf_uu_get_referenced_package(headerdata: *mut MXFHeaderMetadata, source_pkg_id: *mut MXFUmid, set: *mut *mut MXFMetadataSet) -> c_int;

    /* mxf_label_and_keys.h */
    fn mxf_is_picture(key: *const MXFKey) -> c_int;
    fn mxf_is_sound(key: *const MXFKey) -> c_int;
    fn mxf_is_timecode(key: *const MXFKey) -> c_int;
}
