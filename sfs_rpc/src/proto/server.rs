// This file is generated by rust-protobuf 2.27.1. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `server.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_27_1;

#[derive(PartialEq,Clone,Default)]
pub struct Post {
    // message fields
    pub option: i32,
    pub data: ::std::vec::Vec<u8>,
    pub extra: ::std::vec::Vec<u8>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Post {
    fn default() -> &'a Post {
        <Post as ::protobuf::Message>::default_instance()
    }
}

impl Post {
    pub fn new() -> Post {
        ::std::default::Default::default()
    }

    // int32 option = 1;


    pub fn get_option(&self) -> i32 {
        self.option
    }
    pub fn clear_option(&mut self) {
        self.option = 0;
    }

    // Param is passed by value, moved
    pub fn set_option(&mut self, v: i32) {
        self.option = v;
    }

    // bytes data = 2;


    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.data = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.data
    }

    // Take field
    pub fn take_data(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.data, ::std::vec::Vec::new())
    }

    // bytes extra = 3;


    pub fn get_extra(&self) -> &[u8] {
        &self.extra
    }
    pub fn clear_extra(&mut self) {
        self.extra.clear();
    }

    // Param is passed by value, moved
    pub fn set_extra(&mut self, v: ::std::vec::Vec<u8>) {
        self.extra = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_extra(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.extra
    }

    // Take field
    pub fn take_extra(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.extra, ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for Post {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.option = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.data)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.extra)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.option != 0 {
            my_size += ::protobuf::rt::value_size(1, self.option, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.data.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.data);
        }
        if !self.extra.is_empty() {
            my_size += ::protobuf::rt::bytes_size(3, &self.extra);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.option != 0 {
            os.write_int32(1, self.option)?;
        }
        if !self.data.is_empty() {
            os.write_bytes(2, &self.data)?;
        }
        if !self.extra.is_empty() {
            os.write_bytes(3, &self.extra)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Post {
        Post::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "option",
                |m: &Post| { &m.option },
                |m: &mut Post| { &mut m.option },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "data",
                |m: &Post| { &m.data },
                |m: &mut Post| { &mut m.data },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "extra",
                |m: &Post| { &m.extra },
                |m: &mut Post| { &mut m.extra },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Post>(
                "Post",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Post {
        static instance: ::protobuf::rt::LazyV2<Post> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Post::new)
    }
}

impl ::protobuf::Clear for Post {
    fn clear(&mut self) {
        self.option = 0;
        self.data.clear();
        self.extra.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Post {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Post {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PostResult {
    // message fields
    pub err: i32,
    pub data: ::std::vec::Vec<u8>,
    pub extra: ::std::vec::Vec<u8>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a PostResult {
    fn default() -> &'a PostResult {
        <PostResult as ::protobuf::Message>::default_instance()
    }
}

impl PostResult {
    pub fn new() -> PostResult {
        ::std::default::Default::default()
    }

    // int32 err = 1;


    pub fn get_err(&self) -> i32 {
        self.err
    }
    pub fn clear_err(&mut self) {
        self.err = 0;
    }

    // Param is passed by value, moved
    pub fn set_err(&mut self, v: i32) {
        self.err = v;
    }

    // bytes data = 2;


    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.data = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.data
    }

    // Take field
    pub fn take_data(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.data, ::std::vec::Vec::new())
    }

    // bytes extra = 3;


    pub fn get_extra(&self) -> &[u8] {
        &self.extra
    }
    pub fn clear_extra(&mut self) {
        self.extra.clear();
    }

    // Param is passed by value, moved
    pub fn set_extra(&mut self, v: ::std::vec::Vec<u8>) {
        self.extra = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_extra(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.extra
    }

    // Take field
    pub fn take_extra(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.extra, ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for PostResult {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.err = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.data)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.extra)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.err != 0 {
            my_size += ::protobuf::rt::value_size(1, self.err, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.data.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.data);
        }
        if !self.extra.is_empty() {
            my_size += ::protobuf::rt::bytes_size(3, &self.extra);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.err != 0 {
            os.write_int32(1, self.err)?;
        }
        if !self.data.is_empty() {
            os.write_bytes(2, &self.data)?;
        }
        if !self.extra.is_empty() {
            os.write_bytes(3, &self.extra)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> PostResult {
        PostResult::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "err",
                |m: &PostResult| { &m.err },
                |m: &mut PostResult| { &mut m.err },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "data",
                |m: &PostResult| { &m.data },
                |m: &mut PostResult| { &mut m.data },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "extra",
                |m: &PostResult| { &m.extra },
                |m: &mut PostResult| { &mut m.extra },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<PostResult>(
                "PostResult",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static PostResult {
        static instance: ::protobuf::rt::LazyV2<PostResult> = ::protobuf::rt::LazyV2::INIT;
        instance.get(PostResult::new)
    }
}

impl ::protobuf::Clear for PostResult {
    fn clear(&mut self) {
        self.err = 0;
        self.data.clear();
        self.extra.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PostResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PostResult {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0cserver.proto\x12\nsfs_server\"H\n\x04Post\x12\x16\n\x06option\x18\
    \x01\x20\x01(\x05R\x06option\x12\x12\n\x04data\x18\x02\x20\x01(\x0cR\x04\
    data\x12\x14\n\x05extra\x18\x03\x20\x01(\x0cR\x05extra\"H\n\nPostResult\
    \x12\x10\n\x03err\x18\x01\x20\x01(\x05R\x03err\x12\x12\n\x04data\x18\x02\
    \x20\x01(\x0cR\x04data\x12\x14\n\x05extra\x18\x03\x20\x01(\x0cR\x05extra\
    2\xbc\x01\n\tSFSHandle\x122\n\x06handle\x12\x10.sfs_server.Post\x1a\x16.\
    sfs_server.PostResult\x12=\n\rhandle_stream\x12\x10.sfs_server.Post\x1a\
    \x16.sfs_server.PostResult(\x010\x01\x12<\n\x0ehandle_dirents\x12\x10.sf\
    s_server.Post\x1a\x16.sfs_server.PostResult0\x01b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
