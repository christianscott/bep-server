extern crate futures;
extern crate grpc;
extern crate tls_api_stub;

use publish_build_event_proto_rs::{
    Empty, PublishBuildEvent, PublishBuildEventServer, PublishBuildToolEventStreamRequest,
    PublishBuildToolEventStreamResponse, PublishLifecycleEventRequest,
};

use std::env;
use std::str::FromStr;
use std::thread;

struct PublishBuildEventImpl;

impl PublishBuildEvent for PublishBuildEventImpl {
    fn publish_lifecycle_event(
        &self,
        _o: ::grpc::RequestOptions,
        _req: PublishLifecycleEventRequest,
    ) -> ::grpc::SingleResponse<Empty> {
        println!("publish_lifecycle_event");
        grpc::SingleResponse::completed(Empty::new())
    }

    fn publish_build_tool_event_stream(
        &self,
        _o: ::grpc::RequestOptions,
        mut req: ::grpc::StreamingRequest<PublishBuildToolEventStreamRequest>,
    ) -> ::grpc::StreamingResponse<PublishBuildToolEventStreamResponse> {
        println!("publish_build_tool_event_stream");

        let mut r = Vec::new();
        loop {
            match req.0.poll() {
                Ok(futures::Async::Ready(None)) => {
                    break;
                }
                Ok(futures::Async::Ready(Some(req))) => {
                    println!("{:?}", req);
                    let mut res = PublishBuildToolEventStreamResponse::new();
                    res.set_sequence_number(req.get_ordered_build_event().sequence_number);
                    r.push(res)
                }
                _ => {}
            }
        }

        println!("counted {} messages", r.len());
        grpc::StreamingResponse::iter(r.into_iter())
    }
}

fn main() {
    let mut server = grpc::ServerBuilder::<tls_api_stub::TlsAcceptor>::new();
    let port = u16::from_str(&env::args().nth(1).unwrap_or_else(|| "50051".to_owned())).unwrap();
    server.http.set_port(port);
    server.add_service(PublishBuildEventServer::new_service_def(
        PublishBuildEventImpl,
    ));
    server.http.set_cpu_pool_threads(4);
    let server = server.build().expect("server");
    let port = server.local_addr().port().unwrap();
    println!("greeter server started on port {port}");

    loop {
        thread::park();
    }
}
