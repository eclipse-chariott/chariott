use chariott_common::tokio_runtime_fork::BuilderExt;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ess::EventSubSystem;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver};
type Ess = EventSubSystem<ClientId, EventId, Event, Event>;

const EVENT_ID: EventId = EventId;
const DATA1: &str = "data1";
const NUMBER_OF_EVENTS: &[usize] = &[1000, 10000];
const NUMBER_OF_SUBSCRIBERS: &[usize] = &[1, 10, 100];

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct EventId;

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BenchmarkEvent")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ClientId(String);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
struct SeqNum(u64);

#[derive(Clone)]
struct Event(EventId, SeqNum, &'static str);

fn event_sub_system_bench(c: &mut Criterion) {
    for events in NUMBER_OF_EVENTS.iter().cloned() {
        for subscribers in NUMBER_OF_SUBSCRIBERS.iter().cloned() {
            let runtime =
                tokio::runtime::Builder::new_multi_thread().worker_threads(1).fork().unwrap();
            let sut = Arc::new(Ess::new_with_config(
                ess::Config::default()
                    .set_client_buffer_size(events)
                    .set_publish_buffer_size(events)
                    .clone(),
            ));

            let (sender, _) = broadcast::channel(subscribers);
            for i in 0..subscribers {
                let client_id = ClientId(format!("client{}", i));
                let (_, mut receiver_stream) = sut.read_events(client_id.clone());
                {
                    let sender = sender.clone();
                    _ = runtime.handle().spawn(async move {
                        let mut count: usize = 0;
                        while (receiver_stream.next().await).is_some() {
                            count += 1;
                            if count == events {
                                count = 0;
                                sender.send(()).unwrap();
                            }
                        }
                    });
                }
                for sub in sut.register_subscriptions(client_id, [EVENT_ID]).unwrap() {
                    runtime.handle().spawn(
                        sub.serve(move |Event(id, _, data), seq| Event(id, SeqNum(seq), data)),
                    );
                }
            }

            let function_setup = FunctionSetup { events, subscribers };
            c.bench_with_input(
                BenchmarkId::new("ess", function_setup),
                &function_setup,
                |b, &function_setup| {
                    b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                        bench_function(function_setup, sut.clone(), sender.subscribe()).await
                    });
                },
            );
        }
    }
}

async fn bench_function(setup: FunctionSetup, sut: Arc<Ess>, mut receiver: Receiver<()>) {
    for i in 0..setup.events {
        sut.publish(&EVENT_ID, Event(EVENT_ID, SeqNum(i as _), DATA1));
    }
    for _ in 0..setup.subscribers {
        receiver.recv().await.unwrap();
    }
}

#[derive(Clone, Copy)]
struct FunctionSetup {
    events: usize,
    subscribers: usize,
}

impl std::fmt::Display for FunctionSetup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}-subscribers/{}-events", self.subscribers, self.events).as_str())
    }
}

criterion_group!(benches, event_sub_system_bench);
criterion_main!(benches);
