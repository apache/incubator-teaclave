// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(
    clippy::unused_unit,
    clippy::needless_lifetimes,
    clippy::redundant_closure
)]
// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use yasna::ASN1Result;
use yasna::{BERReader, BERReaderSeq, BERReaderSet};
use yasna::{DERWriter, DERWriterSeq, DERWriterSet};

pub type Writer<'a> = DERWriter<'a>;
pub type Reader<'a, 'b> = BERReader<'a, 'b>;

pub trait ConsWriter<'a> {
    fn next<'b>(&'b mut self) -> Writer<'b>;
}

pub trait ConsReader<'a, 'b>
where
    'a: 'b,
{
    fn next<'c>(&'c mut self, tags: &[yasna::Tag]) -> ASN1Result<Reader<'a, 'c>>;
}

impl<'a> ConsWriter<'a> for DERWriterSeq<'a> {
    fn next<'b>(&'b mut self) -> Writer<'b> {
        self.next()
    }
}

impl<'a> ConsWriter<'a> for DERWriterSet<'a> {
    fn next<'b>(&'b mut self) -> Writer<'b> {
        self.next()
    }
}

impl<'a, 'b> ConsReader<'a, 'b> for BERReaderSeq<'a, 'b> {
    fn next<'c>(&'c mut self, _tags: &[yasna::Tag]) -> ASN1Result<Reader<'a, 'c>> {
        Ok(self.next())
    }
}

impl<'a, 'b> ConsReader<'a, 'b> for BERReaderSet<'a, 'b> {
    fn next<'c>(&'c mut self, tags: &[yasna::Tag]) -> ASN1Result<Reader<'a, 'c>> {
        self.next(tags)
    }
}

pub trait Asn1Ty {
    type ValueTy;
    const TAG: yasna::Tag;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) -> ();
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b;
}

pub trait Asn1ConsTy
where
    Self: std::marker::Sized,
{
    type ValueTy;
    fn dump<'a, W: ConsWriter<'a>>(writer: &mut W, value: Self::ValueTy) -> ();
    fn load<'a, 'b, R>(reader: &mut R) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
        R: ConsReader<'a, 'b>;
}

pub trait Asn1Tag {
    const TAG: yasna::Tag;
}

const fn context_tag(tnum: u64) -> yasna::Tag {
    yasna::Tag {
        tag_class: yasna::TagClass::ContextSpecific,
        tag_number: tnum,
    }
}

mod no_instance {
    #![allow(dead_code)]

    use super::{Asn1ConsTy, Asn1Tag, Asn1Ty};
    use std::marker::PhantomData;

    pub(crate) struct CtxT0;
    pub(crate) struct CtxT1;
    pub(crate) struct CtxT3;

    pub(crate) struct Oid;
    pub(crate) struct U8;
    pub(crate) struct I8;
    pub(crate) struct BigUint;
    pub(crate) struct Utf8Str;
    pub(crate) struct UtcTime;
    pub(crate) struct BitVec;
    pub(crate) struct Bytes;
    pub(crate) struct Tagged<T: Asn1Tag, S: Asn1Ty> {
        t: PhantomData<T>,
        s: PhantomData<S>,
    }
    pub(crate) struct Sequence<U: Asn1Ty, V: Asn1ConsTy> {
        u: PhantomData<U>,
        v: PhantomData<V>,
    }
    pub(crate) struct Set<U: Asn1Ty, V: Asn1ConsTy> {
        u: PhantomData<U>,
        v: PhantomData<V>,
    }
    pub(crate) struct Cons<U: Asn1Ty, V: Asn1ConsTy> {
        u: PhantomData<U>,
        v: PhantomData<V>,
    }
    pub(crate) struct Nil;
}

use no_instance::*;

impl Asn1Tag for CtxT0 {
    const TAG: yasna::Tag = context_tag(0);
}

impl Asn1Tag for CtxT1 {
    const TAG: yasna::Tag = context_tag(1);
}

impl Asn1Tag for CtxT3 {
    const TAG: yasna::Tag = context_tag(3);
}

impl<U: Asn1Ty, V: Asn1ConsTy> Asn1ConsTy for Cons<U, V> {
    type ValueTy = (U::ValueTy, V::ValueTy);
    fn dump<'a, W: ConsWriter<'a>>(writer: &mut W, value: Self::ValueTy) -> () {
        U::dump(writer.next(), value.0);
        V::dump(writer, value.1);
    }
    fn load<'a, 'b, R>(reader: &mut R) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
        R: ConsReader<'a, 'b>,
    {
        let first = U::load(reader.next(&[U::TAG])?)?;
        let second = V::load(reader)?;
        Ok((first, second))
    }
}

impl Asn1ConsTy for Nil {
    type ValueTy = ();
    fn dump<'a, W: ConsWriter<'a>>(_writer: &mut W, _value: Self::ValueTy) -> () {}
    fn load<'a, 'b, R>(_reader: &mut R) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
        R: ConsReader<'a, 'b>,
    {
        Ok(())
    }
}

impl<T: Asn1Tag, S: Asn1Ty> Asn1Ty for Tagged<T, S> {
    type ValueTy = S::ValueTy;
    const TAG: yasna::Tag = T::TAG;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_tagged(T::TAG, |writer| S::dump(writer, value));
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_tagged(T::TAG, |reader| S::load(reader))
    }
}

impl<U: Asn1Ty, V: Asn1ConsTy> Asn1Ty for Sequence<U, V> {
    type ValueTy = (U::ValueTy, V::ValueTy);
    const TAG: yasna::Tag = yasna::tags::TAG_SEQUENCE;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_sequence(|writer| {
            U::dump(writer.next(), value.0);
            V::dump(writer, value.1);
        });
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_sequence(|reader| {
            let first = U::load(reader.next())?;
            let second = V::load(reader)?;
            Ok((first, second))
        })
    }
}

impl<U: Asn1Ty, V: Asn1ConsTy> Asn1Ty for Set<U, V> {
    type ValueTy = (U::ValueTy, V::ValueTy);
    const TAG: yasna::Tag = yasna::tags::TAG_SET;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_set(|writer| {
            U::dump(writer.next(), value.0);
            V::dump(writer, value.1);
        });
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_set(|reader| {
            let first = U::load(reader.next(&[U::TAG])?)?;
            let second = V::load(reader)?;
            Ok((first, second))
        })
    }
}

impl Asn1Ty for U8 {
    type ValueTy = u8;
    const TAG: yasna::Tag = yasna::tags::TAG_INTEGER;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_u8(value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_u8()
    }
}

impl Asn1Ty for I8 {
    type ValueTy = i8;
    const TAG: yasna::Tag = yasna::tags::TAG_INTEGER;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_i8(value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_i8()
    }
}

impl Asn1Ty for BigUint {
    type ValueTy = num_bigint::BigUint;
    const TAG: yasna::Tag = yasna::tags::TAG_INTEGER;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_biguint(&value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_biguint()
    }
}

impl Asn1Ty for Utf8Str {
    type ValueTy = String;
    const TAG: yasna::Tag = yasna::tags::TAG_UTF8STRING;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_utf8_string(value.as_str());
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_utf8string()
    }
}

impl Asn1Ty for Oid {
    type ValueTy = yasna::models::ObjectIdentifier;
    const TAG: yasna::Tag = yasna::tags::TAG_OID;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_oid(&value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_oid()
    }
}

impl Asn1Ty for UtcTime {
    type ValueTy = yasna::models::UTCTime;
    const TAG: yasna::Tag = yasna::tags::TAG_UTCTIME;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_utctime(&value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_utctime()
    }
}

impl Asn1Ty for BitVec {
    type ValueTy = bit_vec::BitVec;
    const TAG: yasna::Tag = yasna::tags::TAG_BITSTRING;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_bitvec(&value);
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_bitvec()
    }
}

impl Asn1Ty for Bytes {
    type ValueTy = Vec<u8>;
    const TAG: yasna::Tag = yasna::tags::TAG_OCTETSTRING;
    fn dump<'a>(writer: Writer<'a>, value: Self::ValueTy) {
        writer.write_bytes(&value.as_slice());
    }
    fn load<'a, 'b>(reader: Reader<'a, 'b>) -> ASN1Result<Self::ValueTy>
    where
        'a: 'b,
    {
        reader.read_bytes()
    }
}

macro_rules! cons {
    () => { Nil };
    ($t: ty) => { Cons<$t, Nil> };
    ($t: ty, $($tt: ty),* $(,)?) => {
        Cons<$t, cons! { $($tt),* }>
    };
}

macro_rules! asn1_seq_ty {
    ($t: ty) => { Sequence<$t, Nil> };
    ($t: ty, $($tt: ty),* $(,)?) => {
        Sequence<$t, cons! { $($tt),* }>
    };
}

macro_rules! asn1_set_ty {
    ($t: ty) => { Set<$t, Nil> };
    ($t: ty, $($tt: ty),* $(,)?) => {
        Set<$t, cons! { $($tt),* }>
    };
}

#[cfg(feature = "mesalock_sgx")]
macro_rules! asn1_seq {
    () => { () };
    ($e: expr) => {
        ($e, ())
    };
    ($e: expr , $($ee: expr),* $(,)?) => {
        ($e, asn1_seq!{ $($ee),* } )
    };
}

pub(crate) type Version = Tagged<CtxT0, I8>;
pub(crate) type Serial = U8;
pub(crate) type CertSignAlgo = asn1_seq_ty!(Oid);
pub(crate) type ValidRange = asn1_seq_ty!(UtcTime, UtcTime);
pub(crate) type Issuer = asn1_seq_ty!(asn1_set_ty!(asn1_seq_ty!(Oid, Utf8Str)));
pub(crate) type Subject = Issuer;
pub(crate) type PubKeyAlgo = asn1_seq_ty!(Oid, Oid);
pub(crate) type PubKey = asn1_seq_ty!(PubKeyAlgo, BitVec);
pub(crate) type SgxRaCertExt = Tagged<CtxT3, asn1_seq_ty!(asn1_seq_ty!(Oid, Bytes))>;
pub(crate) type TbsCert = asn1_seq_ty!(
    Version,
    Serial,
    CertSignAlgo,
    Issuer,
    ValidRange,
    Subject,
    PubKey,
    SgxRaCertExt,
);
pub(crate) type CertSig = BitVec;
pub(crate) type X509 = asn1_seq_ty!(TbsCert, CertSignAlgo, CertSig);
