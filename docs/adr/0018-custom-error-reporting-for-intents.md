# Custom Error Reporting for Intents

- Status: accepted
- Authors: Atif Aziz, Dariusz Parys, Patrick Schuler
- Last updated: 2022-08-30

## Context and Problem Statement

Applications communicate via GRPC through the chariott runtime. GRPC has a
dedicated set of error codes that are used to indicate the reason for the
failure of a GRPC call. In order to provide custom errors for intents as well as
provider specific messages we need a definition on how to propagate those errors
through the chariott runtime.

## Requirements

1. Chariott must support the standard GRPC error codes.
2. Chariott must support custom applications/provider error codes wrapped in the
   GRPC error codes.
3. Chariott will provide pre-defined error codes for common errors.

## Chariott Common Error Codes

Chariott will provide a proto definition with pre-defined error codes. This
serves as a starting point:

```proto
package chariott.v1.error;

message Error {
  enum Code {
    UNKNOWN = 0;
    INTENT_NOT_FOUND = 1;
    INTENT_NOT_SUPPORTED = 2;
    INTENT_NOT_IMPLEMENTED = 3;
    INTENT_NOT_AUTHORIZED = 4;
    INTENT_NOT_SUPPORTED_BY_PROVIDER = 5;
    INTENT_NOT_IMPLEMENTED_BY_PROVIDER = 6;
    INTENT_NOT_AUTHORIZED_BY_PROVIDER = 7;
  }
  Code code = 1;
  string message = 2;
}
```

Applications that want to provide error messages will use the
`chariott.v1.error.Error` message, choose an applicable error code and provide a
descriptive message.

Chariott will only use the `chariott.v1.error.Error` for all errors message
type.

## GRPC / Chariott Error Code Mapping

As chariott is providing a GRPC interface, the GRPC error codes are used to
indicate the reason for the failure of a GRPC call. In order to indicate that
the consuming application should check the `details` field to retrieve more
error information we need a mapping. The GRPC error codes are mapped to the
chariott error codes as follows:

| grpc error code | chariott error code |
|-|-|
| UNKNOWN | UNKNOWN |
| NOT_FOUND | INTENT_NOT_FOUND |
| UNIMPLEMENTED | INTENT_NOT_IMPLEMENTED |
| UNIMPLEMENTED | INTENT_NOT_IMPLEMENTED_BY_PROVIDER |
| UNIMPLEMENTED | INTENT_NOT_SUPPORTED |
| UNIMPLEMENTED | INTENT_NOT_SUPPORTED_BY_PROVIDER |
| PERMISSION_DENIED | INTENT_NOT_AUTHORIZED |
| PERMISSION_DENIED | INTENT_NOT_AUTHORIZED_BY_PROVIDER |

## Additional Thoughts

There are currently [16 error
codes](https://github.com/googleapis/googleapis/blob/master/google/rpc/code.proto)
defined in the GRPC standard.

So any custom error code not known to GRPC needs to be handled differently.
Providing an own custom error information together with the existing error codes
available leads to the selection of either `unknown` or `internal` as the error
code.

GRPC also defines a so called **richer error model**. This model allows to
provide additional information about the error. There is a [dedicated
section](https://grpc.io/docs/guides/error/#richer-error-model) in the GRPC
documentation that describes the richer error model in detail. There is also the
mention of standard set of error message types. The [`RetryInfo` message
type](https://github.com/googleapis/googleapis/blob/4fcb07884d696e90527809f185e8b1f87871245d/google/rpc/error_details.proto#L40-L43)
would be useful for chariott as well.

Currently the richer error model is not supported for rust. There is an [open
issue](https://github.com/hyperium/tonic/issues/1034) in the tonic repository
that describes the problem and there is active development right now to
integrate this.

## Option 1: Custom Error Codes in Metadata

The first option is to use the metadata field of the GRPC response to propagate
the custom error codes. To implement it, it requires only a mapping from the
struct to the metadata field. The following code fragment shows how this can be
done:

```rust
impl From<proto::chariott::v1::error::Error> for Status {
    fn from(error: proto::chariott::v1::error::Error) -> Self {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "x-chariott-error-code",
            AsciiMetadataValue::try_from(error.code).unwrap(),
        );
        metadata.insert(
            "x-chariott-error-message",
            AsciiMetadataValue::try_from(error.message.clone()).unwrap(),
        );
        Status::with_metadata(tonic::Code::Internal, "chariott.v1.error.Error", metadata)
    }
}
```

The drawback is that we have to define custom metadata keys and probably extend
those if the error model is extended.

The keys defined in the example are:

- `x-chariott-error-code`: the error code as defined in the proto definition
- `x-chariott-error-message`: the error message as defined in the proto
  definition

## Option 2: Custom Error Codes in Details

The second option is to use the details field of the GRPC response to propagate
the chariott error codes. In order to inject custom error codes we have to use
the `tonic_types::Status` struct. That struct defines a `details` field that is
a vector of `prost::Any` messages.

The following code fragments shows how this can be done:

1. Instantiate the `Error` message type and set the error code and message.
2. Convert the `Error` message type to a `prost::Any` message.
3. Create a `tonic_types::Status` struct and set the error code and message,
   serialized as bytes.
4. Provide serialized `tonic_types::Status` struct as details for the
   `tonic::Status` `with_details` call.

```rust

// A trait definition to convert a struct to a `prost::Any` message.
trait IntoAny {
    fn into_any(self) -> Any;
}

// The implementation for the specific error message
impl IntoAny for sdv::vtd::errors::Error {
    fn into_any(self) -> Any {
        Any {
            type_url: "sdv.vtd.errors.Error".to_string(),
            value: self.encode_to_vec(),
        }
    }
}

// Helper function to serialize `tonic_types::Status` to bytes, including the chariott error message.
fn gen_details_bytes(code: Code, message: &String, details: Vec<Any>) -> Bytes {
    let status = tonic_types::Status {
        code: code as i32,
        message: message.clone(),
        details,
    };
    Bytes::from(status.encode_to_vec())
}

// Definition of the chariott error message that should be returned in an error case
let intent_not_supported_error = crate::sdv::vtd::errors::Error {
    code: crate::sdv::vtd::errors::error::Code::IntentNotSupported as i32,
    message: "Intent not supported".to_string(),
};

// Convert the chariott error messages to a `tonic_types::Status` struct as serialized bytes.
let details = gen_details_bytes(
    tonic::Code::Unavailable,
    &"Intent not supported errors".to_string(),
    vec![intent_not_supported_error.into_any()],
);

// The standard `tonic::Status` response using the `with_details` call.
let err_status = Status::with_details(
    tonic::Code::Internal,
    "chariott.v1.errors.Error",
    details,
);

Err(err_status)
```

The advantages of this approach are:

1. The error codes are defined in a proto file and can be shared between all
   parties that will work with chariott.
2. Cross-Language support is easier to implement as the error codes are defined
   in a proto file.

It is also important to note that if the Status contains a wrapped
google.rpc.Status, which maps to the `tonic_types::Status` struct, it will be
used as the canonical status code and message, instead of the outer status code
and message.

Usage out in a go client (snippet):

```go
r, err := c.SayHello(ctx, &pb.HelloRequest{Name: "option 2"})
if err != nil {
    log.Printf("err code: %s", err)
    s := status.Convert(err)
    for _, d := range s.Details() {
        switch info := d.(type) {
        case *epb.Error:
            log.Printf("Intent not supported: %s", info)
        default:
            log.Printf("Unexpected type: %s", info)
        }
    }
    os.Exit(1)
}
log.Printf("Greeting: %s", r.Message)
```

and corresponding output

```bash
❯ go run main.go
2022/08/31 14:51:49 err code: rpc error: code = Unavailable desc = Intent not supported errors
2022/08/31 14:51:49 Error: code:2 message:"Intent not supported"
exit status 1
```

## Decision: Option 2 - Custom Error Codes in Details

The decision is to use the details field of the GRPC response to propagate the
custom error codes together with a proto definition of the error. This will also
replace all existing GRPC error codes in the current chariott runtime using this
approach.
