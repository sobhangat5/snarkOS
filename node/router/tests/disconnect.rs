// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

mod common;
use common::*;

use snarkos_node_tcp::{protocols::Handshake, P2P};

use core::time::Duration;

#[tokio::test]
async fn test_disconnect_without_handshake() {
    initialize_logger(3);

    // Create 2 routers.
    let node0 = validator(0, 1).await;
    let node1 = client(0, 1).await;
    assert_eq!(node0.number_of_connected_peers(), 0);
    assert_eq!(node1.number_of_connected_peers(), 0);

    // Connect node0 to node1.
    node0.connect(node1.local_ip());
    // Sleep briefly.
    tokio::time::sleep(Duration::from_millis(100)).await;

    print_tcp!(node0);
    print_tcp!(node1);

    assert_eq!(node0.tcp().num_connected(), 1);
    assert_eq!(node0.tcp().num_connecting(), 0);
    assert_eq!(node1.tcp().num_connected(), 1);
    assert_eq!(node1.tcp().num_connecting(), 0);

    // Disconnect node0 from node1.
    node0.disconnect(node1.local_ip());
    // Sleep briefly.
    tokio::time::sleep(Duration::from_millis(100)).await;

    print_tcp!(node0);
    print_tcp!(node1);

    assert_eq!(node0.tcp().num_connected(), 0);
    assert_eq!(node0.tcp().num_connecting(), 0);
    assert_eq!(node1.tcp().num_connected(), 1); // Router 1 has no way of knowing that Router 0 disconnected.
    assert_eq!(node1.tcp().num_connecting(), 0);
}

#[tokio::test]
async fn test_disconnect_with_handshake() {
    initialize_logger(3);

    // Create 2 routers.
    let node0 = validator(0, 1).await;
    let node1 = client(0, 1).await;
    assert_eq!(node0.number_of_connected_peers(), 0);
    assert_eq!(node1.number_of_connected_peers(), 0);

    // Enable handshake protocol.
    node0.enable_handshake().await;
    node1.enable_handshake().await;

    // Connect node0 to node1.
    node0.connect(node1.local_ip());
    // Sleep briefly.
    tokio::time::sleep(Duration::from_millis(100)).await;

    print_tcp!(node0);
    print_tcp!(node1);

    // Check the TCP level.
    assert_eq!(node0.tcp().num_connected(), 1);
    assert_eq!(node0.tcp().num_connecting(), 0);
    assert_eq!(node1.tcp().num_connected(), 1);
    assert_eq!(node1.tcp().num_connecting(), 0);

    // Check the router level.
    assert_eq!(node0.number_of_connected_peers(), 1);
    assert_eq!(node1.number_of_connected_peers(), 1);

    // Disconnect node0 from node1.
    node0.disconnect(node1.local_ip());
    // Sleep briefly.
    tokio::time::sleep(Duration::from_millis(100)).await;

    print_tcp!(node0);
    print_tcp!(node1);

    assert_eq!(node0.tcp().num_connected(), 0);
    assert_eq!(node0.tcp().num_connecting(), 0);
    assert_eq!(node1.tcp().num_connected(), 1); // Router 1 has no way of knowing that Router 0 disconnected.
    assert_eq!(node1.tcp().num_connecting(), 0);
}
