use serde::ser::{
    self, Impossible, SerializeMap, SerializeStruct, SerializeStructVariant, Serializer,
};
use serde::Serialize;
use std::borrow::Cow;
mod key;
use key::Key;

struct NestedSeq<'a, S> {
    parent: PhabNestedSerializer<'a, S>,
    index: u32,
}

impl<'a, S> NestedSeq<'a, S> {
    fn new(parent: PhabNestedSerializer<'a, S>) -> Self {
        NestedSeq { parent, index: 0 }
    }
}

impl<'a, S> ser::SerializeSeq for NestedSeq<'a, S>
where
    S: ser::SerializeMap,
{
    type Ok = ();
    type Error = S::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = self.parent.nest(self.index);
        self.index += 1;

        value.serialize(s)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct NestedMap<'a, S> {
    parent: PhabNestedSerializer<'a, S>,
    key: Option<Cow<'a, str>>,
}

impl<S> SerializeMap for NestedMap<'_, S>
where
    S: ser::SerializeMap,
{
    type Ok = ();
    type Error = S::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = Key::<S>::new();
        self.key = Some(key.serialize(s)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if let Some(key) = self.key.take() {
            let s = self.parent.nest(key);
            value.serialize(s)?;
            Ok(())
        } else {
            Err(ser::Error::custom("value serialized before key"))
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct PhabNestedSerializer<'a, S> {
    inner: &'a mut S,
    prefix: Cow<'static, str>,
}

impl<'a, S> PhabNestedSerializer<'a, S> {
    fn new(s: &'a mut S, prefix: Cow<'static, str>) -> Self {
        PhabNestedSerializer { inner: s, prefix }
    }

    fn append<D>(self, index: D) -> Self
    where
        D: std::fmt::Display,
    {
        let prefix = format!("{}[{}]", self.prefix, index);
        PhabNestedSerializer::new(self.inner, prefix.into())
    }

    fn nest<D>(&mut self, index: D) -> PhabNestedSerializer<'_, S>
    where
        D: std::fmt::Display,
    {
        let prefix = format!("{}[{}]", self.prefix, index);

        PhabNestedSerializer::new(&mut self.inner, prefix.into())
    }
}

impl<'a, S> Serializer for PhabNestedSerializer<'a, S>
where
    S: ser::SerializeMap,
{
    type Ok = ();
    type Error = S::Error;

    type SerializeSeq = NestedSeq<'a, S>;
    type SerializeTuple = Impossible<Self::Ok, S::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, S::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, S::Error>;
    type SerializeMap = NestedMap<'a, S>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_entry(&self.prefix, &v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let s = self.append(variant);
        value.serialize(s)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(NestedSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ser::Error::custom("tuple unsupported"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ser::Error::custom("tuple struct unsupported"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ser::Error::custom("tuple variant unsupported"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(NestedMap {
            parent: self,
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self.append(variant))
    }
}

impl<'a, S> SerializeStruct for PhabNestedSerializer<'a, S>
where
    S: ser::SerializeMap,
{
    type Ok = ();
    type Error = S::Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = self.nest(key);
        value.serialize(s)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> SerializeStructVariant for PhabNestedSerializer<'a, S>
where
    S: ser::SerializeMap,
{
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = self.nest(key);
        value.serialize(s)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct PhabSerializeStruct<S> {
    inner: S,
}

impl<S> SerializeStruct for PhabSerializeStruct<S>
where
    S: ser::SerializeMap,
{
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = PhabNestedSerializer::new(&mut self.inner, key.into());
        value.serialize(s)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

struct MapSerializer<S> {
    inner: S,
    key: Option<Cow<'static, str>>,
}

impl<S> SerializeMap for MapSerializer<S>
where
    S: ser::SerializeMap,
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let s = Key::<Self>::new();
        self.key = Some(key.serialize(s)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if let Some(key) = self.key.take() {
            let s = PhabNestedSerializer::new(&mut self.inner, key);
            value.serialize(s)?;
            Ok(())
        } else {
            Err(ser::Error::custom("value serialized before key"))
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

struct PhabSerializer<S> {
    inner: S,
}

impl<S> Serializer for PhabSerializer<S>
where
    S: Serializer,
{
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = MapSerializer<S::SerializeMap>;
    type SerializeStruct = PhabSerializeStruct<S::SerializeMap>;
    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut sm = self.inner.serialize_map(None)?;
        let s = PhabNestedSerializer::new(&mut sm, variant.into());
        value.serialize(s)?;
        sm.end()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.inner.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.inner.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.inner
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let sm = self.inner.serialize_map(None)?;
        Ok(MapSerializer {
            inner: sm,
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let sm = self.inner.serialize_map(None)?;
        Ok(PhabSerializeStruct { inner: sm })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.inner
            .serialize_struct_variant(name, variant_index, variant, len)
    }
}

pub(crate) fn serialize_phab<S, D>(data: &D, s: S) -> Result<S::Ok, S::Error>
where
    D: Serialize,
    S: Serializer,
{
    let phab = PhabSerializer { inner: s };
    data.serialize(phab)
}

#[cfg(test)]
mod test {
    use serde::Serialize;
    use std::collections::HashMap;

    fn compare<L, R>(left: L, right: R)
    where
        L: Serialize,
        R: Serialize,
    {
        let l = serde_urlencoded::to_string(&left).unwrap();
        let r = serde_urlencoded::to_string(&right).unwrap();
        assert_eq!(l, r);
    }

    #[derive(Debug, Serialize)]
    struct Wrap<T>
    where
        T: Serialize,
    {
        #[serde(flatten, serialize_with = "super::serialize_phab")]
        w: T,
    }

    #[test]
    fn basic() {
        #[derive(Debug, Serialize)]
        struct Test<'a> {
            badger: &'a str,
        }

        let w = Wrap {
            w: Test { badger: "mushroom" },
        };

        let encoded = serde_urlencoded::to_string(w).unwrap();
        assert_eq!(encoded, "badger=mushroom");
    }

    #[test]
    fn simple_array() {
        #[derive(Debug, Serialize)]
        struct Test<'a> {
            badgers: Vec<&'a str>,
        };

        let t = Test {
            badgers: vec!["mushroom", "snake"],
        };
        let w = Wrap { w: t };

        let encoded = serde_urlencoded::to_string(&w).unwrap();
        assert_eq!(encoded, "badgers%5B0%5D=mushroom&badgers%5B1%5D=snake");
    }

    #[test]
    fn option_array() {
        #[derive(Debug, Serialize)]
        struct Test<'a> {
            badgers: Option<Vec<&'a str>>,
        };

        let t = Test {
            badgers: Some(vec!["mushroom", "snake"]),
        };

        let w = Wrap { w: t };
        let encoded = serde_urlencoded::to_string(&w).unwrap();
        assert_eq!(encoded, "badgers%5B0%5D=mushroom&badgers%5B1%5D=snake");
    }

    #[test]
    fn option_array_none() {
        #[derive(Debug, Serialize)]
        struct Test<'a> {
            badgers: Option<Vec<&'a str>>,
        };

        let t = Test { badgers: None };
        let w = Wrap { w: t };
        let encoded = serde_urlencoded::to_string(&w).unwrap();
        assert_eq!(encoded, "");
    }

    #[test]
    fn nested_array() {
        #[derive(Debug, Serialize)]
        struct Items<'a> {
            items: Vec<&'a str>,
        }

        #[derive(Debug, Serialize)]
        struct Test<'a> {
            badgers: Vec<Items<'a>>,
        };

        let i0 = Items {
            items: vec!["a", "b"],
        };
        let i1 = Items {
            items: vec!["c", "d"],
        };

        let t = Test {
            badgers: vec![i0, i1],
        };

        let w = Wrap { w: t };
        let expected = &[
            ("badgers[0][items][0]", "a"),
            ("badgers[0][items][1]", "b"),
            ("badgers[1][items][0]", "c"),
            ("badgers[1][items][1]", "d"),
        ];

        let encoded = serde_urlencoded::to_string(&w).unwrap();
        let expected = serde_urlencoded::to_string(&expected).unwrap();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn newtype_variant() {
        #[derive(Debug, Serialize)]
        enum E {
            N(u8),
        };
        let w = Wrap { w: E::N(42) };
        let expected = &[("N", "42")];

        compare(w, expected);
    }

    #[test]
    fn newtype_variant_seq() {
        #[derive(Debug, Serialize)]
        enum E {
            N(Vec<u8>),
        };

        let w = Wrap { w: E::N(vec![42]) };
        let expected = &[("N[0]", "42")];

        compare(w, expected);
    }

    #[test]
    fn nested_newtype_variant() {
        #[derive(Debug, Serialize)]
        enum E {
            N(Vec<u8>),
        };

        #[derive(Debug, Serialize)]
        struct Test {
            enums: Vec<E>,
        }

        let w = Wrap {
            w: Test {
                enums: vec![E::N(vec![42])],
            },
        };
        let expected = &[("enums[0][N][0]", "42")];

        compare(w, expected);
    }

    #[test]
    fn unit_variant() {
        #[derive(Debug, Serialize)]
        enum E {
            A,
        };

        #[derive(Debug, Serialize)]
        struct Test {
            e: E,
        };
        let w = Wrap {
            w: Test { e: E::A },
        };
        let expected = &[("e", "A")];

        compare(w, expected);
    }

    #[test]
    fn newtype_struct() {
        #[derive(Debug, Serialize)]
        struct S(u8);

        #[derive(Debug, Serialize)]
        struct Test {
            s: S,
        };
        let w = Wrap {
            w: Test { s: S(42) },
        };
        let expected = &[("s", "42")];

        compare(w, expected);
    }

    #[test]
    fn nested_newtype_struct() {
        #[derive(Debug, Serialize)]
        struct S(Vec<u8>);

        #[derive(Debug, Serialize)]
        struct Test {
            s: S,
        };
        let w = Wrap {
            w: Test { s: S(vec![42, 47]) },
        };
        let expected = &[("s[0]", "42"), ("s[1]", "47")];

        compare(w, expected);
    }

    #[test]
    fn struct_variant() {
        #[derive(Debug, Serialize)]
        enum E {
            S { a: u8, b: u8 },
        };
        #[derive(Debug, Serialize)]
        struct Test {
            e: E,
        };

        let w = Wrap {
            w: Test {
                e: E::S { a: 42, b: 47 },
            },
        };
        let expected = &[("e[S][a]", "42"), ("e[S][b]", "47")];

        compare(w, expected);
    }

    #[test]
    fn map() {
        let mut h = HashMap::new();
        h.insert("badger", 42);

        let w = Wrap { w: h };
        let expected = &[("badger", "42")];

        compare(w, expected);
    }

    #[test]
    fn map_seq() {
        let mut h = HashMap::new();
        h.insert("badger", vec![42, 47]);

        let w = Wrap { w: h };
        let expected = &[("badger[0]", "42"), ("badger[1]", "47")];

        compare(w, expected);
    }

    #[test]
    fn nested_map() {
        let mut h = HashMap::new();
        h.insert("badger", vec![42, 47]);

        #[derive(Debug, Serialize)]
        struct Test {
            h: HashMap<&'static str, Vec<u8>>,
        };

        let w = Wrap { w: Test { h } };
        let expected = &[("h[badger][0]", "42"), ("h[badger][1]", "47")];

        compare(w, expected);
    }
}
