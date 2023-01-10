extern crate socket2;

use self::socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::{SocketAddr, Ipv4Addr};
use crate::sma_decoder::decode_speedwire;
use std::collections::HashMap;
use std::mem::MaybeUninit;

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

//default values
const PORT: u16  = 9522;

pub fn initialize_socket() -> Result<Socket, &'static str> {
    let socket;
    match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
        Ok(s) => { socket = s; }
        Err(_e) => { return Err("Unable to create socket"); }
    }
    match socket.set_reuse_address(true) {
        Ok(()) => {}
        Err(_e) => { return Err("Unable to reuse address."); }
    }
    match socket.bind(&SockAddr::from(SocketAddr::new(
        Ipv4Addr::new(0, 0, 0, 0).into(),
        PORT))) {
        Ok(()) => {}
        Err(_e) => { return Err("Unable to bind."); }
    }
    return match socket.join_multicast_v4(&Ipv4Addr::new(239, 12, 255, 254), &Ipv4Addr::new(0, 0, 0, 0)) {
        Ok(()) => { Ok(socket) }
        Err(_e) => { Err("Unable to join multicast.") }
    }
}

/// Assume the `buf`fer to be initialised.
// TODO: replace with `MaybeUninit::slice_assume_init_ref` once stable.
unsafe fn assume_init(buf: &[MaybeUninit<u8>]) -> &[u8] {
    &*(buf as *const [MaybeUninit<u8>] as *const [u8])
}

pub fn read_sma_homemanager(socket : &Socket) -> HashMap<String, String> {
    let mut buffer = [MaybeUninit::new(0 as u8); 608];
    assert!(socket.recv(&mut buffer).is_ok());
    return decode_speedwire(unsafe { assume_init(&buffer) } );
}
