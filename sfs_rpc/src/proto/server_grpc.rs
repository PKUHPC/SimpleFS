// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_SFS_HANDLE_HANDLE: ::grpcio::Method<super::server::Post, super::server::PostResult> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/sfs_server.SFSHandle/handle",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_SFS_HANDLE_HANDLE_STREAM: ::grpcio::Method<super::server::Post, super::server::PostResult> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Duplex,
    name: "/sfs_server.SFSHandle/handle_stream",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_SFS_HANDLE_HANDLE_DIRENTS: ::grpcio::Method<super::server::Post, super::server::PostResult> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/sfs_server.SFSHandle/handle_dirents",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct SfsHandleClient {
    client: ::grpcio::Client,
}

impl SfsHandleClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        SfsHandleClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn handle_opt(&self, req: &super::server::Post, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::PostResult> {
        self.client.unary_call(&METHOD_SFS_HANDLE_HANDLE, req, opt)
    }

    pub fn handle(&self, req: &super::server::Post) -> ::grpcio::Result<super::server::PostResult> {
        self.handle_opt(req, ::grpcio::CallOption::default())
    }

    pub fn handle_async_opt(&self, req: &super::server::Post, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::PostResult>> {
        self.client.unary_call_async(&METHOD_SFS_HANDLE_HANDLE, req, opt)
    }

    pub fn handle_async(&self, req: &super::server::Post) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::PostResult>> {
        self.handle_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn handle_stream_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::server::Post>, ::grpcio::ClientDuplexReceiver<super::server::PostResult>)> {
        self.client.duplex_streaming(&METHOD_SFS_HANDLE_HANDLE_STREAM, opt)
    }

    pub fn handle_stream(&self) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::server::Post>, ::grpcio::ClientDuplexReceiver<super::server::PostResult>)> {
        self.handle_stream_opt(::grpcio::CallOption::default())
    }

    pub fn handle_dirents_opt(&self, req: &super::server::Post, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::PostResult>> {
        self.client.server_streaming(&METHOD_SFS_HANDLE_HANDLE_DIRENTS, req, opt)
    }

    pub fn handle_dirents(&self, req: &super::server::Post) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::PostResult>> {
        self.handle_dirents_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait SfsHandle {
    fn handle(&mut self, ctx: ::grpcio::RpcContext, req: super::server::Post, sink: ::grpcio::UnarySink<super::server::PostResult>);
    fn handle_stream(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::server::Post>, sink: ::grpcio::DuplexSink<super::server::PostResult>);
    fn handle_dirents(&mut self, ctx: ::grpcio::RpcContext, req: super::server::Post, sink: ::grpcio::ServerStreamingSink<super::server::PostResult>);
}

pub fn create_sfs_handle<S: SfsHandle + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_SFS_HANDLE_HANDLE, move |ctx, req, resp| {
        instance.handle(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_duplex_streaming_handler(&METHOD_SFS_HANDLE_HANDLE_STREAM, move |ctx, req, resp| {
        instance.handle_stream(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_server_streaming_handler(&METHOD_SFS_HANDLE_HANDLE_DIRENTS, move |ctx, req, resp| {
        instance.handle_dirents(ctx, req, resp)
    });
    builder.build()
}
