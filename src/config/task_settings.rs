use std::collections::HashSet;

use deno_runtime::permissions::PermissionsOptions as DenoPermissionsOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
/// Task behavior settings.
pub struct TaskSettings {
    /// Dependent Task Name
    #[serde(default)]
    pub depends: HashSet<String>,
    /// Deno behavior settings.
    #[serde(default)]
    pub deno: DenoSettings,
}

#[derive(Serialize, Deserialize, Default)]
/// Deno behavior settings.
pub struct DenoSettings {
    #[serde(flatten)]
    #[serde(deserialize_with = "deserializer::deserialize")]
    /// Deno Permission.
    pub permissions: DenoPermissionsOptions,
}

mod deserializer {
    use deno_runtime::permissions::PermissionsOptions;
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    pub fn deserialize<'de, __D>(
        __deserializer: __D,
    ) -> _serde::__private::Result<PermissionsOptions, __D::Error>
    where
        __D: _serde::Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        enum __Field {
            __field0,
            __field1,
            __field2,
            __field3,
            __field4,
            __field5,
            __field6,
            __field7,
            __field8,
            __field9,
            __field10,
            __field11,
            __field12,
            __field13,
            __field14,
            __field15,
            __field16,
            __ignore,
        }
        #[doc(hidden)]
        struct __FieldVisitor;
        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
            type Value = __Field;
            fn expecting(
                &self,
                __formatter: &mut _serde::__private::Formatter,
            ) -> _serde::__private::fmt::Result {
                _serde::__private::Formatter::write_str(__formatter, "field identifier")
            }
            fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
            where
                __E: _serde::de::Error,
            {
                match __value {
                    0u64 => _serde::__private::Ok(__Field::__field0),
                    1u64 => _serde::__private::Ok(__Field::__field1),
                    2u64 => _serde::__private::Ok(__Field::__field2),
                    3u64 => _serde::__private::Ok(__Field::__field3),
                    4u64 => _serde::__private::Ok(__Field::__field4),
                    5u64 => _serde::__private::Ok(__Field::__field5),
                    6u64 => _serde::__private::Ok(__Field::__field6),
                    7u64 => _serde::__private::Ok(__Field::__field7),
                    8u64 => _serde::__private::Ok(__Field::__field8),
                    9u64 => _serde::__private::Ok(__Field::__field9),
                    10u64 => _serde::__private::Ok(__Field::__field10),
                    11u64 => _serde::__private::Ok(__Field::__field11),
                    12u64 => _serde::__private::Ok(__Field::__field12),
                    13u64 => _serde::__private::Ok(__Field::__field13),
                    14u64 => _serde::__private::Ok(__Field::__field14),
                    15u64 => _serde::__private::Ok(__Field::__field15),
                    16u64 => _serde::__private::Ok(__Field::__field16),
                    _ => _serde::__private::Ok(__Field::__ignore),
                }
            }
            fn visit_str<__E>(self, __value: &str) -> _serde::__private::Result<Self::Value, __E>
            where
                __E: _serde::de::Error,
            {
                match __value {
                    "allow_env" => _serde::__private::Ok(__Field::__field0),
                    "deny_env" => _serde::__private::Ok(__Field::__field1),
                    "allow_hrtime" => _serde::__private::Ok(__Field::__field2),
                    "deny_hrtime" => _serde::__private::Ok(__Field::__field3),
                    "allow_net" => _serde::__private::Ok(__Field::__field4),
                    "deny_net" => _serde::__private::Ok(__Field::__field5),
                    "allow_ffi" => _serde::__private::Ok(__Field::__field6),
                    "deny_ffi" => _serde::__private::Ok(__Field::__field7),
                    "allow_read" => _serde::__private::Ok(__Field::__field8),
                    "deny_read" => _serde::__private::Ok(__Field::__field9),
                    "allow_run" => _serde::__private::Ok(__Field::__field10),
                    "deny_run" => _serde::__private::Ok(__Field::__field11),
                    "allow_sys" => _serde::__private::Ok(__Field::__field12),
                    "deny_sys" => _serde::__private::Ok(__Field::__field13),
                    "allow_write" => _serde::__private::Ok(__Field::__field14),
                    "deny_write" => _serde::__private::Ok(__Field::__field15),
                    "prompt" => _serde::__private::Ok(__Field::__field16),
                    _ => _serde::__private::Ok(__Field::__ignore),
                }
            }
            fn visit_bytes<__E>(self, __value: &[u8]) -> _serde::__private::Result<Self::Value, __E>
            where
                __E: _serde::de::Error,
            {
                match __value {
                    b"allow_env" => _serde::__private::Ok(__Field::__field0),
                    b"deny_env" => _serde::__private::Ok(__Field::__field1),
                    b"allow_hrtime" => _serde::__private::Ok(__Field::__field2),
                    b"deny_hrtime" => _serde::__private::Ok(__Field::__field3),
                    b"allow_net" => _serde::__private::Ok(__Field::__field4),
                    b"deny_net" => _serde::__private::Ok(__Field::__field5),
                    b"allow_ffi" => _serde::__private::Ok(__Field::__field6),
                    b"deny_ffi" => _serde::__private::Ok(__Field::__field7),
                    b"allow_read" => _serde::__private::Ok(__Field::__field8),
                    b"deny_read" => _serde::__private::Ok(__Field::__field9),
                    b"allow_run" => _serde::__private::Ok(__Field::__field10),
                    b"deny_run" => _serde::__private::Ok(__Field::__field11),
                    b"allow_sys" => _serde::__private::Ok(__Field::__field12),
                    b"deny_sys" => _serde::__private::Ok(__Field::__field13),
                    b"allow_write" => _serde::__private::Ok(__Field::__field14),
                    b"deny_write" => _serde::__private::Ok(__Field::__field15),
                    b"prompt" => _serde::__private::Ok(__Field::__field16),
                    _ => _serde::__private::Ok(__Field::__ignore),
                }
            }
        }
        impl<'de> _serde::Deserialize<'de> for __Field {
            #[inline]
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
            }
        }
        #[doc(hidden)]
        struct __Visitor<'de> {
            marker: _serde::__private::PhantomData<PermissionsOptions>,
            lifetime: _serde::__private::PhantomData<&'de ()>,
        }
        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
            type Value = PermissionsOptions;
            fn expecting(
                &self,
                __formatter: &mut _serde::__private::Formatter,
            ) -> _serde::__private::fmt::Result {
                _serde::__private::Formatter::write_str(__formatter, "struct PermissionsOptions")
            }
            #[inline]
            fn visit_seq<__A>(
                self,
                mut __seq: __A,
            ) -> _serde::__private::Result<Self::Value, __A::Error>
            where
                __A: _serde::de::SeqAccess<'de>,
            {
                let __field0 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field1 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field2 = match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)? {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            2usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field3 = match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)? {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            3usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field4 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                4usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field5 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                5usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field6 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            6usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field7 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            7usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field8 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            8usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field9 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            9usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field10 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                10usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field11 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                11usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field12 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                12usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field13 =
                    match _serde::de::SeqAccess::next_element::<Option<Vec<String>>>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                13usize,
                                &"struct PermissionsOptions with 17 elements",
                            ));
                        }
                    };
                let __field14 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            14usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field15 = match _serde::de::SeqAccess::next_element::<
                    Option<Vec<std::path::PathBuf>>,
                >(&mut __seq)?
                {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            15usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                let __field16 = match _serde::de::SeqAccess::next_element::<bool>(&mut __seq)? {
                    _serde::__private::Some(__value) => __value,
                    _serde::__private::None => {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            16usize,
                            &"struct PermissionsOptions with 17 elements",
                        ));
                    }
                };
                _serde::__private::Ok(PermissionsOptions {
                    allow_env: __field0,
                    deny_env: __field1,
                    allow_hrtime: __field2,
                    deny_hrtime: __field3,
                    allow_net: __field4,
                    deny_net: __field5,
                    allow_ffi: __field6,
                    deny_ffi: __field7,
                    allow_read: __field8,
                    deny_read: __field9,
                    allow_run: __field10,
                    deny_run: __field11,
                    allow_sys: __field12,
                    deny_sys: __field13,
                    allow_write: __field14,
                    deny_write: __field15,
                    prompt: __field16,
                })
            }
            #[inline]
            fn visit_map<__A>(
                self,
                mut __map: __A,
            ) -> _serde::__private::Result<Self::Value, __A::Error>
            where
                __A: _serde::de::MapAccess<'de>,
            {
                let mut __field0: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field1: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field2: _serde::__private::Option<bool> = _serde::__private::None;
                let mut __field3: _serde::__private::Option<bool> = _serde::__private::None;
                let mut __field4: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field5: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field6: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field7: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field8: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field9: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field10: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field11: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field12: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field13: _serde::__private::Option<Option<Vec<String>>> =
                    _serde::__private::None;
                let mut __field14: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field15: _serde::__private::Option<Option<Vec<std::path::PathBuf>>> =
                    _serde::__private::None;
                let mut __field16: _serde::__private::Option<bool> = _serde::__private::None;
                while let _serde::__private::Some(__key) =
                    _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                {
                    match __key {
                        __Field::__field0 => {
                            if _serde::__private::Option::is_some(&__field0) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("allow_env"),
                                );
                            }
                            __field0 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field1 => {
                            if _serde::__private::Option::is_some(&__field1) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_env"),
                                );
                            }
                            __field1 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field2 => {
                            if _serde::__private::Option::is_some(&__field2) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                        "allow_hrtime",
                                    ),
                                );
                            }
                            __field2 = _serde::__private::Some(
                                _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                            );
                        }
                        __Field::__field3 => {
                            if _serde::__private::Option::is_some(&__field3) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                        "deny_hrtime",
                                    ),
                                );
                            }
                            __field3 = _serde::__private::Some(
                                _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                            );
                        }
                        __Field::__field4 => {
                            if _serde::__private::Option::is_some(&__field4) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("allow_net"),
                                );
                            }
                            __field4 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field5 => {
                            if _serde::__private::Option::is_some(&__field5) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_net"),
                                );
                            }
                            __field5 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field6 => {
                            if _serde::__private::Option::is_some(&__field6) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("allow_ffi"),
                                );
                            }
                            __field6 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field7 => {
                            if _serde::__private::Option::is_some(&__field7) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_ffi"),
                                );
                            }
                            __field7 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field8 => {
                            if _serde::__private::Option::is_some(&__field8) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                        "allow_read",
                                    ),
                                );
                            }
                            __field8 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field9 => {
                            if _serde::__private::Option::is_some(&__field9) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_read"),
                                );
                            }
                            __field9 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field10 => {
                            if _serde::__private::Option::is_some(&__field10) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("allow_run"),
                                );
                            }
                            __field10 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field11 => {
                            if _serde::__private::Option::is_some(&__field11) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_run"),
                                );
                            }
                            __field11 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field12 => {
                            if _serde::__private::Option::is_some(&__field12) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("allow_sys"),
                                );
                            }
                            __field12 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field13 => {
                            if _serde::__private::Option::is_some(&__field13) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("deny_sys"),
                                );
                            }
                            __field13 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<String>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field14 => {
                            if _serde::__private::Option::is_some(&__field14) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                        "allow_write",
                                    ),
                                );
                            }
                            __field14 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field15 => {
                            if _serde::__private::Option::is_some(&__field15) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                        "deny_write",
                                    ),
                                );
                            }
                            __field15 =
                                _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                    Option<Vec<std::path::PathBuf>>,
                                >(
                                    &mut __map
                                )?);
                        }
                        __Field::__field16 => {
                            if _serde::__private::Option::is_some(&__field16) {
                                return _serde::__private::Err(
                                    <__A::Error as _serde::de::Error>::duplicate_field("prompt"),
                                );
                            }
                            __field16 = _serde::__private::Some(
                                _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                            );
                        }
                        _ => {
                            let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                &mut __map,
                            )?;
                        }
                    }
                }
                // let __field0 = match __field0 {
                //     _serde::__private::Some(__field0) => __field0,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_env")?,
                // };
                // let __field1 = match __field1 {
                //     _serde::__private::Some(__field1) => __field1,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_env")?,
                // };
                // let __field2 = match __field2 {
                //     _serde::__private::Some(__field2) => __field2,
                //     _serde::__private::None => {
                //         _serde::__private::de::missing_field("allow_hrtime")?
                //     }
                // };
                // let __field3 = match __field3 {
                //     _serde::__private::Some(__field3) => __field3,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_hrtime")?,
                // };
                // let __field4 = match __field4 {
                //     _serde::__private::Some(__field4) => __field4,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_net")?,
                // };
                // let __field5 = match __field5 {
                //     _serde::__private::Some(__field5) => __field5,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_net")?,
                // };
                // let __field6 = match __field6 {
                //     _serde::__private::Some(__field6) => __field6,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_ffi")?,
                // };
                // let __field7 = match __field7 {
                //     _serde::__private::Some(__field7) => __field7,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_ffi")?,
                // };
                // let __field8 = match __field8 {
                //     _serde::__private::Some(__field8) => __field8,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_read")?,
                // };
                // let __field9 = match __field9 {
                //     _serde::__private::Some(__field9) => __field9,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_read")?,
                // };
                // let __field10 = match __field10 {
                //     _serde::__private::Some(__field10) => __field10,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_run")?,
                // };
                // let __field11 = match __field11 {
                //     _serde::__private::Some(__field11) => __field11,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_run")?,
                // };
                // let __field12 = match __field12 {
                //     _serde::__private::Some(__field12) => __field12,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_sys")?,
                // };
                // let __field13 = match __field13 {
                //     _serde::__private::Some(__field13) => __field13,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_sys")?,
                // };
                // let __field14 = match __field14 {
                //     _serde::__private::Some(__field14) => __field14,
                //     _serde::__private::None => _serde::__private::de::missing_field("allow_write")?,
                // };
                // let __field15 = match __field15 {
                //     _serde::__private::Some(__field15) => __field15,
                //     _serde::__private::None => _serde::__private::de::missing_field("deny_write")?,
                // };
                // let __field16 = match __field16 {
                //     _serde::__private::Some(__field16) => __field16,
                //     _serde::__private::None => _serde::__private::de::missing_field("prompt")?,
                // };
                _serde::__private::Ok(PermissionsOptions {
                    allow_env: __field0.unwrap_or_default(),
                    deny_env: __field1.unwrap_or_default(),
                    allow_hrtime: __field2.unwrap_or(true),
                    deny_hrtime: __field3.unwrap_or(false),
                    allow_net: __field4.unwrap_or_default(),
                    deny_net: __field5.unwrap_or_default(),
                    allow_ffi: __field6.unwrap_or_default(),
                    deny_ffi: __field7.unwrap_or_default(),
                    allow_read: __field8.unwrap_or_default(),
                    deny_read: __field9.unwrap_or_default(),
                    allow_run: __field10.unwrap_or_default(),
                    deny_run: __field11.unwrap_or_default(),
                    allow_sys: __field12.unwrap_or_default(),
                    deny_sys: __field13.unwrap_or_default(),
                    allow_write: __field14.unwrap_or_default(),
                    deny_write: __field15.unwrap_or_default(),
                    prompt: __field16.unwrap_or(false),
                })
            }
        }
        #[doc(hidden)]
        const FIELDS: &[&str] = &[
            "allow_env",
            "deny_env",
            "allow_hrtime",
            "deny_hrtime",
            "allow_net",
            "deny_net",
            "allow_ffi",
            "deny_ffi",
            "allow_read",
            "deny_read",
            "allow_run",
            "deny_run",
            "allow_sys",
            "deny_sys",
            "allow_write",
            "deny_write",
            "prompt",
        ];
        _serde::Deserializer::deserialize_struct(
            __deserializer,
            "PermissionsOptions",
            FIELDS,
            __Visitor {
                marker: _serde::__private::PhantomData::<PermissionsOptions>,
                lifetime: _serde::__private::PhantomData,
            },
        )
    }
}
