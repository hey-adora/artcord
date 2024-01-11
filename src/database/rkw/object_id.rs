use bson::oid::ObjectId;
use bson::DateTime;
use bytecheck::CheckBytes;
use rkyv::{
    ser::Serializer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Archived, Fallible,
};

pub struct OBJ;

#[derive(Debug, CheckBytes)]
#[repr(transparent)]
pub struct ArchivedObjectId(ArchivedString);

impl PartialEq<ObjectId> for ArchivedObjectId {
    fn eq(&self, other: &ObjectId) -> bool {
        self.0 == other.to_string()
    }
}

impl ArchiveWith<ObjectId> for OBJ {
    type Archived = ArchivedObjectId;
    type Resolver = StringResolver;

    unsafe fn resolve_with(
        id: &ObjectId,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        id.to_string().resolve(pos, resolver, out.cast())
    }
}

impl<S: Fallible + Serializer + ?Sized> SerializeWith<ObjectId, S> for OBJ {
    fn serialize_with(id: &ObjectId, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(id.to_string().as_str(), serializer)
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedObjectId, ObjectId, D> for OBJ {
    fn deserialize_with(
        archived: &ArchivedObjectId,
        _deserializer: &mut D,
    ) -> Result<ObjectId, D::Error> {
        Ok(ObjectId::parse_str(archived.0.as_str()).unwrap_or_default())
    }
}
