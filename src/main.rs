extern crate futures;
extern crate grpc;
extern crate protobuf;
extern crate tls_api_stub;

use publish_build_event_proto_rs::build_event_stream;
use publish_build_event_proto_rs::build_events;
use publish_build_event_proto_rs::{
    Empty, PublishBuildEvent, PublishBuildEventServer, PublishBuildToolEventStreamRequest,
    PublishBuildToolEventStreamResponse, PublishLifecycleEventRequest,
};

use std::env;
use std::str::FromStr;
use std::thread;

struct PublishBuildEventImpl;

impl PublishBuildEventImpl {
    fn handle_build_event(&self, event: build_events::BuildEvent) -> () {
        match event.event {
            Some(build_events::BuildEvent_oneof_event::bazel_event(any)) => {
                if any.type_url == "type.googleapis.com/build_event_stream.BuildEvent" {
                    let msg =
                        protobuf::parse_from_bytes::<build_event_stream::BuildEvent>(&any.value)
                            .expect("could not parse BuildEvent");
                    println!("{:#?}", msg);
                }
            }
            _ => {}
        }
    }
}

impl PublishBuildEvent for PublishBuildEventImpl {
    fn publish_lifecycle_event(
        &self,
        _o: grpc::RequestOptions,
        _req: PublishLifecycleEventRequest,
    ) -> grpc::SingleResponse<Empty> {
        println!("publish_lifecycle_event");
        grpc::SingleResponse::completed(Empty::new())
    }

    fn publish_build_tool_event_stream(
        &self,
        _o: grpc::RequestOptions,
        mut req: grpc::StreamingRequest<PublishBuildToolEventStreamRequest>,
    ) -> grpc::StreamingResponse<PublishBuildToolEventStreamResponse> {
        println!("publish_build_tool_event_stream");

        let mut r = Vec::new();
        loop {
            match req.0.poll() {
                Ok(futures::Async::Ready(None)) => {
                    // stream closed
                    break;
                }
                Ok(futures::Async::Ready(Some(req))) => {
                    let mut res = PublishBuildToolEventStreamResponse::new();
                    let ordered_build_event = req.get_ordered_build_event();

                    self.handle_build_event(ordered_build_event.event.clone().unwrap());

                    res.set_sequence_number(ordered_build_event.sequence_number);
                    res.set_stream_id(ordered_build_event.stream_id.clone().unwrap());
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
    println!("BEP server started on port {port}");

    loop {
        thread::park();
    }
}
