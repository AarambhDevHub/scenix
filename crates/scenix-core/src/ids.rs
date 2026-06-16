use core::fmt;

macro_rules! id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name(u64);

        impl $name {
            /// Creates an ID from a raw value.
            #[inline]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the raw ID value.
            #[inline]
            pub const fn get(self) -> u64 {
                self.0
            }

            /// Returns whether this ID is the default zero sentinel.
            #[inline]
            pub const fn is_null(self) -> bool {
                self.0 == 0
            }
        }

        impl fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl From<u64> for $name {
            #[inline]
            fn from(value: u64) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for u64 {
            #[inline]
            fn from(value: $name) -> Self {
                value.get()
            }
        }
    };
}

id_type!(NodeId, "Typed identifier for a scene node.");
id_type!(MeshId, "Typed identifier for a mesh resource.");
id_type!(MaterialId, "Typed identifier for a material resource.");
id_type!(TextureId, "Typed identifier for a texture resource.");
id_type!(LightId, "Typed identifier for a light resource.");
id_type!(CameraId, "Typed identifier for a camera resource.");
id_type!(AssetId, "Typed identifier for an imported asset package.");
id_type!(SkinId, "Typed identifier for imported skinning metadata.");
id_type!(
    AnimationClipId,
    "Typed identifier for an imported animation clip."
);

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn id_newtypes_are_copy_hashable_and_debuggable() {
        let id = NodeId::new(42);
        let copied = id;
        assert_eq!(copied.get(), 42);
        assert_eq!(format!("{id:?}"), "NodeId(42)");
    }
}
