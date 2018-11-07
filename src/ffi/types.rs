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
#[repr(C)]
#[derive(Debug)]
pub struct AvidNameValuePair {
    pub name: *mut c_char,
    pub value: *mut c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct AvidTaggedValue {
    pub name: *mut c_char,
    pub value: *mut c_char,
    pub atributes: *mut AvidNameValuePair,
    pub num_attributes: c_int,
}

/* MXF Key */
#[repr(C)]
#[derive(Default)]
pub struct MXFkey {
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

impl std::fmt::Debug for MXFkey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}",
               self.octet0, self.octet1, self.octet2, self.octet3, self.octet4, self.octet5, self.octet6, self.octet7,
               self.octet8, self.octet9, self.octet10, self.octet11, self.octet12, self.octet13, self.octet14, self.octet15)
    }
}

#[repr(C)]
#[derive(Default)]
pub struct MXFumid {
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

impl std::fmt::Debug for MXFumid {
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
#[derive(Clone, Debug)]
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
    key: MXFkey,
    major_version: uint16,
    minor_version: uint16,
    kag_size: uint32,
    this_partition: uint64,
    previous_partition: uint64,
    footer_partition: uint64,
    pub header_byte_count: uint64,
    index_byte_count: uint64,
    indes_sid: uint32,
    body_offset: uint64,
    body_sid: uint32,
    pub operational_pattern: MXFkey,
    essence_containers: MXFList,
    header_mark_in_pos: int64,
    index_mark_in_pos: int64,
}

/* MXF Header Metadata */
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MXFHeaderMetadata {
    datamodel: *mut MXFDataModel,
    primerpack: *mut MXFPrimerPack,
    sets: MXFList,
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
    pub key: MXFkey,
    pub instance_uid: MXFkey,
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

/* MXF Rational */
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct MXFRational {
    numerator: int32,
    denominator: int32,
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

