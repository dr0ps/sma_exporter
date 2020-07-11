use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use hyper::body::Buf;

/*
 *
 * by david-m-m 2019-Mar-17
 * by datenschuft 2020-Jan-04
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
 */

// unit definitions with scaling
lazy_static! {
    static ref SMA_UNITS: HashMap<&'static str,  u32> = {
        let mut map = HashMap::new();
        map.insert("W",10);
        map.insert("VA",10);
        map.insert("VAr",10);
        map.insert("Wh",3600);
        map.insert("VAh",3600);
        map.insert("VArh",3600);
        map.insert("A",1000);
        map.insert("V",1000);
        map.insert("°",1000);
        map.insert("Hz",1000);
        map
    };
}

//map of all defined SMA channels
//format: <channel_number>:(emparts_name>,<unit_actual>,<unit_total>)
lazy_static! {
    static ref SMA_CHANNELS: HashMap<u32, Vec<&'static str>> = {
        let mut map = HashMap::new();
        // totals
        map.insert(1, vec!["pconsume","W","Wh"]);
        map.insert(2, vec!["psupply","W","Wh"]);
        map.insert(3, vec!["qconsume","VAr","VArh"]);
        map.insert(4, vec!["qsupply","VAr","VArh"]);
        map.insert(9, vec!["sconsume","VA","VAh"]);
        map.insert(10, vec!["ssupply","VA","VAh"]);
        map.insert(13, vec!["cosphi","°"]);
        map.insert(14, vec!["frequency","Hz"]);
        // phase 1
        map.insert(21, vec!["p1consume","W","Wh"]);
        map.insert(22, vec!["p1supply","W","Wh"]);
        map.insert(23, vec!["q1consume","VAr","VArh"]);
        map.insert(24, vec!["q1supply","VAr","VArh"]);
        map.insert(29, vec!["s1consume","VA","VAh"]);
        map.insert(30, vec!["s1supply","VA","VAh"]);
        map.insert(31, vec!["i1","A"]);
        map.insert(32, vec!["u1","V"]);
        map.insert(33, vec!["cosphi1","°"]);
        // phase 2
        map.insert(41, vec!["p2consume","W","Wh"]);
        map.insert(42, vec!["p2supply","W","Wh"]);
        map.insert(43, vec!["q2consume","VAr","VArh"]);
        map.insert(44, vec!["q2supply","VAr","VArh"]);
        map.insert(49, vec!["s2consume","VA","VAh"]);
        map.insert(50, vec!["s2supply","VA","VAh"]);
        map.insert(51, vec!["i2","A"]);
        map.insert(52, vec!["u2","V"]);
        map.insert(53, vec!["cosphi2","°"]);
        // phase 3
        map.insert(61, vec!["p3consume","W","Wh"]);
        map.insert(62, vec!["p3supply","W","Wh"]);
        map.insert(63, vec!["q3consume","VAr","VArh"]);
        map.insert(64, vec!["q3supply","VAr","VArh"]);
        map.insert(69, vec!["s3consume","VA","VAh"]);
        map.insert(70, vec!["s3supply","VA","VAh"]);
        map.insert(71, vec!["i3","A"]);
        map.insert(72, vec!["u3","V"]);
        map.insert(73, vec!["cosphi3","°"]);
        // common
        map.insert(36864, vec!["speedwire-version",""]);
        map
    };
}

fn decode_obis(obis: &[u8]) -> (u16, &str){
    let mut rdr = Cursor::new(obis.to_vec());
    let measurement = rdr.read_u16::<BigEndian>().unwrap();
    let raw_type = rdr.read_i8().unwrap();

    let datatype;
    if raw_type==4 {
        datatype="actual"
    }
    else if raw_type == 8 {
        datatype = "counter"
    }
    else if raw_type == 0 && measurement==36864{
        datatype = "version"
    }
    else {
        datatype="unknown"
    }

    return (measurement, datatype);
}

pub fn decode_speedwire(datagram: &[u8]) -> HashMap<String, String>{
    let mut emparts : HashMap<String, String> = HashMap::new();

    // process data only of SMA header is present
    if datagram.starts_with(&['S' as u8, 'M' as u8, 'A' as u8])
    {
        let mut rdr = Cursor::new(datagram.to_vec());
        // datagram length
        rdr.set_position(12);
        let datalength = rdr.read_u16::<BigEndian>().unwrap() + 16;

        // serial number
        rdr.set_position(20);
        let em_id = rdr.read_u32::<BigEndian>().unwrap();
        emparts.insert("serial".to_string(), em_id.to_string());

        // timestamp
        rdr.set_position(24);
        let _timestamp = rdr.read_u32::<BigEndian>().unwrap();

        // decode OBIS data blocks
        // start with header

        let mut position: u64 = 28;
        while position < datalength as u64 {
            // decode header
            rdr.set_position(position);
            let (measurement, datatype) = decode_obis(rdr.bytes());
            // decode values
            // actual values
            if datatype == "actual" {
                rdr.set_position(position + 4);
                let value = rdr.read_u32::<BigEndian>().unwrap();
                position += 8;

                if SMA_CHANNELS.contains_key(&(measurement as u32))
                {
                    let sma_channel = &SMA_CHANNELS[&(measurement as u32)];
                    emparts.insert(sma_channel[0].to_string(), (value / SMA_UNITS[sma_channel[1]]).to_string());
                    let unit_key = sma_channel[0].to_owned() +"unit";
                    emparts.insert(unit_key, sma_channel[1].to_string());
                }
            }
            // counter values
            else if datatype == "counter"
            {
                rdr.set_position(position + 4);
                let value = rdr.read_u64::<BigEndian>().unwrap();
                position += 12;
                if SMA_CHANNELS.contains_key(&(measurement as u32))
                {
                    let sma_channel = &SMA_CHANNELS[&(measurement as u32)];
                    let counter_key = sma_channel[0].to_owned() +"counter";
                    emparts.insert(counter_key, (value / SMA_UNITS[sma_channel[2]] as u64).to_string());
                    let unit_key = sma_channel[0].to_owned() +"counterunit";
                    emparts.insert(unit_key, sma_channel[2].to_string());
                }
            }
            else if datatype == "version" {
                position += 8
          }
          else {
              position += 8
          }
        }
    }
    emparts
}