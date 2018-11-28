use chrono::{NaiveDate, NaiveDateTime};
use num_rational::Rational32;
use num_traits::Zero;
use std::ffi::CString;
use std::fmt;
use std::path::Path;
use std::ptr;

use ffi::consts::*;

/* C Types */
pub type uint8 = libc::uint8_t;
pub type uint16 = libc::uint16_t;
pub type uint32 = libc::uint32_t;
pub type uint64 = libc::uint64_t;
pub type int64 = libc::int64_t;
pub type int16 = libc::int16_t;
pub type int32 = libc::int32_t;
pub type size_t = libc::size_t;
pub type c_char = libc::c_char;
pub type c_int = libc::c_int;
pub type c_void = libc::c_void;

fn convert_string(utf16str: *mut uint16) -> Option<String> {
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

enum _MXFFile {}
pub enum MXFDataModel {}

impl MXFDataModel {
    pub fn is_subclass_of(&mut self, key_a: &MXFKey, key_b: &MXFKey) -> bool {
        unsafe { !mxf_is_subclass_of(self, key_a, key_b).is_zero() }
    }
}

pub enum MXFPrimerPack {}

/* Stub types */
pub struct MXFFile {
    mxffile: *mut _MXFFile,
    pub headerpartition: *mut MXFPartition,
    pub datamodel: *mut MXFDataModel,
    pub headerdata: *mut MXFHeaderMetadata,
}

impl MXFFile {
    pub fn from_file(filename: &Path) -> Result<MXFFile, String> {
        let filename = filename
            .to_str()
            .ok_or("Filename not UTF-8 compliant.".to_string())?;
        let filename =
            CString::new(filename).map_err(|_| "Filename not CString compliant.".to_string())?;
        let mut mxffile = ptr::null_mut();
        let mut headerpartition = ptr::null_mut();
        let mut headerdata = ptr::null_mut();
        let mut datamodel = ptr::null_mut();
        let mut mxful = MXFKey::default();
        let mut llen = 0;
        let mut len = 0;

        unsafe {
            if mxf_disk_file_open_read(filename.as_ptr(), &mut mxffile).is_zero() {
                return Err("Could not read file.".to_string());
            }

            if mxf_read_header_pp_kl(mxffile, &mut mxful, &mut llen, &mut len).is_zero() {
                mxf_file_close(&mut mxffile);
                return Err("Could not read header.".to_string());
            }

            if mxf_read_partition(mxffile, &mxful, &mut headerpartition).is_zero() {
                mxf_file_close(&mut mxffile);
                return Err("Could not read header partition.".to_string());
            }

            if !headerpartition
                .as_ref()
                .unwrap()
                .operational_pattern
                .is_op_atom()
            {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                return Err("Is not OP-Atom.".to_string());
            }

            if mxf_load_data_model(&mut datamodel).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                return Err("Could not load datamodel.".to_string());
            }

            if mxf_avid_load_extensions(datamodel).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                return Err("Could not load avid extensions.".to_string());
            }

            if mxf_finalise_data_model(datamodel).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                return Err("Could not finalize datamodel.".to_string());
            }

            if mxf_read_next_nonfiller_kl(mxffile, &mut mxful, &mut llen, &mut len).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                return Err("Could not read next nonfiller kl.".to_string());
            }

            if mxf_is_header_metadata(&mut mxful).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                return Err("Is not header metadata.".to_string());
            }

            if mxf_create_header_metadata(&mut headerdata, datamodel).is_zero() {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                return Err("Could not read header metadata.".to_string());
            }

            if mxf_avid_read_filtered_header_metadata(
                mxffile,
                0,
                headerdata,
                headerpartition.as_ref().unwrap().header_byte_count,
                &mxful,
                llen,
                len,
            ).is_zero()
            {
                mxf_file_close(&mut mxffile);
                mxf_free_partition(&mut headerpartition);
                mxf_free_data_model(&mut datamodel);
                mxf_free_header_metadata(&mut headerdata);
                return Err("Could not read header metadata.".to_string());
            }
        }

        Ok(MXFFile {
            mxffile: mxffile,
            headerpartition: headerpartition,
            datamodel: datamodel,
            headerdata: headerdata,
        })
    }

    pub fn get_mob_attribute(
        name: &Vec<uint16>,
        names: &MXFList,
        values: &MXFList,
    ) -> Option<String> {
        unsafe {
            let mut value = ptr::null_mut();
            if mxf_avid_get_mob_attribute(name.as_ptr(), names, values, &mut value).is_zero() {
                None
            } else {
                convert_string(value)
            }
        }
    }

    pub fn headerpartition(&self) -> &mut MXFPartition {
        unsafe { self.headerpartition.as_mut().unwrap() }
    }

    pub fn datamodel(&self) -> &mut MXFDataModel {
        unsafe { self.datamodel.as_mut().unwrap() }
    }

    pub fn headerdata(&self) -> &mut MXFHeaderMetadata {
        unsafe { self.headerdata.as_mut().unwrap() }
    }
}

impl Drop for MXFFile {
    fn drop(&mut self) {
        unsafe {
            mxf_file_close(&mut self.mxffile);
            mxf_free_partition(&mut self.headerpartition);
            mxf_free_data_model(&mut self.datamodel);
            mxf_free_header_metadata(&mut self.headerdata);
        }
    }
}

pub enum AvidPhysicalPackageType {
    UnkownPhysType,
    TapePhysType,
    ImportPhysType,
    RecordingPhysType,
}

impl Default for AvidPhysicalPackageType {
    fn default() -> AvidPhysicalPackageType {
        AvidPhysicalPackageType::UnkownPhysType
    }
}

impl fmt::Display for AvidPhysicalPackageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AvidPhysicalPackageType::UnkownPhysType => write!(f, "Unknown"),
            AvidPhysicalPackageType::TapePhysType => write!(f, "Tape"),
            AvidPhysicalPackageType::ImportPhysType => write!(f, "Import"),
            AvidPhysicalPackageType::RecordingPhysType => write!(f, "Recording"),
        }
    }
}

impl fmt::Debug for AvidPhysicalPackageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub enum AvidEssenceType {
    Unknown,
    Mpeg30,
    Mpeg40,
    Mpeg50,
    DV25_411,
    DV25_420,
    DV50,
    DV100,
    MJpeg_20_1,
    MJpeg_2_1S,
    MJpeg_4_1S,
    MJpeg_15_1S,
    MJpeg_10_1,
    MJpeg_10_1M,
    MJpeg_4_1M,
    MJpeg_3_1,
    MJpeg_2_1,
    Unc_1_1,
    Unc_1_1_10B,
    MJpeg_35_1P,
    MJpeg_28_1P,
    MJpeg_14_1P,
    MJpeg_3_1P,
    MJpeg_2_1P,
    MJpeg_3_1M,
    MJpeg_8_1M,
    Dnxhd_1235,
    Dnxhd_1237,
    Dnxhd_1238,
    Dnxhd_1241,
    Dnxhd_1242,
    Dnxhd_1243,
    Dnxhd_1250,
    Dnxhd_1251,
    Dnxhd_1252,
    Dnxhd_1253,
    Mpeg4,
    XDCamHD,
    AVCIntra_100,
    AVCIntra_50,
    PCM,
}

impl Default for AvidEssenceType {
    fn default() -> AvidEssenceType {
        AvidEssenceType::Unknown
    }
}

impl fmt::Display for AvidEssenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AvidEssenceType::Unknown => write!(f, "not recognized"),
            AvidEssenceType::Mpeg30 => write!(f, "MPEG 30"),
            AvidEssenceType::Mpeg40 => write!(f, "MPEG 40"),
            AvidEssenceType::Mpeg50 => write!(f, "MPEG 50"),
            AvidEssenceType::DV25_411 => write!(f, "DV 25 411"),
            AvidEssenceType::DV25_420 => write!(f, "DV 25 420"),
            AvidEssenceType::DV50 => write!(f, "DV 50"),
            AvidEssenceType::DV100 => write!(f, "DV 100"),
            AvidEssenceType::MJpeg_20_1 => write!(f, "20:1"),
            AvidEssenceType::MJpeg_2_1S => write!(f, "2:1s"),
            AvidEssenceType::MJpeg_4_1S => write!(f, "4:1s"),
            AvidEssenceType::MJpeg_15_1S => write!(f, "15:1s"),
            AvidEssenceType::MJpeg_10_1 => write!(f, "10:1"),
            AvidEssenceType::MJpeg_10_1M => write!(f, "10:1m"),
            AvidEssenceType::MJpeg_4_1M => write!(f, "4:1m"),
            AvidEssenceType::MJpeg_3_1 => write!(f, "3:1"),
            AvidEssenceType::MJpeg_2_1 => write!(f, "2:1"),
            AvidEssenceType::Unc_1_1 => write!(f, "1:1"),
            AvidEssenceType::Unc_1_1_10B => write!(f, "1:1 10b"),
            AvidEssenceType::MJpeg_35_1P => write!(f, "35:1p"),
            AvidEssenceType::MJpeg_28_1P => write!(f, "28:1p"),
            AvidEssenceType::MJpeg_14_1P => write!(f, "14:1p"),
            AvidEssenceType::MJpeg_3_1P => write!(f, "3:1p"),
            AvidEssenceType::MJpeg_2_1P => write!(f, "2:1p"),
            AvidEssenceType::MJpeg_3_1M => write!(f, "3:1m"),
            AvidEssenceType::MJpeg_8_1M => write!(f, "8:1m"),
            AvidEssenceType::Dnxhd_1235 => write!(f, "DNxHD 1235"),
            AvidEssenceType::Dnxhd_1237 => write!(f, "DNxHD 1237"),
            AvidEssenceType::Dnxhd_1238 => write!(f, "DNxHD 1238"),
            AvidEssenceType::Dnxhd_1241 => write!(f, "DNxHD 1241"),
            AvidEssenceType::Dnxhd_1242 => write!(f, "DNxHD 1242"),
            AvidEssenceType::Dnxhd_1243 => write!(f, "DNxHD 1243"),
            AvidEssenceType::Dnxhd_1250 => write!(f, "DNxHD 1250"),
            AvidEssenceType::Dnxhd_1251 => write!(f, "DNxHD 1251"),
            AvidEssenceType::Dnxhd_1252 => write!(f, "DNxHD 1252"),
            AvidEssenceType::Dnxhd_1253 => write!(f, "DNxHD 1253"),
            AvidEssenceType::Mpeg4 => write!(f, "MPEG-4"),
            AvidEssenceType::XDCamHD => write!(f, "XDCAM HD"),
            AvidEssenceType::AVCIntra_100 => write!(f, "AVC-Intra 100"),
            AvidEssenceType::AVCIntra_50 => write!(f, "AVC-Intra 50"),
            AvidEssenceType::PCM => write!(f, "PCM"),
        }
    }
}

impl fmt::Debug for AvidEssenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[repr(C)]
#[derive(Default)]
pub struct MXFUmid {
    pub octet0: libc::uint8_t,
    pub octet1: libc::uint8_t,
    pub octet2: libc::uint8_t,
    pub octet3: libc::uint8_t,
    pub octet4: libc::uint8_t,
    pub octet5: libc::uint8_t,
    pub octet6: libc::uint8_t,
    pub octet7: libc::uint8_t,
    pub octet8: libc::uint8_t,
    pub octet9: libc::uint8_t,
    pub octet10: libc::uint8_t,
    pub octet11: libc::uint8_t,
    pub octet12: libc::uint8_t,
    pub octet13: libc::uint8_t,
    pub octet14: libc::uint8_t,
    pub octet15: libc::uint8_t,
    pub octet16: libc::uint8_t,
    pub octet17: libc::uint8_t,
    pub octet18: libc::uint8_t,
    pub octet19: libc::uint8_t,
    pub octet20: libc::uint8_t,
    pub octet21: libc::uint8_t,
    pub octet22: libc::uint8_t,
    pub octet23: libc::uint8_t,
    pub octet24: libc::uint8_t,
    pub octet25: libc::uint8_t,
    pub octet26: libc::uint8_t,
    pub octet27: libc::uint8_t,
    pub octet28: libc::uint8_t,
    pub octet29: libc::uint8_t,
    pub octet30: libc::uint8_t,
    pub octet31: libc::uint8_t,
}

impl MXFUmid {
    pub fn new(
        o0: u8,
        o1: u8,
        o2: u8,
        o3: u8,
        o4: u8,
        o5: u8,
        o6: u8,
        o7: u8,
        o8: u8,
        o9: u8,
        o10: u8,
        o11: u8,
        o12: u8,
        o13: u8,
        o14: u8,
        o15: u8,
        o16: u8,
        o17: u8,
        o18: u8,
        o19: u8,
        o20: u8,
        o21: u8,
        o22: u8,
        o23: u8,
        o24: u8,
        o25: u8,
        o26: u8,
        o27: u8,
        o28: u8,
        o29: u8,
        o30: u8,
        o31: u8,
    ) -> MXFUmid {
        MXFUmid {
            octet0: o0,
            octet1: o1,
            octet2: o2,
            octet3: o3,
            octet4: o4,
            octet5: o5,
            octet6: o6,
            octet7: o7,
            octet8: o8,
            octet9: o9,
            octet10: o10,
            octet11: o11,
            octet12: o12,
            octet13: o13,
            octet14: o14,
            octet15: o15,
            octet16: o16,
            octet17: o17,
            octet18: o18,
            octet19: o19,
            octet20: o20,
            octet21: o21,
            octet22: o22,
            octet23: o23,
            octet24: o24,
            octet25: o25,
            octet26: o26,
            octet27: o27,
            octet28: o28,
            octet29: o29,
            octet30: o30,
            octet31: o31,
        }
    }
}

impl std::cmp::PartialEq for MXFUmid {
    fn eq(&self, other: &MXFUmid) -> bool {
        unsafe { !mxf_equals_umid(self, other).is_zero() }
    }
}

impl std::fmt::Debug for MXFUmid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}",
               self.octet0, self.octet1, self.octet2, self.octet3, self.octet4, self.octet5, self.octet6, self.octet7,
               self.octet8, self.octet9, self.octet10, self.octet11, self.octet12, self.octet13, self.octet14, self.octet15,
               self.octet16, self.octet17, self.octet18, self.octet19, self.octet20, self.octet21, self.octet22, self.octet23,
               self.octet24, self.octet25, self.octet26, self.octet27, self.octet28, self.octet29, self.octet30, self.octet31)
    }
}

/* MXF List */
pub type free_func_type = Option<unsafe extern "C" fn(_: *mut c_void) -> ()>;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct MXFList {
    pub elements: *mut MXFListElement,
    pub last_element: *mut MXFListElement,
    pub len: size_t,
    pub free_func: free_func_type,
}

impl Default for MXFList {
    fn default() -> MXFList {
        MXFList {
            elements: ptr::null_mut(),
            last_element: ptr::null_mut(),
            len: 0,
            free_func: None,
        }
    }
}

impl MXFList {
    pub fn at(&mut self, index: size_t) -> *mut c_void {
        unsafe { mxf_get_list_element(self, index) }
    }

    pub fn len(&mut self) -> size_t {
        unsafe { mxf_get_list_length(self) }
    }

    pub fn get_iter(&self) -> MXFListIterator {
        unsafe {
            let mut iter = MXFListIterator::default();
            mxf_initialise_list_iter(&mut iter, self);
            iter
        }
    }

    pub fn free(list: &mut MXFList) {
        unsafe {
            mxf_free_list(&mut (list as *mut MXFList));
        }
    }
}

/* MXF List Element */
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct MXFListElement {
    pub next: *mut MXFListElement,
    pub data: *mut c_void,
}

impl Default for MXFListElement {
    fn default() -> MXFListElement {
        MXFListElement {
            next: ptr::null_mut(),
            data: ptr::null_mut(),
        }
    }
}

/* MXF List Iterator */
#[derive(Debug)]
#[repr(C)]
pub struct MXFListIterator {
    pub next: *mut MXFListElement,
    pub data: *mut c_void,
    pub index: size_t,
}

impl Iterator for MXFListIterator {
    type Item = *mut c_void;

    fn next(&mut self) -> Option<*mut c_void> {
        unsafe {
            if mxf_next_list_iter_element(self).is_zero() {
                None
            } else {
                Some(mxf_get_iter_element(self))
            }
        }
    }
}

impl Default for MXFListIterator {
    fn default() -> MXFListIterator {
        MXFListIterator {
            next: ptr::null_mut(),
            data: ptr::null_mut(),
            index: 0,
        }
    }
}

/* MXF Partition */
#[repr(C)]
#[derive(Debug, Default)]
pub struct MXFPartition {
    pub key: MXFKey,
    pub major_version: uint16,
    pub minor_version: uint16,
    pub kag_size: uint32,
    pub this_partition: uint64,
    pub previous_partition: uint64,
    pub footer_partition: uint64,
    pub header_byte_count: uint64,
    pub index_byte_count: uint64,
    pub indes_sid: uint32,
    pub body_offset: uint64,
    pub body_sid: uint32,
    pub operational_pattern: MXFKey,
    pub essence_containers: MXFList,
    pub header_mark_in_pos: int64,
    pub index_mark_in_pos: int64,
}

impl MXFPartition {
    pub fn essence_containers(&mut self) -> &mut MXFList {
        &mut self.essence_containers
    }
}

/* MXF Header Metadata */
#[repr(C)]
#[derive(Clone)]
pub struct MXFHeaderMetadata {
    pub datamodel: *mut MXFDataModel,
    pub primerpack: *mut MXFPrimerPack,
    pub sets: MXFList,
}

impl MXFHeaderMetadata {
    pub fn datamodel(&mut self) -> &mut MXFDataModel {
        unsafe { self.datamodel.as_mut().unwrap() }
    }

    pub fn find_singular_set_by_key(
        &mut self,
        mxfkey: &MXFKey,
    ) -> Result<&mut MXFMetadataSet, String> {
        unsafe {
            let mut dataset = ptr::null_mut();
            if mxf_find_singular_set_by_key(self, mxfkey, &mut dataset).is_zero() {
                return Err(format!("Could not find singular set by key: {:?}.", mxfkey));
            }

            Ok(dataset.as_mut().unwrap())
        }
    }

    pub fn find_set_by_key(&mut self, mxfkey: &MXFKey) -> Result<&mut MXFList, String> {
        unsafe {
            let mut list = ptr::null_mut();
            if mxf_find_set_by_key(self, mxfkey, &mut list).is_zero() {
                return Err(format!("Could not find set by key: {:?}.", mxfkey));
            }

            Ok(list.as_mut().unwrap())
        }
    }

    pub fn get_strongref(&mut self, value: &uint8) -> Option<&'static mut MXFMetadataSet> {
        unsafe {
            let mut set = ptr::null_mut();
            if mxf_get_strongref(self, value, &mut set).is_zero() {
                None
            } else {
                Some(set.as_mut().unwrap())
            }
        }
    }

    pub fn get_top_file_package(&mut self) -> Option<&mut MXFMetadataSet> {
        unsafe {
            let mut set = ptr::null_mut();
            if mxf_uu_get_top_file_package(self, &mut set).is_zero() {
                None
            } else {
                Some(set.as_mut().unwrap())
            }
        }
    }

    pub fn get_data_def(&mut self, uuid: &MXFKey) -> Option<MXFKey> {
        unsafe {
            let mut ddef = MXFKey::default();
            if mxf_avid_get_data_def(self, uuid, &mut ddef).is_zero() {
                None
            } else {
                Some(ddef)
            }
        }
    }

    pub fn get_referenced_package(
        &mut self,
        source_pkg_id: &MXFUmid,
    ) -> Option<&'static mut MXFMetadataSet> {
        unsafe {
            let mut set = ptr::null_mut();
            if mxf_uu_get_referenced_package(self, source_pkg_id, &mut set).is_zero() {
                None
            } else {
                Some(set.as_mut().unwrap())
            }
        }
    }
}

impl std::fmt::Debug for MXFHeaderMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            " MXFHeaderMetadata (\n\
             datamodel: {:?}\n\
             primerpack: {:?}\n\
             sets: \n{:?}\n\
             )",
            self.datamodel, self.primerpack, self.sets
        )
    }
}

impl Default for MXFHeaderMetadata {
    fn default() -> MXFHeaderMetadata {
        MXFHeaderMetadata {
            datamodel: ptr::null_mut(),
            primerpack: ptr::null_mut(),
            sets: MXFList::default(),
        }
    }
}

/* MXF Metadata Set */
#[repr(C)]
pub struct MXFMetadataSet {
    pub key: MXFKey,
    pub instance_uid: MXFKey,
    pub items: MXFList,
    pub header_metadata: *mut MXFHeaderMetadata,
    pub fixed_space_allocation: uint64,
}

impl<'a> From<*mut c_void> for &'a mut MXFMetadataSet {
    fn from(raw: *mut c_void) -> Self {
        unsafe { &mut *(raw as *mut MXFMetadataSet) }
    }
}

impl MXFMetadataSet {
    pub fn headerdata(&self) -> &mut MXFHeaderMetadata {
        unsafe { self.header_metadata.as_mut().unwrap() }
    }

    pub fn has_item(&mut self, mxfkey: &MXFKey) -> bool {
        unsafe { !mxf_have_item(self, mxfkey).is_zero() }
    }

    pub fn get_string(&mut self, mxfkey: &MXFKey) -> Option<String> {
        unsafe {
            let mut utf16size: uint16 = 0;
            if mxf_get_utf16string_item_size(self, mxfkey, &mut utf16size) == 0 {
                return None;
            }
            let mut utf16str: Vec<uint16> = vec![0; utf16size as usize];
            let u_ptr = utf16str.as_mut_ptr();
            if mxf_get_utf16string_item(self, mxfkey, u_ptr) == 0 {
                return None;
            }

            convert_string(u_ptr)
        }
    }

    pub fn get_rational(&mut self, mxfkey: &MXFKey) -> Option<Rational32> {
        unsafe {
            let mut mxf_rational = MXFRational::default();
            if mxf_get_rational_item(self, mxfkey, &mut mxf_rational).is_zero()
                || mxf_rational.denominator.is_zero()
            {
                None
            } else {
                Some(Rational32::new(
                    mxf_rational.numerator,
                    mxf_rational.denominator,
                ))
            }
        }
    }

    pub fn get_uint8(&mut self, mxfkey: &MXFKey) -> Option<uint8> {
        unsafe {
            let mut val = 0;
            if mxf_get_uint8_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_uint16(&mut self, mxfkey: &MXFKey) -> Option<uint16> {
        unsafe {
            let mut val = 0;
            if mxf_get_uint16_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_uint32(&mut self, mxfkey: &MXFKey) -> Option<uint32> {
        unsafe {
            let mut val = 0;
            if mxf_get_uint32_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_int32(&mut self, mxfkey: &MXFKey) -> Option<int32> {
        unsafe {
            let mut val = 0;
            if mxf_get_int32_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_umid(&mut self, mxfkey: &MXFKey) -> Option<MXFUmid> {
        unsafe {
            let mut val = MXFUmid::default();
            if mxf_get_umid_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_timestamp(&mut self, mxfkey: &MXFKey) -> Option<NaiveDateTime> {
        unsafe {
            let mut val = MXFTimestamp::default();
            if mxf_get_timestamp_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(
                    NaiveDate::from_ymd(val.year as i32, val.month as u32, val.day as u32).and_hms(
                        val.hour as u32,
                        val.min as u32,
                        val.sec as u32,
                    ),
                )
            }
        }
    }

    pub fn get_array_len(&mut self, mxfkey: &MXFKey) -> Option<uint32> {
        unsafe {
            let mut val = 0;
            if mxf_get_array_item_count(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_array_element(
        &mut self,
        mxfkey: &MXFKey,
        index: uint32,
    ) -> Option<&'static mut uint8> {
        unsafe {
            let mut val = ptr::null_mut();
            if mxf_get_array_item_element(self, mxfkey, index, &mut val).is_zero() {
                None
            } else {
                Some(val.as_mut().unwrap())
            }
        }
    }

    pub fn get_ul(&mut self, mxfkey: &MXFKey) -> Option<MXFKey> {
        unsafe {
            let mut val = MXFKey::default();
            if mxf_get_ul_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_length(&mut self, mxfkey: &MXFKey) -> Option<int64> {
        unsafe {
            let mut val = 0;
            if mxf_get_length_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_position(&mut self, mxfkey: &MXFKey) -> Option<int64> {
        unsafe {
            let mut val = 0;
            if mxf_get_position_item(self, mxfkey, &mut val).is_zero() {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_strongref(&mut self, mxfkey: &MXFKey) -> Option<&'static mut MXFMetadataSet> {
        unsafe {
            let mut set = ptr::null_mut();
            if mxf_get_strongref_item(self, mxfkey, &mut set).is_zero() {
                None
            } else {
                Some(set.as_mut().unwrap())
            }
        }
    }

    pub fn is_subclass_of(&mut self, mxfkey: &MXFKey) -> bool {
        unsafe { !mxf_set_is_subclass_of(self, mxfkey).is_zero() }
    }

    pub fn read_string_mob_attributes(&mut self) -> Option<(&mut MXFList, &mut MXFList)> {
        unsafe {
            let mut names = ptr::null_mut();
            let mut values = ptr::null_mut();
            if mxf_avid_read_string_mob_attributes(self, &mut names, &mut values).is_zero() {
                None
            } else {
                Some((names.as_mut().unwrap(), values.as_mut().unwrap()))
            }
        }
    }

    pub fn read_string_tagged_values(
        &mut self,
        mxfkey: &MXFKey,
    ) -> Option<(&mut MXFList, &mut MXFList)> {
        unsafe {
            let mut names = ptr::null_mut();
            let mut values = ptr::null_mut();
            if mxf_avid_read_string_tagged_values(self, mxfkey, &mut names, &mut values).is_zero() {
                None
            } else {
                Some((names.as_mut().unwrap(), values.as_mut().unwrap()))
            }
        }
    }

    pub fn read_string_tagged_value(&mut self) -> Result<(String, String), String> {
        unsafe {
            let mut name = ptr::null_mut();
            let mut value = ptr::null_mut();
            if mxf_avid_read_string_tagged_value(self, &mut name, &mut value).is_zero() {
                return Err("Tagged value: failed reading name and value.".to_string());
            } else {
                let name = convert_string(name).ok_or({
                    libc::free(name as *mut c_void);
                    libc::free(value as *mut c_void);
                    "Tagged value: name is not utf16.".to_string()
                })?;
                let value = convert_string(value).ok_or({
                    libc::free(value as *mut c_void);
                    "Tagged value: value is not utf16.".to_string()
                })?;

                Ok((name, value))
            }
        }
    }

    pub fn get_package_tracks(&mut self) -> Option<MXFTrackIterator> {
        unsafe {
            let mut iter = MXFArrayItemIterator::default();
            if mxf_uu_get_package_tracks(self, &mut iter).is_zero() {
                None
            } else {
                Some(MXFTrackIterator {
                    array_iter: iter,
                    headerdata: None,
                })
            }
        }
    }

    pub fn get_track_datadef(&mut self) -> Option<MXFKey> {
        unsafe {
            let mut mxfkey = MXFKey::default();
            if mxf_uu_get_track_datadef(self, &mut mxfkey).is_zero() {
                None
            } else {
                Some(mxfkey)
            }
        }
    }

    pub fn get_track_duration(&mut self) -> Option<i64> {
        unsafe {
            let mut duration = 0;
            if mxf_uu_get_track_duration(self, &mut duration).is_zero() {
                None
            } else {
                Some(duration)
            }
        }
    }

    pub fn initialize_array_iterator(&mut self, mxfkey: &MXFKey) -> Option<MXFArrayItemIterator> {
        unsafe {
            let mut iter = MXFArrayItemIterator::default();
            if mxf_initialise_array_item_iterator(self, mxfkey, &mut iter).is_zero() {
                None
            } else {
                Some(iter)
            }
        }
    }

    pub fn get_single_track_component(
        &mut self,
        mxfkey: &MXFKey,
    ) -> Option<&'static mut MXFMetadataSet> {
        let sequence_set = match self.get_strongref(&G_GENERICTRACK_SEQUENCE_ITEM_KEY) {
            Some(x) => x,
            None => return None,
        };

        let component_set = if sequence_set.is_subclass_of(&G_SEQUENCE_SET_KEY) {
            match sequence_set.get_array_len(&G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY) {
                Some(x) => {
                    if x == 1 {
                        x
                    } else {
                        return None;
                    }
                }
                None => return None,
            };

            let value = match sequence_set
                .get_array_element(&G_SEQUENCE_STRUCTURALCOMPONENTS_ITEM_KEY, 0)
            {
                Some(x) => x,
                None => return None,
            };

            match self.headerdata().get_strongref(value) {
                Some(x) => x,
                None => return None,
            }
        } else {
            sequence_set
        };

        if !component_set.is_subclass_of(mxfkey) {
            return None;
        }

        Some(component_set)
    }
}

impl std::fmt::Debug for MXFMetadataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                " MXFMetadataSet (\n\
                 key: {:?}\n\
                 instance_uid: {:?}\n\
                 items: \n{:?}\n\
                 header_metadata: \n{:?}\n\
                 fixed_space_allocation: {:?}\n\
                 )",
                self.key,
                self.instance_uid,
                self.items,
                *self.header_metadata,
                self.fixed_space_allocation
            )
        }
    }
}

/* MXF Metadata Item */
#[derive(Debug)]
#[repr(C)]
pub struct MXFMetadataItem {
    pub key: MXFKey,
    pub tag: uint16,
    pub is_persistent: c_int,
    pub length: uint16,
    pub value: *mut uint8,
    pub set: *mut MXFMetadataSet,
}

/* MXF Array Item Iterator */
#[derive(Debug)]
#[repr(C)]
pub struct MXFArrayItemIterator {
    pub item: *mut MXFMetadataItem,
    pub element_count: uint32,
    pub curr_length: uint32,
    pub curr_index: uint32,
}

impl Default for MXFArrayItemIterator {
    fn default() -> MXFArrayItemIterator {
        MXFArrayItemIterator {
            item: ptr::null_mut(),
            element_count: 0,
            curr_length: 0,
            curr_index: 0,
        }
    }
}

impl Iterator for MXFArrayItemIterator {
    type Item = &'static mut uint8;

    fn next(&mut self) -> Option<&'static mut uint8> {
        unsafe {
            let mut element = ptr::null_mut();
            let mut len = 0;
            if mxf_next_array_item_element(self, &mut element, &mut len).is_zero() {
                None
            } else {
                Some(element.as_mut().unwrap())
            }
        }
    }
}

pub struct MXFTrackIterator {
    pub array_iter: MXFArrayItemIterator,
    pub headerdata: Option<*mut MXFHeaderMetadata>,
}

impl Iterator for MXFTrackIterator {
    type Item = &'static mut MXFMetadataSet;

    fn next(&mut self) -> Option<&'static mut MXFMetadataSet> {
        if self.headerdata.is_none() {
            return None;
        }

        unsafe {
            let mut val = ptr::null_mut();
            if mxf_uu_next_track(self.headerdata.unwrap(), &mut self.array_iter, &mut val).is_zero()
            {
                None
            } else {
                Some(val.as_mut().unwrap())
            }
        }
    }
}

/* MXF Rational */
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct MXFRational {
    pub numerator: int32,
    pub denominator: int32,
}

/* MXF Timestamp */
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct MXFTimestamp {
    pub year: int16,
    pub month: uint8,
    pub day: uint8,
    pub hour: uint8,
    pub min: uint8,
    pub sec: uint8,
    pub qmsec: uint8,
}

impl std::fmt::Debug for MXFTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{} {}:{}:{}.{}",
            self.day, self.month, self.year, self.hour, self.min, self.sec, self.qmsec
        )
    }
}

/* MXF Key */
#[repr(C)]
#[derive(Default, Clone)]
pub struct MXFKey {
    pub octet0: libc::uint8_t,
    pub octet1: libc::uint8_t,
    pub octet2: libc::uint8_t,
    pub octet3: libc::uint8_t,
    pub octet4: libc::uint8_t,
    pub octet5: libc::uint8_t,
    pub octet6: libc::uint8_t,
    pub octet7: libc::uint8_t,
    pub octet8: libc::uint8_t,
    pub octet9: libc::uint8_t,
    pub octet10: libc::uint8_t,
    pub octet11: libc::uint8_t,
    pub octet12: libc::uint8_t,
    pub octet13: libc::uint8_t,
    pub octet14: libc::uint8_t,
    pub octet15: libc::uint8_t,
}

impl MXFKey {
    pub fn new(
        o0: u8,
        o1: u8,
        o2: u8,
        o3: u8,
        o4: u8,
        o5: u8,
        o6: u8,
        o7: u8,
        o8: u8,
        o9: u8,
        o10: u8,
        o11: u8,
        o12: u8,
        o13: u8,
        o14: u8,
        o15: u8,
    ) -> MXFKey {
        MXFKey {
            octet0: o0,
            octet1: o1,
            octet2: o2,
            octet3: o3,
            octet4: o4,
            octet5: o5,
            octet6: o6,
            octet7: o7,
            octet8: o8,
            octet9: o9,
            octet10: o10,
            octet11: o11,
            octet12: o12,
            octet13: o13,
            octet14: o14,
            octet15: o15,
        }
    }

    pub fn is_picture(&self) -> bool {
        unsafe {
            if mxf_is_picture(self).is_zero() {
                false
            } else {
                true
            }
        }
    }

    pub fn is_sound(&self) -> bool {
        unsafe {
            if mxf_is_sound(self).is_zero() {
                false
            } else {
                true
            }
        }
    }

    pub fn is_timecode(&self) -> bool {
        unsafe {
            if mxf_is_timecode(self).is_zero() {
                false
            } else {
                true
            }
        }
    }

    pub fn is_op_atom(&self) -> bool {
        unsafe {
            if mxf_is_op_atom(self).is_zero() {
                false
            } else {
                true
            }
        }
    }
}

impl From<*mut c_void> for MXFKey {
    fn from(raw: *mut c_void) -> Self {
        unsafe { (*(raw as *mut MXFKey)).clone() }
    }
}

impl std::cmp::PartialEq for MXFKey {
    fn eq(&self, other: &MXFKey) -> bool {
        self.octet0 == other.octet0
            && self.octet1 == other.octet1
            && self.octet2 == other.octet2
            && self.octet3 == other.octet3
            && self.octet4 == other.octet4
            && self.octet5 == other.octet5
            && self.octet6 == other.octet6
            && self.octet7 == other.octet7
            && self.octet8 == other.octet8
            && self.octet9 == other.octet9
            && self.octet10 == other.octet10
            && self.octet11 == other.octet11
            && self.octet12 == other.octet12
            && self.octet13 == other.octet13
            && self.octet14 == other.octet14
            && self.octet15 == other.octet15
    }
}

impl std::fmt::Debug for MXFKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}",
            self.octet0,
            self.octet1,
            self.octet2,
            self.octet3,
            self.octet4,
            self.octet5,
            self.octet6,
            self.octet7,
            self.octet8,
            self.octet9,
            self.octet10,
            self.octet11,
            self.octet12,
            self.octet13,
            self.octet14,
            self.octet15
        )
    }
}

extern "C" {
    /* mxf_file.h */
    fn mxf_disk_file_open_read(filename: *const c_char, mxffile: *mut *mut _MXFFile) -> c_int;
    fn mxf_file_close(mxffile: *mut *mut _MXFFile);

    /* mxf_utils.h */
    fn mxf_find_set_by_key(
        headerdata: *mut MXFHeaderMetadata,
        mxfkey: *const MXFKey,
        list: *mut *mut MXFList,
    ) -> c_int;
    fn mxf_utf16_to_utf8(u8_str: *mut u8, u16_str: *const uint16, u8_size: size_t) -> size_t;
    fn mxf_equals_umid(umid_a: *const MXFUmid, umid_b: *const MXFUmid) -> c_int;
    fn mxf_get_strongref(
        hederdata: *mut MXFHeaderMetadata,
        value: *const uint8,
        dataset: *mut *mut MXFMetadataSet,
    ) -> c_int;
    fn mxf_next_array_item_element(
        iter: *mut MXFArrayItemIterator,
        value: *mut *mut uint8,
        len: *mut uint32,
    ) -> c_int;
    fn mxf_initialise_array_item_iterator(
        set: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        iter: *mut MXFArrayItemIterator,
    ) -> c_int;

    /* mxf_partition.h*/
    fn mxf_read_header_pp_kl(
        mxffile: *mut _MXFFile,
        mxfkey: *mut MXFKey,
        llen: *mut uint8,
        len: *mut uint64,
    ) -> c_int;
    fn mxf_read_partition(
        mxffile: *mut _MXFFile,
        mxfkey: *const MXFKey,
        mxfpartition: *mut *mut MXFPartition,
    ) -> c_int;
    fn mxf_read_next_nonfiller_kl(
        mxffile: *mut _MXFFile,
        mxfkey: *const MXFKey,
        llen: *mut uint8,
        len: *mut uint64,
    ) -> c_int;
    fn mxf_free_partition(partition: *mut *mut MXFPartition);

    /* mxf_avid.h */
    fn mxf_avid_read_filtered_header_metadata(
        mxffile: *mut _MXFFile,
        skip_data_refs: c_int,
        headerdata: *mut MXFHeaderMetadata,
        header_byte_count: uint64,
        mxfkey: *const MXFKey,
        llen: uint8,
        len: uint64,
    ) -> c_int;
    fn mxf_avid_load_extensions(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_avid_get_mob_attribute(
        name: *const uint16,
        names: *const MXFList,
        values: *const MXFList,
        value: *mut *mut uint16,
    ) -> c_int;
    fn mxf_avid_read_string_mob_attributes(
        dataset: *mut MXFMetadataSet,
        names: *mut *mut MXFList,
        values: *mut *mut MXFList,
    ) -> c_int;
    fn mxf_avid_read_string_tagged_values(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        names: *mut *mut MXFList,
        values: *mut *mut MXFList,
    ) -> c_int;
    fn mxf_avid_read_string_tagged_value(
        dataset: *mut MXFMetadataSet,
        name: *mut *mut uint16,
        value: *mut *mut uint16,
    ) -> c_int;
    fn mxf_avid_get_data_def(
        headerdata: *mut MXFHeaderMetadata,
        uuid: *const MXFKey,
        datadef: *mut MXFKey,
    ) -> c_int;

    /* mxf_header_metadata.h */
    fn mxf_is_header_metadata(mxfkey: *const MXFKey) -> c_int;
    fn mxf_create_header_metadata(
        headerdata: *mut *mut MXFHeaderMetadata,
        datamodel: *const MXFDataModel,
    ) -> c_int;
    fn mxf_free_header_metadata(headerdata: *mut *mut MXFHeaderMetadata);
    fn mxf_find_singular_set_by_key(
        headerdata: *mut MXFHeaderMetadata,
        mxfkey: *const MXFKey,
        dataset: *mut *mut MXFMetadataSet,
    ) -> c_int;
    fn mxf_have_item(dataset: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> c_int;
    fn mxf_get_utf16string_item_size(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        size: *mut uint16,
    ) -> c_int;
    fn mxf_get_utf16string_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut uint16,
    ) -> c_int;
    fn mxf_get_rational_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        mxffractional: *mut MXFRational,
    ) -> c_int;
    fn mxf_get_uint8_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut uint8,
    ) -> c_int;
    fn mxf_get_uint16_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut uint16,
    ) -> c_int;
    fn mxf_get_uint32_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut uint32,
    ) -> c_int;
    fn mxf_get_int32_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut int32,
    ) -> c_int;
    fn mxf_get_umid_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        mxfumid: *mut MXFUmid,
    ) -> c_int;
    fn mxf_get_timestamp_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        mxftimestamp: *mut MXFTimestamp,
    ) -> c_int;
    fn mxf_get_array_item_count(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        count: *mut uint32,
    ) -> c_int;
    fn mxf_get_array_item_element(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        index: uint32,
        element: *mut *mut uint8,
    ) -> c_int;
    fn mxf_get_strongref_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        value: *mut *mut MXFMetadataSet,
    ) -> c_int;
    fn mxf_get_ul_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        mxful: *mut MXFKey,
    ) -> c_int;
    fn mxf_get_length_item(
        dataset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        val: *mut int64,
    ) -> c_int;
    fn mxf_get_position_item(
        datset: *mut MXFMetadataSet,
        mxfkey: *const MXFKey,
        pos: *mut int64,
    ) -> c_int;
    fn mxf_set_is_subclass_of(set: *mut MXFMetadataSet, mxfkey: *const MXFKey) -> c_int;

    /* mxf_data_model.h */
    fn mxf_load_data_model(datamodel: *mut *mut MXFDataModel) -> c_int;
    fn mxf_finalise_data_model(datamodel: *mut MXFDataModel) -> c_int;
    fn mxf_free_data_model(datamodel: *mut *mut MXFDataModel);
    fn mxf_is_subclass_of(
        datamodel: *mut MXFDataModel,
        setkey: *const MXFKey,
        mxfkey: *const MXFKey,
    ) -> c_int;

    /* mxf_label_and_keys.h */
    fn mxf_is_picture(key: *const MXFKey) -> c_int;
    fn mxf_is_sound(key: *const MXFKey) -> c_int;
    fn mxf_is_timecode(key: *const MXFKey) -> c_int;
    fn mxf_is_op_atom(mxful: *const MXFKey) -> c_int;

    /* mxf_list.h */
    fn mxf_free_list(list: *mut *mut MXFList);
    fn mxf_get_list_length(list: *mut MXFList) -> size_t;
    fn mxf_get_list_element(list: *mut MXFList, index: size_t) -> *mut c_void;
    fn mxf_initialise_list_iter(iter: *mut MXFListIterator, list: *const MXFList);
    fn mxf_next_list_iter_element(iter: *mut MXFListIterator) -> c_int;
    fn mxf_get_iter_element(iter: *mut MXFListIterator) -> *mut c_void;

    /* mxf_uu_metadata.h */
    fn mxf_uu_get_top_file_package(
        headerdata: *mut MXFHeaderMetadata,
        dataset: *mut *mut MXFMetadataSet,
    ) -> c_int;
    fn mxf_uu_get_package_tracks(
        set: *mut MXFMetadataSet,
        iter: *mut MXFArrayItemIterator,
    ) -> c_int;
    fn mxf_uu_next_track(
        headerdata: *mut MXFHeaderMetadata,
        iter: *mut MXFArrayItemIterator,
        set: *mut *mut MXFMetadataSet,
    ) -> c_int;
    fn mxf_uu_get_track_datadef(set: *mut MXFMetadataSet, key: *mut MXFKey) -> c_int;
    fn mxf_uu_get_track_duration(set: *mut MXFMetadataSet, duration: *mut i64) -> c_int;
    fn mxf_uu_get_referenced_package(
        headerdata: *mut MXFHeaderMetadata,
        source_pkg_id: *const MXFUmid,
        set: *mut *mut MXFMetadataSet,
    ) -> c_int;
}
