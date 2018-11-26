/* NOTE: concat_idents! is not stable yet
macro_rules! mxf_set_k {
    ($name: ident) => (
        concat_idents!(G_, $name, _SET_KEY);
    )
}

macro_rules! mxf_item_k {
    ($setname: ident, $name: ident) => (
        concat_idents!(G_, $setname, _, $name, _ITEM_KEY);
    )
}
*/

macro_rules! mxf_set_definition {
    ($name: ident, $o0: tt, $o1: tt, $o2: tt, $o3: tt, $o4: tt, $o5: tt, $o6: tt, $o7: tt,
     $o8: tt, $o9: tt, $o10: tt, $o11: tt, $o12: tt, $o13: tt, $o14: tt, $o15: tt) => (
        pub const $name: MXFKey = MXFKey {
            octet0: $o0,
            octet1: $o1,
            octet2: $o2,
            octet3: $o3,
            octet4: $o4,
            octet5: $o5,
            octet6: $o6,
            octet7: $o7,
            octet8: $o8,
            octet9: $o9,
            octet10: $o10,
            octet11: $o11,
            octet12: $o12,
            octet13: $o13,
            octet14: $o14,
            octet15: $o15,
        };
    )
}

macro_rules! mxf_item_definition {
    ($name: ident, $o0: tt, $o1: tt, $o2: tt, $o3: tt, $o4: tt, $o5: tt, $o6: tt, $o7: tt,
     $o8: tt, $o9: tt, $o10: tt, $o11: tt, $o12: tt, $o13: tt, $o14: tt, $o15: tt) => (
        pub const $name: MXFKey = MXFKey {
            octet0: $o0,
            octet1: $o1,
            octet2: $o2,
            octet3: $o3,
            octet4: $o4,
            octet5: $o5,
            octet6: $o6,
            octet7: $o7,
            octet8: $o8,
            octet9: $o9,
            octet10: $o10,
            octet11: $o11,
            octet12: $o12,
            octet13: $o13,
            octet14: $o14,
            octet15: $o15,
        };
    )
}

#[macro_export]
macro_rules! check {
    ($meth_name: ident,
     $msg: tt,
     $($var_name: expr),*) => (
        if $meth_name($($var_name,)*) == 0 {
            return Err($msg.to_string());
        }
    )
}
