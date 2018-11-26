use std::ptr;
use std::fmt;

/* C Types */
pub type uint8  = libc::uint8_t;
pub type uint16 = libc::uint16_t;
pub type uint32 = libc::uint32_t;
pub type uint64 = libc::uint64_t;
pub type int64  = libc::int64_t;
pub type int16  = libc::int16_t;
pub type int32  = libc::int32_t;
pub type size_t = libc::size_t;
pub type c_char = libc::c_char;
pub type c_int  = libc::c_int;
pub type c_void  = libc::c_void;

/* Stub types */
pub enum MXFFile {}
pub enum MXFDataModel {}
pub enum MXFPrimerPack {}
pub type MXFLength = int64;

pub enum AvidPhysicalPackageType {
    UnkownPhysType,
    TapePhysType,
    ImportPhysType,
    RecordingPhysType,
}

impl Default for AvidPhysicalPackageType {
    fn default() -> AvidPhysicalPackageType { AvidPhysicalPackageType::UnkownPhysType }
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
    PCM
}

impl Default for AvidEssenceType {
    fn default() -> AvidEssenceType { AvidEssenceType::Unknown }
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
    pub fn new(o0: u8, o1: u8, o2: u8, o3: u8, o4: u8, o5: u8, o6: u8, o7: u8,
           o8: u8, o9: u8, o10: u8, o11: u8, o12: u8, o13: u8, o14: u8, o15: u8) -> MXFKey {
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
}

impl std::cmp::PartialEq for MXFKey {
    fn eq(&self, other: &MXFKey) -> bool {
        self.octet0 == other.octet0 && self.octet1 == other.octet1 && self.octet2 == other.octet2 && self.octet3 == other.octet3 && self.octet4 == other.octet4 &&
            self.octet5 == other.octet5 && self.octet6 == other.octet6 && self.octet7 == other.octet7 && self.octet8 == other.octet8 && self.octet9 == other.octet9 &&
            self.octet10 == other.octet10 && self.octet11 == other.octet11 && self.octet12 == other.octet12 && self.octet13 == other.octet13 && self.octet14 == other.octet14 &&
            self.octet15 == other.octet15
    }
}

impl std::fmt::Debug for MXFKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}",
               self.octet0, self.octet1, self.octet2, self.octet3, self.octet4, self.octet5, self.octet6, self.octet7,
               self.octet8, self.octet9, self.octet10, self.octet11, self.octet12, self.octet13, self.octet14, self.octet15)
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
    pub fn new(o0: u8, o1: u8, o2: u8, o3: u8, o4: u8, o5: u8, o6: u8, o7: u8,
           o8: u8, o9: u8, o10: u8, o11: u8, o12: u8, o13: u8, o14: u8, o15: u8,
           o16: u8, o17: u8, o18: u8, o19: u8, o20: u8, o21: u8, o22: u8, o23: u8,
           o24: u8, o25: u8, o26: u8, o27: u8, o28: u8, o29: u8, o30: u8, o31: u8) -> MXFUmid {
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
        self.octet0 == other.octet0 && self.octet1 == other.octet1 && self.octet2 == other.octet2 && self.octet3 == other.octet3 && self.octet4 == other.octet4 &&
            self.octet5 == other.octet5 && self.octet6 == other.octet6 && self.octet7 == other.octet7 && self.octet8 == other.octet8 && self.octet9 == other.octet9 &&
            self.octet10 == other.octet10 && self.octet11 == other.octet11 && self.octet12 == other.octet12 && self.octet13 == other.octet13 && self.octet14 == other.octet14 &&
            self.octet15 == other.octet15 && self.octet16 == other.octet16 && self.octet16 == other.octet16 && self.octet17 == other.octet17 && self.octet18 == other.octet18 &&
            self.octet19 == other.octet19 && self.octet20 == other.octet20 && self.octet21 == other.octet21 && self.octet22 == other.octet22 && self.octet23 == other.octet23 &&
            self.octet24 == other.octet24 && self.octet25 == other.octet25 && self.octet26 == other.octet26 && self.octet27 == other.octet27 && self.octet28 == other.octet28 &&
            self.octet29 == other.octet29 && self.octet30 == other.octet30 && self.octet31 == other.octet31
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

#[derive(Copy, Clone, Debug)]
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

/* MXF Header Metadata */
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MXFHeaderMetadata {
    pub datamodel: *mut MXFDataModel,
    pub primerpack: *mut MXFPrimerPack,
    pub sets: MXFList,
}

impl std::fmt::Debug for MXFHeaderMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, " MXFHeaderMetadata (\n\
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

impl std::fmt::Debug for MXFMetadataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            write!(f, " MXFMetadataSet (\n\
                key: {:?}\n\
                instance_uid: {:?}\n\
                items: \n{:?}\n\
                header_metadata: \n{:?}\n\
                fixed_space_allocation: {:?}\n\
                )",
            self.key, self.instance_uid, self.items, *self.header_metadata, self.fixed_space_allocation)
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
        write!(f, "{}.{}.{} {}:{}:{}.{}", self.day, self.month, self.year, self.hour, self.min, self.sec, self.qmsec)
    }
}

