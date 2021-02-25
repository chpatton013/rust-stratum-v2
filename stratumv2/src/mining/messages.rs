use crate::common::SetupConnectionErrorCodes;
use crate::error::{Error, Result};
use crate::mining::{SetupConnectionFlags, SetupConnectionSuccessFlags};
use crate::types::{MessageTypes, STR0_255, U256};
use crate::{BitFlag, Deserializable, Framable, Protocol, Serializable};
use std::borrow::Cow;
use std::{io, str};

// Implementation of the SetupConenction message for the Mining Protocol.
impl_setup_connection!(Protocol::Mining, SetupConnectionFlags);

// Implementation of the SetupConnectionSuccess message for the Mining Protocol.
impl_setup_connection_success!(SetupConnectionSuccessFlags);

// Implementation of the SetupConnectionError message for the Mining Protocol.
impl_setup_connection_error!(SetupConnectionFlags);

/// OpenStandardMiningChannel is a message sent by the client to the server
/// after a [SetupConnection.Success](struct.SetupConnectionSuccess.html) is
/// sent by the server. This message is used to request opening a standard
/// channel to the upstream server. A standard mining channel indicates `header-only`
/// mining.
pub struct OpenStandardMiningChannel {
    pub request_id: u32,
    pub user_identity: String,
    pub nominal_hash_rate: f32,
    pub max_target: U256,
}

impl OpenStandardMiningChannel {
    fn new(
        request_id: u32,
        user_identity: String,
        nominal_hash_rate: f32,
        max_target: U256,
    ) -> OpenStandardMiningChannel {
        OpenStandardMiningChannel {
            request_id,
            user_identity,
            nominal_hash_rate,
            max_target,
        }
    }
}

#[cfg(test)]
mod setup_connection_tests {
    use super::*;
    use crate::util::{frame, serialize};

    #[test]
    fn init_setup_connection() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        );

        assert!(message.is_ok());
    }

    #[test]
    fn setup_connection_invalid_min_value() {
        let message = SetupConnection::new(
            1,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        );

        assert!(message.is_err());
    }

    #[test]
    fn setup_connection_invalid_max_value() {
        let message = SetupConnection::new(
            2,
            0,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        );
        assert!(message.is_err());
    }

    #[test]
    fn setup_connection_empty_vendor() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        );

        assert!(message.is_err())
    }

    #[test]
    fn setup_connection_empty_firmware() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "",
            "some-uuid",
        );

        assert!(message.is_err())
    }

    #[test]
    fn serialize_setup_connection() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        )
        .unwrap();

        let buffer = serialize(message).unwrap();
        assert_eq!(buffer.len(), 75);

        let expected = [
            0x00, // protocol
            0x02, 0x00, // min_version
            0x02, 0x00, // max_version
            0x01, 0x00, 0x00, 0x00, // flags
            0x07, // length_endpoint_host
            0x30, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x30, // endpoint_host
            0x61, 0x21, // endpoint_port
            0x07, // length_vendor
            0x42, 0x69, 0x74, 0x6d, 0x61, 0x69, 0x6e, // vendor
            0x08, // length_hardware_version
            0x53, 0x39, 0x69, 0x20, 0x31, 0x33, 0x2e, 0x35, // hardware_version
            0x1c, // length_firmware
            0x62, 0x72, 0x61, 0x69, 0x69, 0x6e, 0x73, 0x2d, 0x6f, 0x73, 0x2d, 0x32, 0x30, 0x31,
            0x38, 0x2d, 0x30, 0x39, 0x2d, 0x32, 0x32, 0x2d, 0x31, 0x2d, 0x68, 0x61, 0x73,
            0x68, // firmware
            0x09, // length_device_id
            0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x75, 0x75, 0x69, 0x64, // device_id
        ];
        assert_eq!(buffer, expected);

        // Sanity check - deserializing the struct does not return errors.
        assert!(SetupConnection::deserialize(&buffer).is_ok());
    }

    #[test]
    fn serialize_no_flags() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        )
        .unwrap();

        let buffer = serialize(message).unwrap();
        assert_eq!(buffer.len(), 75);

        // Expect the feature flag to have no set flags (0x00).
        assert_eq!(buffer[5], 0x00);
    }

    #[test]
    fn serialize_multiple_flags() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[
                SetupConnectionFlags::RequiresStandardJobs,
                SetupConnectionFlags::RequiresVersionRolling,
            ]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        )
        .unwrap();

        let buffer = serialize(message).unwrap();

        assert_eq!(buffer.len(), 75);
        assert_eq!(buffer[5], 0x05);
    }

    #[test]
    fn serialilze_all_flags() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[
                SetupConnectionFlags::RequiresStandardJobs,
                SetupConnectionFlags::RequiresWorkSelection,
                SetupConnectionFlags::RequiresVersionRolling,
            ]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        )
        .unwrap();

        let buffer = serialize(message).unwrap();

        assert_eq!(buffer.len(), 75);
        assert_eq!(buffer[5], 0x07);
    }

    #[test]
    fn deserialize_setup_connection() {
        let input = [
            0x00, // protocol
            0x02, 0x00, // min_version
            0x02, 0x00, // max_version
            0x01, 0x00, 0x00, 0x00, // flags
            0x07, // length_endpoint_host
            0x30, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x30, // endpoint_host
            0x61, 0x21, // endpoint_port
            0x07, // length_vendor
            0x42, 0x69, 0x74, 0x6d, 0x61, 0x69, 0x6e, // vendor
            0x08, // length_hardware_version
            0x53, 0x39, 0x69, 0x20, 0x31, 0x33, 0x2e, 0x35, // hardware_version
            0x1c, // length_firmware
            0x62, 0x72, 0x61, 0x69, 0x69, 0x6e, 0x73, 0x2d, 0x6f, 0x73, 0x2d, 0x32, 0x30, 0x31,
            0x38, 0x2d, 0x30, 0x39, 0x2d, 0x32, 0x32, 0x2d, 0x31, 0x2d, 0x68, 0x61, 0x73,
            0x68, // firmware
            0x09, // length_device_id
            0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x75, 0x75, 0x69, 0x64, // device_id
        ];

        let message = SetupConnection::deserialize(&input).unwrap();
        assert_eq!(message.min_version, 2);
        assert_eq!(message.max_version, 2);
        assert_eq!(message.flags[0], SetupConnectionFlags::RequiresStandardJobs);
        assert_eq!(message.endpoint_host, "0.0.0.0".to_string());
        assert_eq!(message.endpoint_port, 8545);
        assert_eq!(message.vendor, "Bitmain".to_string());
        assert_eq!(message.hardware_version, "S9i 13.5".to_string());
        assert_eq!(message.firmware, "braiins-os-2018-09-22-1-hash".to_string());
        assert_eq!(message.device_id, "some-uuid".to_string());
    }

    #[test]
    fn deserialize_malformed_setup_connection() {
        // Empty message.
        let input = [];
        assert!(SetupConnection::deserialize(&input).is_err());

        // Unknown protocol.
        let input = [0xAF];
        assert!(SetupConnection::deserialize(&input).is_err());

        let input = [
            0x00, // protocol
            0x02, 0x00, // min_version
            0x02, 0x00, // max_version
            0x01, 0x00, 0x00, 0x00, // flags
            0x07, // length_endpoint_host
            0x30, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x30, // endpoint_host
            0x61, 0x21, // endpoint_port
            0x07, // length_vendor
            0x42, 0x69, 0x74, 0x6d, 0x61, 0x69, 0x6e, // vendor
            0x08, // length_hardware_version
            0x53, 0x39, 0x69, 0x20, 0x31, 0x33, 0x2e, 0x35, // hardware_version
            0x1c, // length_firmware
            0x62, 0x72, 0x61, 0x69, 0x69, 0x6e, 0x73, 0x2d, 0x6f, 0x73, 0x2d, 0x32, 0x30, 0x31,
            0x38, 0x2d, 0x30, 0x39, 0x2d, 0x32, 0x32, 0x2d, 0x31, 0x2d, 0x68, 0x61, 0x73,
            0x68, // firmware
            0x09, // length_device_id
            0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x75, 0x75, 0x69, 0x64, // device_id
        ];

        // Append each byte from input to a new vector of bytes. This should
        // return errors each time on deserialization since the message is
        // malformed because the message is incomplete.
        let mut output = vec![];
        for i in input.iter() {
            assert!(SetupConnection::deserialize(&output).is_err());
            output.push(*i);
        }

        // Now that the vector of bytes contains the full message, deserializing
        // should return ok.
        assert!(SetupConnection::deserialize(&output).is_ok());
    }

    #[test]
    fn frame_setup_connection() {
        let message = SetupConnection::new(
            2,
            2,
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            "0.0.0.0",
            8545,
            "Bitmain",
            "S9i 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-uuid",
        )
        .unwrap();

        let buffer = frame(message).unwrap();
        assert_eq!(buffer.len(), 81);

        let expected = [
            0x00, 0x00, // extension_type
            0x00, // msg_type
            0x4b, 0x00, 0x00, // msg_length
            0x00, // protocol
            0x02, 0x00, // min_version
            0x02, 0x00, // max_version
            0x01, 0x00, 0x00, 0x00, // flags
            0x07, // length_endpoint_host
            0x30, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x30, // endpoint_host
            0x61, 0x21, // endpoint_port
            0x07, // length_vendor
            0x42, 0x69, 0x74, 0x6d, 0x61, 0x69, 0x6e, // vendor
            0x08, // length_hardware_version
            0x53, 0x39, 0x69, 0x20, 0x31, 0x33, 0x2e, 0x35, // hardware_version
            0x1c, // length_firmeware
            0x62, 0x72, 0x61, 0x69, 0x69, 0x6e, 0x73, 0x2d, 0x6f, 0x73, 0x2d, 0x32, 0x30, 0x31,
            0x38, 0x2d, 0x30, 0x39, 0x2d, 0x32, 0x32, 0x2d, 0x31, 0x2d, 0x68, 0x61, 0x73,
            0x68, // firmeware
            0x09, // length_device_id
            0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x75, 0x75, 0x69, 0x64, // device_id
        ];
        assert_eq!(buffer, expected);
    }
}

#[cfg(test)]
mod open_standard_mining_tests {
    use super::*;

    #[test]
    fn new_open_standard_mining_channel_0() {
        // TODO: Substitute the nominal hash rate with something more accurate.
        let target = [0u8; 32];

        let message =
            OpenStandardMiningChannel::new(1, "braiinstest.worker1".to_string(), 12.3, target);

        assert_eq!(message.request_id, 1);
        assert_eq!(message.user_identity, "braiinstest.worker1");
        assert_eq!(message.nominal_hash_rate, 12.3);
        assert_eq!(message.max_target.len(), 32);
    }
}

#[cfg(test)]
mod connection_success_tests {
    use super::*;
    use crate::util::{frame, serialize};

    #[test]
    fn serialize_connection_success() {
        let message = SetupConnectionSuccess::new(2, Cow::Borrowed(&[]));

        let buffer = serialize(message).unwrap();

        let expected = [
            0x02, 0x00, // used_version
            0x00, 0x00, 0x00, 0x00, // flags
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn serialize_connection_sucess() {
        let message = SetupConnectionSuccess::new(
            2,
            Cow::Borrowed(&[SetupConnectionSuccessFlags::RequiresFixedVersion]),
        );

        let buffer = serialize(message).unwrap();

        let expected = [
            0x02, 0x00, // used_version
            0x01, 0x00, 0x00, 0x00, // flags
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn frame_connection_success() {
        let message = SetupConnectionSuccess::new(
            2,
            Cow::Borrowed(&[SetupConnectionSuccessFlags::RequiresFixedVersion]),
        );

        let buffer = frame(message).unwrap();

        let expected = [
            0x00, 0x00, // extension_type
            0x01, // msg_type
            0x06, 0x00, 0x00, // msg_length
            0x02, 0x00, // used_version
            0x01, 0x00, 0x00, 0x00, // flags
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn serialize_connection_success_all_flags() {
        let message = SetupConnectionSuccess::new(
            2,
            Cow::Borrowed(&[
                SetupConnectionSuccessFlags::RequiresFixedVersion,
                SetupConnectionSuccessFlags::RequiresExtendedChannels,
            ]),
        );

        let buffer = serialize(message).unwrap();

        let expected = [
            0x02, 0x00, // used_version
            0x03, 0x00, 0x00, 0x00, // flags
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn serialize_connection_success_no_flags() {
        let message = SetupConnectionSuccess::new(2, Cow::Borrowed(&[]));

        let buffer = serialize(message).unwrap();

        let expected = [
            0x02, 0x00, // used_version
            0x00, 0x00, 0x00, 0x00, // flags
        ];
        assert_eq!(buffer, expected);
    }
}

#[cfg(test)]
mod connection_error_tests {
    use super::*;
    use crate::util::{frame, serialize};

    #[test]
    fn serialize_connection_error() {
        let message = SetupConnectionError::new(
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            SetupConnectionErrorCodes::UnsupportedFeatureFlags,
        )
        .unwrap();

        let buffer = serialize(message).unwrap();

        // Feature flag.
        assert_eq!(buffer[0], 0x01);

        // Length of error code string.
        assert_eq!(buffer[4], 0x19);
    }

    #[test]
    fn serialize_connection_error_empty_flags() {
        let message = SetupConnectionError::new(
            Cow::Borrowed(&[]),
            SetupConnectionErrorCodes::UnsupportedFeatureFlags,
        );

        assert!(message.is_err())
    }

    #[test]
    fn frame_connection_error() {
        let message = SetupConnectionError::new(
            Cow::Borrowed(&[SetupConnectionFlags::RequiresStandardJobs]),
            SetupConnectionErrorCodes::UnsupportedFeatureFlags,
        )
        .unwrap();

        let buffer = frame(message).unwrap();

        let expected = [
            0x00, 0x00, // extension_type
            0x03, // msg_type
            0x1e, 0x00, 0x00, // msg_length
            0x01, 0x00, 0x00, 0x00, // flags
            0x19, // length_error_code
            0x75, 0x6e, 0x73, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0x61, 0x74, 0x75, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];

        assert_eq!(buffer, expected)
    }

    #[test]
    fn deserialize_connection_error() {
        let message = [
            0x01, 0x00, 0x00, 0x00, // flags
            0x19, // length_error_code
            0x75, 0x6e, 0x73, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0x61, 0x74, 0x75, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];

        let conn_error = SetupConnectionError::deserialize(&message).unwrap();
        assert_eq!(
            conn_error.flags[0],
            SetupConnectionFlags::RequiresStandardJobs
        );
        assert_eq!(
            conn_error.error_code,
            SetupConnectionErrorCodes::UnsupportedFeatureFlags
        );
    }

    #[test]
    fn deserialize_malformed_connection_error() {
        // Empty message.
        let input = [];
        assert!(SetupConnection::deserialize(&input).is_err());

        let input = [
            0x01, 0x00, 0x00, 0x00, // flags
            0x19, // length_error_code
            0x75, 0x6e, 0x73, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0x61, 0x74, 0x75, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];

        let mut output = vec![];
        for i in input.iter() {
            assert!(SetupConnectionError::deserialize(&output).is_err());
            output.push(*i);
        }

        assert!(SetupConnectionError::deserialize(&output).is_ok());

        // Incorrect length_error_code.
        let input = [
            0x01, 0x00, 0x00, 0x00, // flags
            0xFF, // length_error_code
            0x75, 0x6e, 0x73, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0x61, 0x74, 0x75, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];
        assert!(SetupConnectionError::deserialize(&input).is_err());

        // Invalid flags.
        let input = [
            0xff, 0xff, 0xff, 0xff, // flags
            0xff, // length_error_code
            0x75, 0x6e, 0x73, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0x61, 0x74, 0x75, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];
        assert!(SetupConnectionError::deserialize(&input).is_err());

        // Invalid error code.
        let input = [
            0x01, 0x00, 0x00, 0x00, // flags
            0x19, // length_error_code
            0xff, 0xff, 0xff, 0x75, 0x70, 0x70, 0x6f, 0x72, 0x74, 0x65, 0x64, 0x2d, 0x66, 0x65,
            0xff, 0xff, 0xff, 0x72, 0x65, 0x2d, 0x66, 0x6c, 0x61, 0x67, 0x73, // error_code
        ];
        assert!(SetupConnectionError::deserialize(&input).is_err());
    }
}
