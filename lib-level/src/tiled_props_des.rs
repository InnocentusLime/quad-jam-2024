//! This module contains deserialization routines for tiled
//! properties. Properties can only be deserialized into a struct or map.
//!
//! ## Type Naming
//! The deserializer strictly checks that your type name equals the one
//! specified in the Tiled properties. This is to avoid misconfigurations
//! and make it easier to trace Tiled properties back to the code.
//!
//! Unfortunately, current version of `tiled-rs` does not report the user
//! type for things like enums. Because of that, this implementation can't
//! enforce the proper naming. It is, however, encouraged to make sure this
//! convention is preserved.
//!
//! ## Extra Fields
//! You might have some fields that are not from the extra properties list.
//! E.g. that can be the object position. Annotating them with `#[serde(skip)]`
//! might break other formats of the level. It is better to mark them with
//! `#[serde(default)]`, as if they are optional fields, and set them later.
//!  
//! ## Required Serde Attributes
//! Tiled does not include a value if its default value has not been changed.
//! Therefore, please mark all your fields with `#[serde(default)]`. Preferrably,
//! in such way that it agrees with the Tiled project's actual defaults. This
//! will make sure both Tiled and code side of things are working in unison.

use serde::{Deserializer, de};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializerError {
    #[error("Property {property:?}: expected a bool, found type {found_ty:?}")]
    ExpectedBool { property: String, found_ty: String },
    #[error("Property {property:?}: expected an integer, found type {found_ty:?}")]
    ExpectedInteger { property: String, found_ty: String },
    #[error("Property {property:?}: expected an integer or float, found type {found_ty:?}")]
    ExpectedIntegerOrFloat { property: String, found_ty: String },
    #[error("Property {property:?}: expected a string, found type {found_ty:?}")]
    ExpectedString { property: String, found_ty: String },
    #[error("Property {property:?}: expected a string or path, found type {found_ty:?}")]
    ExpectedStringOrPath { property: String, found_ty: String },
    #[error("Property {property:?}: expected a class, found type {found_ty:?}")]
    ExpectedClass { property: String, found_ty: String },
    #[error("Exepcted class to be {expected_class:?}, found {found_class:?}")]
    StructNameMismatch {
        expected_class: String,
        found_class: String,
    },
    #[error("Exepcted class to be one of {expected_classes:?}, found {found_class:?}")]
    NoSuchEnumVariant {
        expected_classes: Vec<String>,
        found_class: String,
    },
    #[error("Only struct enum variants can be deserializde from properties")]
    OnlyStructEnumVariants,
    #[error("Only structs and enums can be deserialized from properties")]
    OnlyStructsAndEnums,
    #[error("Deserializing into {requested_ty:?} is not supported")]
    UnsupportedType { requested_ty: String },
    #[error("{msg}")]
    Custom { msg: String },
}

pub fn from_properties<'a, T>(
    ty_name: &'a str,
    props: &'a tiled::Properties,
) -> Result<T, DeserializerError>
where
    T: de::Deserialize<'a>,
{
    let mut de = TiledPropertiesDeserializer { ty_name, props };
    de::Deserialize::deserialize(&mut de)
}

struct TiledPropertiesDeserializer<'de> {
    ty_name: &'de str,
    props: &'de tiled::Properties,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut TiledPropertiesDeserializer<'de> {
    type Error = DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(PropetyMapAccess {
            kv: None,
            props: self.props.iter(),
        })
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name != self.ty_name {
            return Err(DeserializerError::StructNameMismatch {
                expected_class: name.to_string(),
                found_class: self.ty_name.to_string(),
            });
        }
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if !variants.contains(&self.ty_name) {
            return Err(DeserializerError::NoSuchEnumVariant {
                expected_classes: variants.iter().map(|x| x.to_string()).collect(),
                found_class: self.ty_name.to_string(),
            });
        }
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructsAndEnums)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct TiledPropertyDeserializer<'de> {
    name: &'de str,
    prop: &'de tiled::PropertyValue,
}

impl de::Error for DeserializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom {
            msg: msg.to_string(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut TiledPropertyDeserializer<'de> {
    type Error = DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::BoolValue(b) => visitor.visit_bool(*b),
            tiled::PropertyValue::FloatValue(f) => visitor.visit_f32(*f),
            tiled::PropertyValue::IntValue(i) => visitor.visit_i32(*i),
            tiled::PropertyValue::ColorValue(_) => todo!("Colors are not supported"),
            tiled::PropertyValue::StringValue(s) => visitor.visit_str(s.as_str()),
            tiled::PropertyValue::FileValue(s) => visitor.visit_str(s.as_str()),
            tiled::PropertyValue::ObjectValue(_) => todo!("Object values are not supported"),
            tiled::PropertyValue::ClassValue { properties, .. } => {
                visitor.visit_map(PropetyMapAccess {
                    kv: None,
                    props: properties.iter(),
                })
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::BoolValue(b) => visitor.visit_bool(*b),
            _ => Err(DeserializerError::ExpectedBool {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_i8(*i as i8),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_i16(*i as i16),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_i32(*i as i32),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_i64(*i as i64),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_u8(*i as u8),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_u16(*i as u16),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_u32(*i as u32),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_u64(*i as u64),
            _ => Err(DeserializerError::ExpectedInteger {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_f32(*i as f32),
            tiled::PropertyValue::FloatValue(f) => visitor.visit_f32(*f),
            _ => Err(DeserializerError::ExpectedIntegerOrFloat {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::IntValue(i) => visitor.visit_f64(*i as f64),
            tiled::PropertyValue::FloatValue(f) => visitor.visit_f64(*f as f64),
            _ => Err(DeserializerError::ExpectedIntegerOrFloat {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "char".to_string(),
        })
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::StringValue(s) => visitor.visit_str(s.as_str()),
            tiled::PropertyValue::FileValue(s) => visitor.visit_str(s.as_str()),
            _ => Err(DeserializerError::ExpectedStringOrPath {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "bytes".to_string(),
        })
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "byte buf".to_string(),
        })
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "option".to_string(),
        })
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "unit".to_string(),
        })
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "unit struct".to_string(),
        })
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "newtype struct".to_string(),
        })
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "seq".to_string(),
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "tuple".to_string(),
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::UnsupportedType {
            requested_ty: "tuple struct".to_string(),
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::ClassValue { properties, .. } => {
                visitor.visit_map(PropetyMapAccess {
                    kv: None,
                    props: properties.iter(),
                })
            }
            _ => Err(DeserializerError::ExpectedClass {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::ClassValue {
                properties,
                property_type,
            } => {
                if property_type != name {
                    return Err(DeserializerError::StructNameMismatch {
                        expected_class: name.to_string(),
                        found_class: property_type.to_string(),
                    });
                }
                visitor.visit_map(PropetyMapAccess {
                    kv: None,
                    props: properties.iter(),
                })
            }
            _ => Err(DeserializerError::ExpectedClass {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::StringValue(s) => {
                visitor.visit_enum(de::value::StrDeserializer::new(s.as_str()))
            }
            _ => Err(DeserializerError::ExpectedString {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.prop {
            tiled::PropertyValue::StringValue(s) => visitor.visit_str(s.as_str()),
            _ => Err(DeserializerError::ExpectedString {
                property: self.name.to_string(),
                found_ty: property_type_name(self.prop),
            }),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

fn property_type_name(prop: &tiled::PropertyValue) -> String {
    match prop {
        tiled::PropertyValue::BoolValue(_) => "bool".to_string(),
        tiled::PropertyValue::FloatValue(_) => "float".to_string(),
        tiled::PropertyValue::IntValue(_) => "int".to_string(),
        tiled::PropertyValue::ColorValue(_) => "color".to_string(),
        tiled::PropertyValue::StringValue(_) => "string".to_string(),
        tiled::PropertyValue::FileValue(_) => "file".to_string(),
        tiled::PropertyValue::ObjectValue(_) => "object".to_string(),
        tiled::PropertyValue::ClassValue { property_type, .. } => property_type.clone(),
    }
}

struct PropetyMapAccess<'de> {
    kv: Option<(&'de str, &'de tiled::PropertyValue)>,
    props: std::collections::hash_map::Iter<'de, String, tiled::PropertyValue>,
}

impl<'a, 'de> de::EnumAccess<'de> for &'a mut TiledPropertiesDeserializer<'de> {
    type Error = DeserializerError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(de::value::StrDeserializer::new(self.ty_name))?;
        Ok((val, self))
    }
}

impl<'a, 'de> de::VariantAccess<'de> for &'a mut TiledPropertiesDeserializer<'de> {
    type Error = DeserializerError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(DeserializerError::OnlyStructEnumVariants)
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(DeserializerError::OnlyStructEnumVariants)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializerError::OnlyStructEnumVariants)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}

impl<'de> de::MapAccess<'de> for PropetyMapAccess<'de> {
    type Error = DeserializerError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.props.next() else {
            return Ok(None);
        };
        self.kv = Some((key.as_str(), value));

        seed.deserialize(de::value::StrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let (key, prop) = self.kv.unwrap();
        seed.deserialize(&mut TiledPropertyDeserializer { name: key, prop })
    }
}
