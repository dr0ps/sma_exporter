use lazy_static::lazy_static;
use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use prometheus::{Opts, Counter, TextEncoder, Encoder, register, gather, GaugeVec, CounterVec};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use crate::udp_client::{read_sma_homemanager, initialize_socket};

mod sma_decoder;
mod udp_client;

lazy_static! {
    static ref LOCK: Arc<Mutex<u32>> = Arc::new(Mutex::new(0_u32));
}

async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let _lock = LOCK.lock().unwrap();

    let metric_families = gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(Response::new(String::from_utf8(buffer).unwrap().into()))
}

#[tokio::main]
async fn main() {

    // Create a Counter.
    let counter_opts = Opts::new("loop_counter", "Loop counter");
    let counter = Counter::with_opts(counter_opts).unwrap();
    let current_supply_opts = Opts::new("gauge_supply_watts", "Current supply");
    let current_supply = GaugeVec::new(current_supply_opts, &["phase"]).unwrap();
    let current_consume_opts = Opts::new("gauge_consume_watts", "Current consumption");
    let current_consume = GaugeVec::new(current_consume_opts, &["phase"]).unwrap();

    let total_supply_opts = Opts::new("counter_supply_watthours", "Total supply");
    let total_supply = CounterVec::new(total_supply_opts, &["phase"]).unwrap();
    let total_consume_opts = Opts::new("counter_consume_watthours", "Total consumption");
    let total_consume = CounterVec::new(total_consume_opts, &["phase"]).unwrap();


    register(Box::new(counter.clone())).unwrap();
    register(Box::new(current_supply.clone())).unwrap();
    register(Box::new(current_consume.clone())).unwrap();
    register(Box::new(total_supply.clone())).unwrap();
    register(Box::new(total_consume.clone())).unwrap();


    let addr = SocketAddr::from(([127, 0, 0, 1], 9635));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle))
    });

    let socket = initialize_socket();

    // Spawn one second timer
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let em_values = read_sma_homemanager(&socket);

            let _lock = LOCK.lock().unwrap();

            if em_values.contains_key("psupply")
            {
                &current_supply.with_label_values(&["sum"]).set(em_values.get("psupply").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p1supply")
            {
                &current_supply.with_label_values(&["L1"]).set(em_values.get("p1supply").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p2supply")
            {
                &current_supply.with_label_values(&["L2"]).set(em_values.get("p2supply").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p3supply")
            {
                &current_supply.with_label_values(&["L3"]).set(em_values.get("p3supply").unwrap().parse::<f64>().unwrap());
            }

            if em_values.contains_key("pconsume")
            {
                &current_consume.with_label_values(&["sum"]).set(em_values.get("pconsume").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p1consume")
            {
                &current_consume.with_label_values(&["L1"]).set(em_values.get("p1consume").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p2consume")
            {
                &current_consume.with_label_values(&["L2"]).set(em_values.get("p2consume").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p3consume")
            {
                &current_consume.with_label_values(&["L3"]).set(em_values.get("p3consume").unwrap().parse::<f64>().unwrap());
            }

            if em_values.contains_key("psupplycounter")
            {
                &total_supply.with_label_values(&["sum"]).reset();
                &total_supply.with_label_values(&["sum"]).inc_by(em_values.get("psupplycounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p1supplycounter")
            {
                &total_supply.with_label_values(&["L1"]).reset();
                &total_supply.with_label_values(&["L1"]).inc_by(em_values.get("p1supplycounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p2supplycounter")
            {
                &total_supply.with_label_values(&["L2"]).reset();
                &total_supply.with_label_values(&["L2"]).inc_by(em_values.get("p2supplycounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p3supplycounter")
            {
                &total_supply.with_label_values(&["L3"]).reset();
                &total_supply.with_label_values(&["L3"]).inc_by(em_values.get("p3supplycounter").unwrap().parse::<f64>().unwrap());
            }

            if em_values.contains_key("pconsumecounter")
            {
                &total_consume.with_label_values(&["sum"]).reset();
                &total_consume.with_label_values(&["sum"]).inc_by(em_values.get("pconsumecounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p1consumecounter")
            {
                &total_consume.with_label_values(&["L1"]).reset();
                &total_consume.with_label_values(&["L1"]).inc_by(em_values.get("p1consumecounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p2consumecounter")
            {
                &total_consume.with_label_values(&["L2"]).reset();
                &total_consume.with_label_values(&["L2"]).inc_by(em_values.get("p2consumecounter").unwrap().parse::<f64>().unwrap());
            }
            if em_values.contains_key("p3consumecounter")
            {
                &total_consume.with_label_values(&["L3"]).reset();
                &total_consume.with_label_values(&["L3"]).inc_by(em_values.get("p3consumecounter").unwrap().parse::<f64>().unwrap());
            }

            counter.inc()
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
