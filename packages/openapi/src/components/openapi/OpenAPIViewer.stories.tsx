import petStore from '@readme/oas-examples/3.0/json/petstore.json';
import type { Meta, StoryObj } from '@storybook/react';

import { OpenAPIViewer } from './OpenAPIViewer';

const meta: Meta<typeof OpenAPIViewer> = {
  component: OpenAPIViewer,
};

export default meta;

type Story = StoryObj<typeof OpenAPIViewer>;

export const PetStore: Story = {
  args: { schema: petStore },
};

const SCHEMA = `\
swagger: "2.0"
info:
  title: api/scdb_api.proto
  version: version not set
tags:
  - name: SCDBService
    description: |
      SCDBService provides a collection of APIs, that client-user can connect to the SCQL system, execute queries and fetch results.
  - name: SCDBQueryResultCallback
consumes:
  - application/json
produces:
  - application/json
paths:
  /public/submit_query:
    post:
      summary: Submit a query to SCQL
      description: |
        Submit the query (DDL/DCL/DQL) to SCQL, and return immediately. A new
        scdb_session_id will be allocated for the query and returned in the
        response.

        You can use the scdb_session_id to fetch the query result later
        using {ref}scql.pb.SCDBService.Fetch.

        ---

        SCQL supports **two** types of APIs: synchronous and asynchronous. The
        synchronous interface is suitable for executing fast queries, such as
        DDL, DCL, and simple DQL. Meanwhile, the asynchronous interface is
        recommended when the query may take a long time to run.
      operationId: SCDBService_Submit
      responses:
        "200":
          description: A successful response.
          schema:
            $ref: '#/definitions/scql.pb.SCDBSubmitResponse'
        default:
          description: An unexpected error response.
          schema:
            $ref: '#/definitions/google.rpc.Status'
      parameters:
        - name: body
          description: |
            SCDBQueryRequest designed for Client(Biz Service) which allow callback url and traceid
          in: body
          required: true
          schema:
            $ref: '#/definitions/scql.pb.SCDBQueryRequest'
      tags:
        - 'stability: stable'
      externalDocs:
        description: Guide
        url: https://github.com/secretflow/scql/blob/main/docs/intro/tutorial.rst
      x-secretflow-examples:
        - description: |
            This example shows how to submit a query to SCQL.
          parameters:
            body:
              query: show databases;
              user:
                user:
                  nativeUser:
                    name: test
                    password: test
          response:
            "200":
              scdb_session_id: some_session_id
              status:
                code: 0
                details: []
                message: ""
          summary: An example submit request
  /public/fetch_result:
    post:
      summary: Fetch the result of the query submitted asynchronously.
      description: It will return NOT_READY status code if the query is still running.
      operationId: SCDBService_Fetch
      responses:
        "200":
          description: A successful response.
          schema:
            $ref: '#/definitions/scql.pb.SCDBQueryResultResponse'
        default:
          description: An unexpected error response.
          schema:
            $ref: '#/definitions/google.rpc.Status'
      parameters:
        - name: body
          in: body
          required: true
          schema:
            $ref: '#/definitions/scql.pb.SCDBFetchRequest'
      tags:
        - SCDBService
  /public/submit_and_get:
    post:
      summary: |-
        The synchronous query interface allows users to submit a query,
        wait for it to finish, and get the query result in one RPC.
        This interface is suitable for executing fast queries,
        such as DDL, DCL, and simple DQL. However,
        if the query takes a long time to run, it may result in a timeout.
        Therefore, it is recommended to use the synchronous query API to run
        complex queries.
      operationId: SCDBService_SubmitAndGet
      responses:
        "200":
          description: A successful response.
          schema:
            $ref: '#/definitions/scql.pb.SCDBQueryResultResponse'
        default:
          description: An unexpected error response.
          schema:
            $ref: '#/definitions/google.rpc.Status'
      parameters:
        - name: body
          description: |
            SCDBQueryRequest designed for Client(Biz Service) which allow callback url and traceid
          in: body
          required: true
          schema:
            $ref: '#/definitions/scql.pb.SCDBQueryRequest'
      tags:
        - SCDBService
definitions:
  google.protobuf.Any:
    type: object
    properties:
      '@type':
        type: string
        description: |-
          A URL/resource name that uniquely identifies the type of the serialized
          protocol buffer message. This string must contain at least
          one "/" character. The last segment of the URL's path must represent
          the fully qualified name of the type (as in
          path/google.protobuf.Duration). The name should be in a canonical form
          (e.g., leading "." is not accepted).

          In practice, teams usually precompile into the binary all types that they
          expect it to use in the context of Any. However, for URLs which use the
          scheme http, https, or no scheme, one can optionally set up a type
          server that maps type URLs to message definitions as follows:

          * If no scheme is provided, https is assumed.
          * An HTTP GET on the URL must yield a [google.protobuf.Type][]
            value in binary format, or produce an error.
          * Applications are allowed to cache lookup results based on the
            URL, or have them precompiled into a binary to avoid any
            lookup. Therefore, binary compatibility needs to be preserved
            on changes to types. (Use versioned type names to manage
            breaking changes.)

          Note: this functionality is not currently available in the official
          protobuf release, and it is not used for type URLs beginning with
          type.googleapis.com. As of May 2023, there are no widely used type server
          implementations and no plans to implement one.

          Schemes other than http, https (or the empty scheme) might be
          used with implementation specific semantics.
    additionalProperties: {}
    description: |-
      Any contains an arbitrary serialized protocol buffer message along with a
      URL that describes the type of the serialized message.

      Protobuf library provides support to pack/unpack Any values in the form
      of utility functions or additional generated methods of the Any type.

      Example 1: Pack and unpack a message in C++.

          Foo foo = ...;
          Any any;
          any.PackFrom(foo);
          ...
          if (any.UnpackTo(&foo)) {
            ...
          }

      Example 2: Pack and unpack a message in Java.

          Foo foo = ...;
          Any any = Any.pack(foo);
          ...
          if (any.is(Foo.class)) {
            foo = any.unpack(Foo.class);
          }
          // or ...
          if (any.isSameTypeAs(Foo.getDefaultInstance())) {
            foo = any.unpack(Foo.getDefaultInstance());
          }

       Example 3: Pack and unpack a message in Python.

          foo = Foo(...)
          any = Any()
          any.Pack(foo)
          ...
          if any.Is(Foo.DESCRIPTOR):
            any.Unpack(foo)
            ...

       Example 4: Pack and unpack a message in Go

           foo := &pb.Foo{...}
           any, err := anypb.New(foo)
           if err != nil {
             ...
           }
           ...
           foo := &pb.Foo{}
           if err := any.UnmarshalTo(foo); err != nil {
             ...
           }

      The pack methods provided by protobuf library will by default use
      'type.googleapis.com/full.type.name' as the type URL and the unpack
      methods only use the fully qualified type name after the last '/'
      in the type URL, for example "foo.bar.com/x/y.z" will yield type
      name "y.z".

      JSON
      ====
      The JSON representation of an Any value uses the regular
      representation of the deserialized, embedded message, with an
      additional field @type which contains the type URL. Example:

          package google.profile;
          message Person {
            string first_name = 1;
            string last_name = 2;
          }

          {
            "@type": "type.googleapis.com/google.profile.Person",
            "firstName": <string>,
            "lastName": <string>
          }

      If the embedded message type is well-known and has a custom JSON
      representation, that representation will be embedded adding a field
      value which holds the custom JSON in addition to the @type
      field. Example (for message [google.protobuf.Duration][]):

          {
            "@type": "type.googleapis.com/google.protobuf.Duration",
            "value": "1.212s"
          }
  google.rpc.Status:
    type: object
    properties:
      code:
        type: integer
        format: int32
      message:
        type: string
      details:
        type: array
        items:
          type: object
          $ref: '#/definitions/google.protobuf.Any'
  scql.pb.PrimitiveDataType:
    type: string
    enum:
      - PrimitiveDataType_UNDEFINED
      - INT8
      - INT16
      - INT32
      - INT64
      - FLOAT32
      - FLOAT64
      - BOOL
      - STRING
      - DATETIME
      - TIMESTAMP
    default: PrimitiveDataType_UNDEFINED
    description: |-
      the 8-bit signed integer type
       - INT16: the 16-bit signed integer type
       - INT32: the 32-bit signed integer type
       - INT64: the 64-bit signed integer type
       - FLOAT32: the 32-bit binary floating point type
       - FLOAT64: the 64-bit binary floating point type
       - BOOL: Other types
       - DATETIME: DATETIME and TIMESTAMP

      https://dev.mysql.com/doc/refman/8.0/en/datetime.html
       - TIMESTAMP: seconds since '1970-01-01 00:00:00' UTC
    title: '- INT8: Numeric types'
  scql.pb.RequestHeader:
    type: object
    properties:
      custom_headers:
        type: object
        additionalProperties:
          type: string
        description: Custom headers used to record custom information.
    description: RequestHeader carries the user custom headers.
  scql.pb.SCDBCredential:
    type: object
    properties:
      user:
        $ref: '#/definitions/scql.pb.User'
  scql.pb.SCDBFetchRequest:
    type: object
    properties:
      header:
        $ref: '#/definitions/scql.pb.RequestHeader'
      user:
        $ref: '#/definitions/scql.pb.SCDBCredential'
      scdb_session_id:
        type: string
        title: Scdb session id
  scql.pb.SCDBQueryRequest:
    type: object
    example:
      query: show databases;
      user:
        user:
          nativeUser:
            name: test
            password: test
    properties:
      header:
        $ref: '#/definitions/scql.pb.RequestHeader'
        title: The request header
      user:
        $ref: '#/definitions/scql.pb.SCDBCredential'
      query:
        type: string
        description: SCQL query to be run.
      query_result_callback_url:
        type: string
        description: |-
          Optional call back URL to report query result.
          If provided, it should implements the
          SCDBQueryResultCallback.ReportQueryResult method.
      biz_request_id:
        type: string
        description: |
          trace_id is used to trace the request and response of the query process. It is recommended to use UUID as the trace_id.
        pattern: ^[a-f0-9]{8}-[a-f0-9]{4}-[1-5][a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$
        x-secretflow-examples:
          - summary: An example trace_id
            value: 123e4567-e89b-12d3-a456-426614174000
      db_name:
        type: string
        title: Current database name
    description: |
      SCDBQueryRequest designed for Client(Biz Service) which allow callback url and traceid
    title: |-
      SCDBQueryRequest designed for Client(Biz Service) which allow callback url
      and traceid
    required:
      - query
      - user
      - header
    x-secretflow-examples:
      - summary: An example query request
        value:
          query: show databases;
          user:
            user:
              nativeUser:
                name: test
                password': test
  scql.pb.SCDBQueryResultResponse:
    type: object
    properties:
      status:
        $ref: '#/definitions/scql.pb.Status'
        title: Status of response
      out_columns:
        type: array
        items:
          type: object
          $ref: '#/definitions/scql.pb.Tensor'
        description: Output columns.
      scdb_session_id:
        type: string
        title: Scdb session id
      affected_rows:
        type: string
        format: int64
        title: The number of rows affected by a select into, update, insert, or delete
      warnings:
        type: array
        items:
          type: object
          $ref: '#/definitions/scql.pb.SQLWarning'
        title: Warnings for the query
    description: SCDB query result representation (table view by columns).
  scql.pb.SCDBSubmitResponse:
    type: object
    properties:
      status:
        $ref: '#/definitions/scql.pb.Status'
        title: Status of response
      scdb_session_id:
        type: string
        title: Scdb session id
  scql.pb.SQLWarning:
    type: object
    properties:
      reason:
        type: string
        title: Description of the warning
    title: 'TODO: move SQLWarning to a proper place'
  scql.pb.Status:
    type: object
    properties:
      code:
        type: integer
        format: int32
        description: The status code, which should be one of google rpc code or custom code.
      message:
        type: string
        description: Message for recording the error information.
      details:
        type: array
        items:
          type: object
          $ref: '#/definitions/google.protobuf.Any'
        description: A list of messages that carry the additional supplementary error details.
    description: |-
      The Status type defines a logical error model that is suitable for
      different programming environments, including REST APIs and RPC APIs. It is
      used by [gRPC](https://github.com/grpc). Each Status message contains
      three pieces of data: error code, error message, and error details.

      You can find out more about this error model and how to work with it in the
      [API Design Guide](https://cloud.google.com/apis/design/errors).
  scql.pb.Tensor:
    type: object
    properties:
      name:
        type: string
        description: Tensor name.
      shape:
        $ref: '#/definitions/scql.pb.TensorShape'
        description: |-
          Tensor shape.
          In SCQL cases, it's normally [M] (a vector with M elements).
      elem_type:
        $ref: '#/definitions/scql.pb.PrimitiveDataType'
        description: Tensor element type.
      option:
        $ref: '#/definitions/scql.pb.TensorOptions'
        description: Tensor options.
      annotation:
        $ref: '#/definitions/scql.pb.TensorAnnotation'
        title: |-
          Tensor annotation carries physical status information.
          It MUST be there if the <option> is "Reference"
      int32_data:
        type: array
        items:
          type: integer
          format: int32
        title: For int8, int16, int32 data types
      int64_data:
        type: array
        items:
          type: string
          format: int64
        title: For int64 and timestamp data types
      float_data:
        type: array
        items:
          type: number
          format: float
        title: For float32 data type
      double_data:
        type: array
        items:
          type: number
          format: double
        title: For float64 data type
      bool_data:
        type: array
        items:
          type: boolean
        title: For bool data type
      string_data:
        type: array
        items:
          type: string
        title: For string and datetime data types
      ref_num:
        type: integer
        format: int32
        title: Tensor reference nums, used to delete tensor immediately
    description: A tensor data representation.
  scql.pb.TensorAnnotation:
    type: object
    properties:
      status:
        $ref: '#/definitions/scql.pb.TensorStatus'
  scql.pb.TensorOptions:
    type: string
    enum:
      - VALUE
      - REFERENCE
      - VARIABLE
    default: VALUE
    description: |-
      Tensor options.

       - VALUE: A tensor with data.
       - REFERENCE: A tensor with reference (URI).
       - VARIABLE: A tensor variable (declaration).
  scql.pb.TensorShape:
    type: object
    properties:
      dim:
        type: array
        items:
          type: object
          $ref: '#/definitions/scql.pb.TensorShape.Dimension'
    description: |-
      Defines a tensor shape. A dimension can be either an integer value
      or a symbolic variable. A symbolic variable represents an unknown
      dimension.
  scql.pb.TensorShape.Dimension:
    type: object
    properties:
      dim_value:
        type: string
        format: int64
      dim_param:
        type: string
        description: shape is unknown.
  scql.pb.TensorStatus:
    type: string
    enum:
      - TENSORSTATUS_UNKNOWN
      - TENSORSTATUS_PRIVATE
      - TENSORSTATUS_SECRET
      - TENSORSTATUS_CIPHER
      - TENSORSTATUS_PUBLIC
    default: TENSORSTATUS_UNKNOWN
    description: |2-
       - TENSORSTATUS_UNKNOWN: Unknown.
       - TENSORSTATUS_PRIVATE: Private.
       - TENSORSTATUS_SECRET: Secret, usually in the form of secret sharing.
       - TENSORSTATUS_CIPHER: Ciphertext, usually in the form of homomorphic encryption ciphertext.
       - TENSORSTATUS_PUBLIC: Public.
  scql.pb.User:
    type: object
    properties:
      account_system_type:
        $ref: '#/definitions/scql.pb.User.AccountSystemType'
      native_user:
        $ref: '#/definitions/scql.pb.User.NativeUser'
  scql.pb.User.AccountSystemType:
    type: string
    enum:
      - UNKNOWN
      - NATIVE_USER
    default: UNKNOWN
  scql.pb.User.NativeUser:
    type: object
    properties:
      name:
        type: string
        title: e.g. "zhang_san"
      password:
        type: string
        title: e.g. "123456"
  `;

export const Broker: Story = {
  args: { schema: SCHEMA },
};
