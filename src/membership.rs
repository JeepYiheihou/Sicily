use std::sync::Arc;

use crate::arithmetic;
use crate::client::Client;
use crate::command::Request;
use crate::location::Location;
use crate::node::NodeList;
use crate::process;
use crate::utils::Result;

/*
 * Function called every time a new node tries to join in a cluster
 * by talking to location, which represents a node that is already in the cluster.
 */
pub async fn join(
    node_list: Arc<NodeList>,
    virtual_node_id: u8,
    location: Location
) -> Result<()> {

    /* 1. Retrieve the local identifier of the node. */
    let key = {
        let node = node_list.node_list[virtual_node_id as usize].lock().await;
        node.location.identifier.clone()
    };

    /* 2. Based on the identifier, retrieve the successor. */
    let successor = process::find_successor(&location, &key).await?;

    /* 3. Update the node's metadata. */
    {
        let mut node = node_list.node_list[virtual_node_id as usize].lock().await;
        node.predecessor = None;
        node.successor = Some(successor);
    }
    Ok(())
}

/*
 * Periodic function called to stabilize the metadata of nodes in the cluster.
 */
pub async fn stablize(node_list: NodeList, virtual_node_id: u8) -> Result<()> {
    let (mut successor, local_location) = {
        let node = node_list.node_list[virtual_node_id as usize].lock().await;
        let successor = Location::option_to_result(&node.successor)?;
        (successor, node.location.clone())
    };

    let predecessor_of_successor = process::get_predecessor(&successor).await?;
    if arithmetic::is_in_range(
        &predecessor_of_successor.identifier,
        (&local_location.identifier, false),
        (&successor.identifier, false)) {
            {
                let mut node = node_list.node_list[virtual_node_id as usize].lock().await;
                node.successor = Some(predecessor_of_successor.clone());
            }
            successor = predecessor_of_successor.clone();
        }
    
    notify(local_location, successor).await?;
    Ok(())
}

async fn notify(local_location: Location, target_location: Location) -> Result<()> {
    let request = Request::Notify {
        virtual_node_id: target_location.virtual_node_id,
        notifier: local_location,
    };
    let mut client = Client::new(&target_location).await?;

    Ok(())
}