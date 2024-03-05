use crate::udp_client::{read_sma_homemanager, initialize_socket};
use http_body_util::{BodyExt, combinators::BoxBody, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use prometheus::{Opts, TextEncoder, Encoder, register, gather, GaugeVec, CounterVec};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;

/*
 *
 * by dr0ps 2020-Jul-18
 *
 *
 *  this software is released under GNU General Public License, version 2.
 *  This program is free software;
 *  you can redistribute it and/or modify it under the terms of the GNU General Public License
 *  as published by the Free Software Foundation; version 2 of the License.
 *  This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
 *  without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *  See the GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along with this program;
 *  if not, write to the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
 *
 *
 */

mod sma_decoder;
mod udp_client;

lazy_static! {
    static ref LOCK: Arc<Mutex<u32>> = Arc::new(Mutex::new(0_u32));
}

async fn handle(_: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let _lock = LOCK.lock().unwrap();

    let metric_families = gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(Response::new(full(buffer)))
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into()).boxed()
}

lazy_static! {
    pub static ref CHANNEL_MAPPINGS: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        // totals
        map.insert("pconsume", vec!["smahomemanager_real_consumed_watts","smahomemanager_real_consumed_watthours","Total"]);
        map.insert("psupply", vec!["smahomemanager_real_supplied_watts", "smahomemanager_real_supplied_watthours","Total"]);
        map.insert("qconsume", vec!["smahomemanager_reactive_consumed_var","smahomemanager_reactive_consumed_varh", "Total"]);
        map.insert("qsupply", vec!["smahomemanager_reactive_supplied_var","smahomemanager_reactive_supplied_varh", "Total"]);
        map.insert("sconsume", vec!["smahomemanager_apparent_consumed_va","smahomemanager_apparent_consumed_vah", "Total"]);
        map.insert("ssupply", vec!["smahomemanager_apparent_supplied_va","smahomemanager_apparent_supplied_vah", "Total"]);
        map.insert("cosphi", vec!["smahomemanager_cosphi_degrees","Total"]);
        map.insert("frequency", vec!["smahomemanager_frequency_millihertz", "Total"]);
        // phase 1
        map.insert("p1consume", vec!["smahomemanager_real_consumed_watts", "smahomemanager_real_consumed_watthours","L1"]);
        map.insert("p1supply", vec!["smahomemanager_real_supplied_watts", "smahomemanager_real_supplied_watthours","L1"]);
        map.insert("q1consume", vec!["smahomemanager_reactive_consumed_var","smahomemanager_reactive_consumed_varh", "L1"]);
        map.insert("q1supply", vec!["smahomemanager_reactive_supplied_var","smahomemanager_reactive_supplied_varh", "L1"]);
        map.insert("s1consume", vec!["smahomemanager_apparent_consumed_va","smahomemanager_apparent_consumed_vah", "L1"]);
        map.insert("s1supply", vec!["smahomemanager_apparent_supplied_va","smahomemanager_apparent_supplied_vah", "L1"]);
        map.insert("i1", vec!["smahomemanager_current_milliamperes","L1"]);
        map.insert("u1", vec!["smahomemanager_voltage_millivolts","L1"]);
        map.insert("cosphi1", vec!["smahomemanager_cosphi_degrees","L1"]);
        // phase 2
        map.insert("p2consume", vec!["smahomemanager_real_consumed_watts", "smahomemanager_real_consumed_watthours","L2"]);
        map.insert("p2supply", vec!["smahomemanager_real_supplied_watts", "smahomemanager_real_supplied_watthours","L2"]);
        map.insert("q2consume", vec!["smahomemanager_reactive_consumed_var","smahomemanager_reactive_consumed_varh", "L2"]);
        map.insert("q2supply", vec!["smahomemanager_reactive_supplied_var","smahomemanager_reactive_supplied_varh", "L2"]);
        map.insert("s2consume", vec!["smahomemanager_apparent_consumed_va","smahomemanager_apparent_consumed_vah", "L2"]);
        map.insert("s2supply", vec!["smahomemanager_apparent_supplied_va","smahomemanager_apparent_supplied_vah", "L2"]);
        map.insert("i2", vec!["smahomemanager_current_milliamperes","L2"]);
        map.insert("u2", vec!["smahomemanager_voltage_millivolts","L2"]);
        map.insert("cosphi2", vec!["smahomemanager_cosphi_degrees","L2"]);
        // phase 3
        map.insert("p3consume", vec!["smahomemanager_real_consumed_watts", "smahomemanager_real_consumed_watthours","L3"]);
        map.insert("p3supply", vec!["smahomemanager_real_supplied_watts", "smahomemanager_real_supplied_watthours","L3"]);
        map.insert("q3consume", vec!["smahomemanager_reactive_consumed_var","smahomemanager_reactive_consumed_varh", "L3"]);
        map.insert("q3supply", vec!["smahomemanager_reactive_supplied_var","smahomemanager_reactive_supplied_varh", "L3"]);
        map.insert("s3consume", vec!["smahomemanager_apparent_consumed_va","smahomemanager_apparent_consumed_vah", "L3"]);
        map.insert("s3supply", vec!["smahomemanager_apparent_supplied_va","smahomemanager_apparent_supplied_vah", "L3"]);
        map.insert("i3", vec!["smahomemanager_current_milliamperes","L3"]);
        map.insert("u3", vec!["smahomemanager_voltage_millivolts","L3"]);
        map.insert("cosphi3", vec!["smahomemanager_cosphi_degrees","L3"]);
        map
    };
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // Create a Counter.
    let mut gauges:HashMap<&'static str, GaugeVec> = HashMap::new();
    let mut counters:HashMap<&'static str, CounterVec> = HashMap::new();

    for key in CHANNEL_MAPPINGS.keys() {

        let values = CHANNEL_MAPPINGS.get(key).unwrap();
        let gauge_key = values[0];
        if !gauges.contains_key(gauge_key)
        {
            let gauge_opts = Opts::new(gauge_key, values[0]);
            let gauge = GaugeVec::new(gauge_opts, &["phase"]).unwrap();
            register(Box::new(gauge.borrow().clone())).unwrap();
            gauges.insert(gauge_key, gauge);
        }

        let values = CHANNEL_MAPPINGS.get(key).unwrap();
        if values.len() == 3 {
            let counter_key = values[1];
            if !counters.contains_key(counter_key) {
                let counter_opts = Opts::new(counter_key, values[1]);
                let counter = CounterVec::new(counter_opts, &["phase"]).unwrap();
                register(Box::new(counter.borrow().clone())).unwrap();
                counters.insert(counter_key, counter);
            }
        }
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], 9743));

    let socket;
    match initialize_socket() {
        Err(e) => {
            println!("Error when creating socket: {}", e);
            exit(1);
        }
        Ok(s) => {socket = s;}
    }

    // Spawn one-second timer
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let em_values = read_sma_homemanager(&socket);
            let _lock = LOCK.lock().unwrap();

            for key in CHANNEL_MAPPINGS.keys() {

                let gauge_value_key = format!("{}",key).to_string();
                if em_values.contains_key(gauge_value_key.as_str())
                {
                    let values = CHANNEL_MAPPINGS.get(key).unwrap();
                    let gauge_key = values[0];
                    let gauge_phase = if values.len() == 3 {values[2]} else {values[1]};
                    gauges.get(gauge_key).unwrap().with_label_values(&[gauge_phase]).set(em_values.get(gauge_value_key.as_str()).unwrap().parse::<f64>().unwrap());
                }

                let counter_value_key = format!("{}counter", key).to_string();
                if em_values.contains_key(counter_value_key.as_str())
                {
                    let values = CHANNEL_MAPPINGS.get(key).unwrap();
                    let counter_key = values[1];
                    let counter_phase = values[2];
                    let counter = counters.get(counter_key).unwrap().with_label_values(&[counter_phase]);
                    counter.reset();
                    counter.inc_by(em_values.get(counter_value_key.as_str()).unwrap().parse::<f64>().unwrap());
                }
            }
        }
    });

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
