use std::ptr;

/* C Types */
pub type uint8  = libc::uint8_t;
pub type uint16 = libc::uint16_t;
pub type uint32 = libc::uint32_t;
pub type uint64 = libc::uint64_t;
pub type int64  = libc::int64_t;
pub type int32  = libc::int32_t;
pub type size_t = libc::size_t;
pub type c_char = libc::c_char;
pub type c_int  = libc::c_int;

/* Stub types */
pub enum MXFFile {}
pub enum MXFDataModel {}
pub enum MXFPrimerPack {}

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

/* MXF List */
pub type free_func_type = Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>;

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
    pub data: *mut libc::c_void,
}

impl Default for MXFListElement {
    fn default() -> MXFListElement {
        MXFListElement {
            next: ptr::null_mut(),
            data: ptr::null_mut(),
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


/* MXF Rational */
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct MXFRational {
    numerator: int32,
    denominator: int32,
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

