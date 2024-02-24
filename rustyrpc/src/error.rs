#![allow(trivial_bounds)]
use core::num::TryFromIntError;

use thiserror::Error;

use crate::{
    format::{Decode, Encode, EncodingFormat},
    protocol::{
        RemoteServiceIdRequestError, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceIdRequestResult,
    },
    transport,
};

use derive_where::derive_where;

/// Errors that may occur during service id request.
pub type ServiceRequestError<'a, Connection, Format> =
    ServiceRequestErrorGeneric<Connection, Format, <RequestKind<'a> as Encode<Format>>::Error>;

/// Errors that may occur during service id request.
#[derive_where(Debug)]
#[derive(Error)]
pub enum ServiceRequestErrorGeneric<Connection, Format, EncodingError>
where
    Connection: transport::Connection,
    Format: EncodingFormat,
    EncodingError: std::error::Error,
    ServiceIdRequestResult: Decode<Format>,
{
    /// Indicates a failure in opening or closing the stream.
    #[error("Failed to open/close stream")]
    StreamManagement(#[source] Connection::Error),
    /// Indicates a failure in sending or receiving the stream's message.
    #[error("Failed to send/receive stream's message")]
    StreamIO(#[source] <Connection::Stream as transport::Stream>::Error),
    /// Indicates a failure in encoding the request.
    #[error("Failed to encode request")]
    RequestEncode(#[source] SendEncodableError<Connection::Stream, EncodingError>),
    /// Indicates a failure in decoding the response.
    #[error("Failed to decode response")]
    ResponseDecode(
        #[from]
        ReceiveDecodableError<
            Connection::Stream,
            <ServiceIdRequestResult as Decode<Format>>::Error,
        >,
    ),
    /// Indicates a failure in casting received service id to usize.
    #[error(transparent)]
    InvalidServiceId(#[from] TryFromIntError),
    /// Indicates a failure while handling request on remote host.
    #[error(transparent)]
    Remote(#[from] RemoteServiceIdRequestError),
}

/// Errors that may occur during service call.
#[derive_where(Debug)]
#[derive(Error)]
pub enum ServiceCallError<Connection, Format, Args, Returns>
where
    Connection: transport::Connection,
    Format: EncodingFormat,
    RequestKind<'static>: Encode<Format>,
    Args: Encode<Format>,
    Returns: Decode<Format>,
    ServiceCallRequestResult: Decode<Format>,
{
    /// Indicates a failure in opening or closing the stream.
    #[error("Failed to open/close stream")]
    StreamManagement(#[source] Connection::Error),
    /// Indicates a failure in sending or receiving the stream's message.
    #[error("Failed to send/receive stream's message")]
    StreamIO(#[source] <Connection::Stream as transport::Stream>::Error),
    /// Indicates a failure in encoding the request.
    #[error("Failed to encode request")]
    RequestEncode(
        #[source]
        SendEncodableError<Connection::Stream, <RequestKind<'static> as Encode<Format>>::Error>,
    ),
    /// Indicates a failure in encoding the function arguments.
    #[error("Failed to encode function arguments")]
    ArgsEncode(#[source] SendEncodableError<Connection::Stream, <Args as Encode<Format>>::Error>),
    /// Indicates a failure in decoding the response.
    #[error("Failed to decode response")]
    ResponseDecode(
        #[source]
        ReceiveDecodableError<
            Connection::Stream,
            <ServiceCallRequestResult as Decode<Format>>::Error,
        >,
    ),
    /// Indicates a failure in decoding the value returned by the function.
    #[error("Failed to decode value returned by function")]
    ReturnsDecode(
        #[from] ReceiveDecodableError<Connection::Stream, <Returns as Decode<Format>>::Error>,
    ),
    /// Indicates a failure while handling request on remote host.
    #[error(transparent)]
    Remote(#[from] ServiceCallRequestError),
}

/// Errors that may occur while sending data structure that implements [`Encode`]
#[derive_where(Debug)]
#[derive(Error)]
pub enum SendEncodableError<Stream: transport::Stream, EncodingError: std::error::Error> {
    /// Indicates a failure in encoding the message.
    #[error("Failed to encode message")]
    Encode(#[source] EncodingError),
    /// Indicates a failure in sending the message.
    #[error("Failed to send message")]
    Send(#[source] Stream::Error),
}

/// Errors that may occur while receiving data structure that implements [`Decode`]
#[derive_where(Debug)]
#[derive(Error)]
pub enum ReceiveDecodableError<Stream: transport::Stream, DecodeError: std::error::Error> {
    /// Indicates a failure in decoding the message.
    #[error("Failed to decode message")]
    Decode(#[source] DecodeError),
    /// Indicates a failure in receiving the message.
    #[error("Failed to receive message")]
    Receive(#[source] Stream::Error),
}
