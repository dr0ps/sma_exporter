extern crate socket2;

use self::socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::{SocketAddr, Ipv4Addr};
use crate::sma_decoder::decode_speedwire;
use std::collections::HashMap;

/*
 *
 * by Wenger Florian 2015-09-02
 * wenger@unifox.at
 *
 * endless loop (until ctrl+c) displays measurement from SMA Energymeter
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
 * 2018-12-22 Tommi2Day small enhancements
 * 2019-08-13 datenschuft run without config
 * 2020-01-04 datenschuft changes to tun with speedwiredecoder
 *
 */

//default values
const PORT: u16  = 9522;

pub fn initialize_socket() -> Socket {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&SockAddr::from(SocketAddr::new(
        Ipv4Addr::new(0, 0, 0, 0).into(),
        PORT))).unwrap();
    assert!(socket.join_multicast_v4(&Ipv4Addr::new(239, 12, 255, 254), &Ipv4Addr::new(0, 0, 0, 0)).is_ok());
    socket
}

pub fn read_sma_homemanager(socket : &Socket) -> HashMap<String, String> {
    let mut buffer = [0;608];
    assert!(socket.recv(&mut buffer).is_ok());
    return decode_speedwire(&buffer);
}
