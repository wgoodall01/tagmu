pub trait Id:
    From<u64> + Into<u64> + Into<[u8; 8]> + From<[u8; 8]> + std::fmt::Display + Into<sled::IVec>
{
}

macro_rules! generate_id {
    ($id:ident) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id(u64);

        impl $id {
            fn to_bytes(&self) -> [u8; 8] {
                self.0.to_be_bytes()
            }
        }

        impl crate::id::Id for $id {}

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u64> for $id {
            fn from(id: u64) -> $id {
                $id(id)
            }
        }

        impl std::convert::TryFrom<&[u8]> for $id {
            type Error = std::array::TryFromSliceError;

            fn try_from(value: &[u8]) -> Result<$id, Self::Error> {
                use std::convert::TryInto;

                let bytes: [u8; 8] = value.try_into()?;
                Ok(bytes.into())
            }
        }

        impl std::convert::TryFrom<sled::IVec> for $id {
            type Error = std::array::TryFromSliceError;

            fn try_from(ivec: sled::IVec) -> Result<$id, Self::Error> {
                let bytes: &[u8] = &ivec;
                $id::try_from(bytes)
            }
        }

        impl Into<u64> for $id {
            fn into(self) -> u64 {
                self.0
            }
        }

        /// Convert the ID to a big-endian slice of u8
        impl Into<[u8; 8]> for $id {
            fn into(self) -> [u8; 8] {
                self.to_bytes()
            }
        }

        /// Convert 8 big-endian bytes into the ID
        impl From<[u8; 8]> for $id {
            fn from(bytes: [u8; 8]) -> $id {
                u64::from_be_bytes(bytes).into()
            }
        }

        /// Convert to a Sled IVec, to use in the KV index.
        impl Into<sled::IVec> for $id {
            fn into(self) -> sled::IVec {
                let bytes: [u8; 8] = self.into();
                (&bytes).into()
            }
        }

        impl AsMut<u64> for $id {
            fn as_mut(&mut self) -> &mut u64 {
                &mut self.0
            }
        }
    };
}
