//! WebDAV status codes and Depth header representations.

/// A resource status code shared across DAV operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    MultiStatus = 207,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    Conflict = 409,
    PreconditionFailed = 412,
    Locked = 423,
    FailedDependency = 424,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
}

/// The `Depth` header value used to scope a PROPFIND or collection operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Zero,
    One,
    Infinity,
}
