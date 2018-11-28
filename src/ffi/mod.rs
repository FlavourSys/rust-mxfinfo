#![allow(non_camel_case_types)]
mod consts;
mod mxf;

use chrono::NaiveDateTime;
use ffi::consts::*;
use ffi::mxf::*;
pub use ffi::mxf::{MXFKey, MXFUmid};
use num_rational::Rational32;
use std::path::Path;

fn convert_length(target_edit_rate: &Rational32, edit_rate: &Rational32, length: int64) -> int64 {
    ((length * target_edit_rate.numer().clone() as i64 * edit_rate.denom().clone() as i64) as f64
        / (target_edit_rate.denom() * edit_rate.numer()) as f64
        + 0.5) as int64
}

fn compare_length(
    edit_rate_a: &Rational32,
    length_a: int64,
    edit_rate_b: &Rational32,
    length_b: int64,
) -> int64 {
    length_a - convert_length(edit_rate_a, edit_rate_b, length_b)
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
    pub file_source_package_uid: Option<MXFUmid>,
    pub physical_source_package_uid: Option<MXFUmid>,
    pub material_package_uid: Option<MXFUmid>,

    /* UL */
    pub essence_container_label: Option<MXFKey>,
    pub picture_coding_label: Option<MXFKey>,

    /* Timstamps */
    pub clip_created: Option<NaiveDateTime>,

    /* Avid Essence Type */
    pub essence_type: AvidEssenceType,

    /* Avid Physical Pacakge Type */
    pub physical_package_type: AvidPhysicalPackageType,

    /* Integers */
    pub frame_layout: Option<u8>,
    pub stored_width: Option<u32>,
    pub stored_height: Option<u32>,
    pub display_width: Option<u32>,
    pub display_height: Option<u32>,
    pub track_duration: Option<i64>,
    pub segment_duration: Option<i64>,
    pub start_timecode: i64,
    pub track_number: Option<u32>,
    pub channel_count: Option<u32>,
    pub quantization_bits: Option<u32>,
    pub clip_duration: Option<i64>,
    pub is_video: bool,
    pub audio_track_count: u32,
    pub video_track_count: u32,
    pub avid_resolution_id: Option<i32>,
}

impl AvidMXFInfo {
    pub fn from_file(filename: &Path) -> Result<AvidMXFInfo, String> {
        let mut info = AvidMXFInfo::default();
        let mut max_duration = 0;
        let mut max_edit_rate = Rational32::new(25, 1);
        let mut package_uid = MXFUmid::default();
        let mob_name = vec![95, 80, 74, 0];

        /* Open file */
        let file = MXFFile::from_file(filename)?;

        /* Get preface set */
        let preface_set = file
            .headerdata()
            .find_singular_set_by_key(&G_PREFACE_SET_KEY)?;

        /* Get project name */
        if preface_set.has_item(&G_PREFACE_PROJECTNAME_ITEM_KEY) {
            info.project_name = preface_set.get_string(&G_PREFACE_PROJECTNAME_ITEM_KEY);
        }

        /* Get project edit rate if available */
        if preface_set.has_item(&G_PREFACE_PROJECTEDITRATE_ITEM_KEY) {
            info.project_edit_rate = preface_set.get_rational(&G_PREFACE_PROJECTEDITRATE_ITEM_KEY);
        }

        /* Get essence container label */
        info.essence_container_label = Some(MXFKey::from(
            file.headerpartition().essence_containers().at(0),
        ));

        /* Get material package set */
        let material_package_set = file
            .headerdata()
            .find_singular_set_by_key(&G_MATERIALPACKAGE_SET_KEY)?;
        info.material_package_uid =
            material_package_set.get_umid(&G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY);
        if material_package_set.has_item(&G_GENERICPACKAGE_NAME_ITEM_KEY) {
            info.clip_name = material_package_set.get_string(&G_GENERICPACKAGE_NAME_ITEM_KEY);
        }
        if material_package_set.has_item(&G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY) {
            info.clip_created =
                material_package_set.get_timestamp(&G_GENERICPACKAGE_PACKAGECREATIONDATE_ITEM_KEY);
        }

        /* Get the material package project name tagged value if not aleady set */
        if info.project_name.is_none()
            && material_package_set.has_item(&G_GENERICPACKAGE_MOBATTRIBUTELIST_ITEM_KEY)
        {
            let (names, values) = material_package_set
                .read_string_mob_attributes()
                .ok_or("Could not read mob attributes.".to_string())?;
            info.project_name = MXFFile::get_mob_attribute(&mob_name, &names, &values);
            /* NOTE: Lists need to be freed. */
        }

        /* Get the top level file source package and info */
        let file_source_package_set = file
            .headerdata()
            .get_top_file_package()
            .ok_or("Could not read top level file package.".to_string())?;
        info.file_source_package_uid =
            file_source_package_set.get_umid(&G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY);

        /* Get the file source package essence descriptor info */
        let descriptor_set = file_source_package_set
            .get_strongref(&G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY)
            .ok_or("Could not read descriptor set.".to_string())?;
        if file.datamodel().is_subclass_of(
            &descriptor_set.key,
            &G_GENERICPICTUREESSENCEDESCRIPTOR_SET_KEY,
        ) {
            /* Image aspect ratio */
            if descriptor_set.has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_ASPECTRATIO_ITEM_KEY) {
                info.aspect_ratio = descriptor_set
                    .get_rational(&G_GENERICPICTUREESSENCEDESCRIPTOR_ASPECTRATIO_ITEM_KEY);
            }
            /* Frame layout */
            if descriptor_set.has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_FRAMELAYOUT_ITEM_KEY) {
                info.frame_layout = descriptor_set
                    .get_uint8(&G_GENERICPICTUREESSENCEDESCRIPTOR_FRAMELAYOUT_ITEM_KEY);
            }
            /* Stored height */
            if descriptor_set.has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDHEIGHT_ITEM_KEY) {
                info.stored_height = descriptor_set
                    .get_uint32(&G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDHEIGHT_ITEM_KEY);
            }
            /* Stored width */
            if descriptor_set.has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDWIDTH_ITEM_KEY) {
                info.stored_width = descriptor_set
                    .get_uint32(&G_GENERICPICTUREESSENCEDESCRIPTOR_STOREDWIDTH_ITEM_KEY);
            }
            /* Avid Resolution ID */
            if descriptor_set.has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_RESOLUTIONID_ITEM_KEY) {
                info.avid_resolution_id = descriptor_set
                    .get_int32(&G_GENERICPICTUREESSENCEDESCRIPTOR_RESOLUTIONID_ITEM_KEY);
            }
            /* Picture essence coding label */
            if descriptor_set
                .has_item(&G_GENERICPICTUREESSENCEDESCRIPTOR_PICTUREESSENCECODING_ITEM_KEY)
            {
                info.picture_coding_label = descriptor_set
                    .get_ul(&G_GENERICPICTUREESSENCEDESCRIPTOR_PICTUREESSENCECODING_ITEM_KEY);
            }
        } else if file.datamodel().is_subclass_of(
            &descriptor_set.key,
            &G_GENERICSOUNDESSENCEDESCRIPTOR_SET_KEY,
        ) {
            /* Audio Sampling Rate */
            if descriptor_set.has_item(&G_GENERICSOUNDESSENCEDESCRIPTOR_AUDIOSAMPLINGRATE_ITEM_KEY)
            {
                info.audio_sampling_rate = descriptor_set
                    .get_rational(&G_GENERICSOUNDESSENCEDESCRIPTOR_AUDIOSAMPLINGRATE_ITEM_KEY);
            }
            /* Quantization bits */
            if descriptor_set.has_item(&G_GENERICSOUNDESSENCEDESCRIPTOR_QUANTIZATIONBITS_ITEM_KEY) {
                info.quantization_bits = descriptor_set
                    .get_uint32(&G_GENERICSOUNDESSENCEDESCRIPTOR_QUANTIZATIONBITS_ITEM_KEY);
            }
            /* Channel count */
            if descriptor_set.has_item(&G_GENERICSOUNDESSENCEDESCRIPTOR_CHANNELCOUNT_ITEM_KEY) {
                info.channel_count = descriptor_set
                    .get_uint32(&G_GENERICSOUNDESSENCEDESCRIPTOR_CHANNELCOUNT_ITEM_KEY);
            }
        }

        /* Get the material track referencing the file source package and info */
        let mut array_iter = material_package_set
            .get_package_tracks()
            .ok_or("Could not read material track reference.".to_string())?;
        array_iter.headerdata = Some(file.headerdata());
        for track_set in array_iter {
            let mut datadef = track_set
                .get_track_datadef()
                .ok_or("Could not read material track reference.".to_string())?;

            /* Some Avid files have a weak reference to a data definition instead of a UL */
            if !datadef.is_picture() && !datadef.is_sound() && !datadef.is_timecode() {
                match file.headerdata().get_data_def(&datadef) {
                    Some(ddef) => datadef = ddef,
                    None => continue,
                }
            }
            /* Skip non-video and non-audio tracks */
            if !datadef.is_picture() && !datadef.is_sound() {
                continue;
            }
            /* Track counts */
            if datadef.is_picture() {
                info.video_track_count += 1;
            }
            if datadef.is_sound() {
                info.audio_track_count += 1;
            }
            /* Track number */
            let track_number = if track_set.has_item(&G_GENERICTRACK_TRACKNUMBER_ITEM_KEY) {
                track_set
                    .get_uint32(&G_GENERICTRACK_TRACKNUMBER_ITEM_KEY)
                    .ok_or("Could not read track number.".to_string())?
            } else {
                0
            };
            /* Edit rate */
            let edit_rate = track_set.get_rational(&G_TRACK_EDITRATE_ITEM_KEY);
            if info.project_edit_rate.is_none() && datadef.is_picture() {
                info.project_edit_rate = edit_rate;
            }
            /* Track duration */
            let track_duration = track_set
                .get_track_duration()
                .ok_or("Could not read track duration.".to_string())?;
            if edit_rate.is_some()
                && compare_length(
                    &max_edit_rate,
                    max_duration,
                    &edit_rate.unwrap(),
                    track_duration,
                ) <= 0
            {
                max_edit_rate = edit_rate.unwrap();
                max_duration = track_duration;
            }
            /* Get info from this track if it refrences the file source package through a
             * source clip */
            let sequence_set = track_set
                .get_strongref(&G_GENERICTRACK_SEQUENCE_ITEM_KEY)
                .ok_or("Could not read generic track sequences.".to_string())?;
            if !sequence_set
                .headerdata()
                .datamodel()
                .is_subclass_of(&sequence_set.key, &G_SOURCECLIP_SET_KEY)
            {
                let count = sequence_set
                    .get_array_len(&G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY)
                    .ok_or("Could not read structural componenet array count.".to_string())?;

                for i in 0..count {
                    let elem = sequence_set
                        .get_array_element(&G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, i)
                        .ok_or("Could not read array element.".to_string())?;
                    let source_clip_set = match sequence_set.headerdata().get_strongref(&elem) {
                        Some(set) => set,
                        None => continue,
                    };
                    info.segment_duration =
                        source_clip_set.get_length(&G_STRUCTURALCOMPONENT_DURATION_ITEM_KEY);

                    if source_clip_set
                        .headerdata()
                        .datamodel()
                        .is_subclass_of(&source_clip_set.key, &G_ESSENCEGROUP_SET_KEY)
                    {
                        let choices_count = source_clip_set
                            .get_array_len(&G_ESSENCEGROUP_CHOICES_ITEM_KEY)
                            .ok_or(
                                "Could not read array essence group choices count.".to_string(),
                            )?;
                        let mut final_idx = 0;
                        for j in 0..choices_count {
                            final_idx = j;
                            let elem = source_clip_set
                                .get_array_element(&G_ESSENCEGROUP_CHOICES_ITEM_KEY, j)
                                .ok_or("Could not read essence group choices.".to_string())?;

                            let source_clip_set =
                                match source_clip_set.headerdata().get_strongref(elem) {
                                    Some(set) => set,
                                    /* Dark set not registered in dictionary. */
                                    None => continue,
                                };

                            if source_clip_set
                                .headerdata()
                                .datamodel()
                                .is_subclass_of(&source_clip_set.key, &G_SOURCECLIP_SET_KEY)
                            {
                                package_uid = source_clip_set
                                    .get_umid(&G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY)
                                    .ok_or("Could not read source package id.".to_string())?;
                                if &package_uid == info.file_source_package_uid.as_ref().unwrap() {
                                    /* Found source clip referencing file source package */
                                    break;
                                }
                            }
                        }
                        if final_idx < choices_count {
                            /* Found source clip referencing source package */
                            break;
                        }
                    } else if source_clip_set
                        .headerdata()
                        .datamodel()
                        .is_subclass_of(&source_clip_set.key, &G_SOURCECLIP_SET_KEY)
                    {
                        package_uid = source_clip_set
                            .get_umid(&G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY)
                            .ok_or("Could not read source package id.".to_string())?;
                        if &package_uid == info.file_source_package_uid.as_ref().unwrap() {
                            /* Found source clip referencing source package */
                            break;
                        }
                    }
                }
            } else {
                package_uid = sequence_set
                    .get_umid(&G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY)
                    .ok_or("Could not read source package id.".to_string())?;
                info.segment_duration =
                    sequence_set.get_length(&G_STRUCTURALCOMPONENT_DURATION_ITEM_KEY);
            }

            if &package_uid == info.file_source_package_uid.as_ref().unwrap() {
                info.is_video = datadef.is_picture();
                info.clip_edit_rate = edit_rate;
                info.track_duration = Some(track_duration);
                info.track_number = Some(track_number);
            }
        }

        info.clip_duration = Some(convert_length(
            info.project_edit_rate
                .as_ref()
                .ok_or("Project edit rate was empty.".to_string())?,
            &max_edit_rate,
            max_duration,
        ));

        /* Get the physical source package and info */
        let mut list = file
            .headerdata()
            .find_set_by_key(&G_SOURCEPACKAGE_SET_KEY)?;
        let list_iter = list.get_iter();
        for elem in list_iter {
            let set = <(&mut MXFMetadataSet)>::from(elem);
            /* The Physical source package is the source package that references a physical
             * descriptor */

            if !set.has_item(&G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY) {
                continue;
            }
            let descriptor_set = match set.get_strongref(&G_SOURCEPACKAGE_DESCRIPTOR_ITEM_KEY) {
                Some(x) => x,
                None => continue,
            };

            /* Get first physical network locator */
            if descriptor_set.has_item(&G_GENERICDESCRIPTOR_LOCATORS_ITEM_KEY) {
                let mut array_iter = descriptor_set
                    .initialize_array_iterator(&G_GENERICDESCRIPTOR_LOCATORS_ITEM_KEY)
                    .ok_or("Could not read array item iterator.".to_string())?;
                for item in array_iter {
                    let locator_set = file
                        .headerdata()
                        .get_strongref(item)
                        .ok_or("Could not read locator set.")?;
                    if file
                        .headerdata()
                        .datamodel()
                        .is_subclass_of(&locator_set.key, &G_NETWORKLOCATOR_SET_KEY)
                    {
                        info.physical_package_locator =
                            locator_set.get_string(&G_NETWORKLOCATOR_URLSTRING_ITEM_KEY);
                    }
                }
            }

            /* NOTE: Some descriptors could be dark and so we don't assume we can dereference
             * */
            if file
                .headerdata()
                .datamodel()
                .is_subclass_of(&descriptor_set.key, &G_PHYSICALDESCRIPTOR_SET_KEY)
            {
                if file
                    .headerdata()
                    .datamodel()
                    .is_subclass_of(&descriptor_set.key, &G_TAPEDESCRIPTOR_SET_KEY)
                {
                    info.physical_package_type = AvidPhysicalPackageType::TapePhysType;
                } else if file
                    .headerdata()
                    .datamodel()
                    .is_subclass_of(&descriptor_set.key, &G_IMPORTDESCRIPTOR_SET_KEY)
                {
                    info.physical_package_type = AvidPhysicalPackageType::ImportPhysType;
                } else if file
                    .headerdata()
                    .datamodel()
                    .is_subclass_of(&descriptor_set.key, &G_RECORDINGDESCRIPTOR_SET_KEY)
                {
                    info.physical_package_type = AvidPhysicalPackageType::RecordingPhysType;
                } else {
                    info.physical_package_type = AvidPhysicalPackageType::UnkownPhysType;
                }

                info.physical_source_package_uid =
                    set.get_umid(&G_GENERICPACKAGE_PACKAGEUID_ITEM_KEY);
                if set.has_item(&G_GENERICPACKAGE_NAME_ITEM_KEY) {
                    info.physical_package_name = set.get_string(&G_GENERICPACKAGE_NAME_ITEM_KEY);
                }

                break;
            }
        }
        MXFList::free(&mut list);

        /* Get the start timecode
         * the source timecode is calculated using the SourceClip::start_position in the file
         * source package in conjunction with the TimecodeComponent in the referenced physical
         * source package */
        let mut array_iter = file_source_package_set
            .get_package_tracks()
            .ok_or("Could not read package tracks.".to_string())?;
        let mut has_timecode = false;
        array_iter.headerdata = Some(file.headerdata());
        for track_set in array_iter {
            if has_timecode {
                break;
            }
            let mut datadef = track_set
                .get_track_datadef()
                .ok_or("Could not read datadef of trackset.".to_string())?;
            /* Skip non timecode tracks */
            if !datadef.is_picture() && !datadef.is_sound() && !datadef.is_timecode() {
                /* Some Avid files have a weak reference to a data definition instead of a UL */
                datadef = match file.headerdata().get_data_def(&mut datadef) {
                    Some(x) => x,
                    None => continue,
                };
            }

            if !datadef.is_picture() && !datadef.is_sound() {
                continue;
            }

            /* Get the timecode component */
            let source_clip_set = match track_set.get_single_track_component(&G_SOURCECLIP_SET_KEY)
            {
                Some(x) => x,
                None => continue,
            };
            let package_edit_rate = track_set.get_rational(&G_TRACK_EDITRATE_ITEM_KEY);
            let package_start_pos = source_clip_set
                .get_position(&G_SOURCECLIP_STARTPOSITION_ITEM_KEY)
                .ok_or("Could not read track start position.".to_string())?;

            /* Get the package referenced by the source clip */
            let mut source_package_id = source_clip_set
                .get_umid(&G_SOURCECLIP_SOURCEPACKAGEID_ITEM_KEY)
                .ok_or("Could not read source package id.".to_string())?;
            let ref_source_package_set = source_clip_set
                .headerdata()
                .get_referenced_package(&mut source_package_id);
            if source_package_id == MXFUmid::default() || ref_source_package_set.is_none() {
                /* Either at the end of the chain or doesn't have the referenced package */
                continue;
            }

            /* Find the timecode componenet in the physical source package and calculate the start
             * timecode */
            let ref_source_package_set = ref_source_package_set
                .ok_or("Could not read referenced source package set.".to_string())?;
            let mut array_iter2 = ref_source_package_set
                .get_package_tracks()
                .ok_or("Could not read pacakge track iterator".to_string())?;
            array_iter2.headerdata = Some(file.headerdata());
            for inner_track_set in array_iter2 {
                let mut datadef = inner_track_set
                    .get_track_datadef()
                    .ok_or("Could not read track datadef.".to_string())?;

                /* Some avid files have a weak reference to a data definition instead of a UL */
                if !datadef.is_picture() && !datadef.is_sound() && !datadef.is_timecode() {
                    datadef = match file.headerdata().get_data_def(&mut datadef) {
                        Some(x) => x,
                        None => continue,
                    };
                }
                if !datadef.is_timecode() {
                    continue;
                }

                /* Get the Timecode component */
                let timecode_component_set = match inner_track_set
                    .get_single_track_component(&G_TIMECODECOMPONENT_SET_KEY)
                {
                    Some(x) => x,
                    None => continue,
                };

                /* Get the start timecode and rounded timecode base for the timecode component */
                let mut start_timecode = timecode_component_set
                    .get_position(&G_TIMECODECOMPONENT_STARTTIMECODE_ITEM_KEY)
                    .ok_or("Could not read timecode track position item.".to_string())?;
                let mut timecode_base = timecode_component_set
                    .get_uint16(&G_TIMECODECOMPONENT_ROUNDEDTIMECODEBASE_ITEM_KEY)
                    .ok_or("Could not read timecode base item.".to_string())?;
                if info.clip_edit_rate.is_none() || package_edit_rate.is_none() {
                    return Err("Clip edit rate is empty for timecode calculation.".to_string());
                }
                let c_edit_rate = info.clip_edit_rate.as_ref().unwrap();
                let p_edit_rate = package_edit_rate.as_ref().unwrap();

                let mut start_pos = package_start_pos;
                if start_timecode > 0 {
                    let tmp_timecode_base = (p_edit_rate.numer().clone() as f64
                        / p_edit_rate.denom().clone() as f64
                        + 0.5) as u16;
                    if tmp_timecode_base == timecode_base {
                        start_pos += start_timecode;
                    } else if tmp_timecode_base == 2 * timecode_base {
                        start_pos += 2 * start_timecode;
                    } else {
                        /* TODO: Complete support for different timecode and edit rates */
                    }
                }

                info.start_timecode = ((start_pos
                    * c_edit_rate.numer().clone() as i64
                    * p_edit_rate.denom().clone() as i64)
                    as f64
                    / (c_edit_rate.denom().clone() * p_edit_rate.numer().clone()) as f64
                    + 0.5) as i64;

                has_timecode = true;
                break;
            }
        }

        return Ok(info);
    }

    pub fn is_renderfile(&self) -> bool {
        self.physical_package_name == Some("Precompute Source Mob".to_string())
    }
}
