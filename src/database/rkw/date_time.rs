use bson::DateTime;
use bytecheck::CheckBytes;
use rkyv::{
    ser::Serializer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Archived, Fallible,
};
use serde::{Deserialize, Serialize};

pub struct DT;

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    CheckBytes,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[repr(transparent)]
pub struct ArchivedDateTime(pub(crate) Archived<i64>);

impl PartialEq<DateTime> for ArchivedDateTime {
    fn eq(&self, other: &DateTime) -> bool {
        self.0 == other.timestamp_millis()
    }
}

impl ArchiveWith<DateTime> for DT {
    type Archived = ArchivedDateTime;
    type Resolver = ();

    unsafe fn resolve_with(
        datetime: &DateTime,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        datetime
            .timestamp_millis()
            .resolve(pos, resolver, out.cast());
    }
}

impl<S: Fallible + ?Sized> SerializeWith<DateTime, S> for DT {
    fn serialize_with(
        _datetime: &DateTime,
        _serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedDateTime, DateTime, D> for DT {
    fn deserialize_with(
        archived: &ArchivedDateTime,
        _deserializer: &mut D,
    ) -> Result<DateTime, D::Error> {
        Ok(DateTime::from_millis(archived.0))
    }
}
