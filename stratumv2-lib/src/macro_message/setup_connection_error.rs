pub mod macro_prelude {
    pub use crate::common::SetupConnectionErrorCode;
    pub use crate::error::{Error, Result};
    pub use crate::frame::Frameable;
    pub use crate::parse::{ByteParser, Deserializable, Serializable};
    pub use crate::types::{MessageType, STR0_255};
    pub use std::io;
}

/// Implementation of the SetupConnectionError message for each sub protocol.
#[macro_export]
macro_rules! impl_setup_connection_error {
    ($flags_type:ident) => {
        use crate::macro_message::setup_connection_error::macro_prelude::*;

        /// SetupConnectionError is one of the required responses from a Server
        /// to a Client when a new connection has failed. The server is required
        /// to send this message with an error code before closing the connection.
        ///
        /// If the error is a variant of [UnsupportedFeatureFlags](enum.SetupConnectionErrorCode.html),
        /// the server MUST respond with all the feature flags that it does NOT support.
        ///
        /// If the flag is 0, then the error is some condition aside from unsupported
        /// flags.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use stratumv2_lib::mining;
        /// use stratumv2_lib::common::SetupConnectionErrorCode;
        ///
        /// let conn_error = mining::SetupConnectionError::new(
        ///     mining::SetupConnectionFlags::REQUIRES_VERSION_ROLLING,
        ///     SetupConnectionErrorCode::UnsupportedFeatureFlags
        /// );
        ///
        /// assert!(conn_error.is_ok());
        /// assert_eq!(
        ///     conn_error.unwrap().error_code,
        ///     SetupConnectionErrorCode::UnsupportedFeatureFlags
        /// );
        /// ```
        pub struct SetupConnectionError {
            /// Indicates all the flags that the server does NOT support.
            pub flags: $flags_type,

            /// Error code is a predefined STR0_255 error code.
            pub error_code: SetupConnectionErrorCode,
        }

        impl SetupConnectionError {
            /// Constructor for the SetupConnectionError message.
            pub fn new(
                flags: $flags_type,
                error_code: SetupConnectionErrorCode,
            ) -> Result<SetupConnectionError> {
                if flags.is_empty()
                    && error_code == SetupConnectionErrorCode::UnsupportedFeatureFlags
                {
                    return Err(Error::RequirementError(
                        "a full set of unsupported flags MUST be returned to the client".into(),
                    ));
                }

                Ok(SetupConnectionError { flags, error_code })
            }
        }

        impl Serializable for SetupConnectionError {
            fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<usize> {
                let length =
                    self.flags.bits().serialize(writer)? + self.error_code.serialize(writer)?;

                Ok(length)
            }
        }

        impl Deserializable for SetupConnectionError {
            fn deserialize(parser: &mut ByteParser) -> Result<SetupConnectionError> {
                let flags = $flags_type::from_bits(u32::deserialize(parser)?)
                    .ok_or(Error::UnknownFlags())?;
                let error_code = SetupConnectionErrorCode::deserialize(parser)?;

                Ok(SetupConnectionError {
                    flags: flags,
                    error_code: error_code,
                })
            }
        }

        impl Frameable for SetupConnectionError {
            fn message_type() -> MessageType {
                MessageType::SetupConnectionError
            }
        }
    };
}
