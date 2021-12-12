use num::bigint::BigUint;

use crate::arithmetic;
use crate::command::{ Request, Response };
use crate::client::Client;
use crate::location::Location;
use crate::utils::Result;

/*
 * Find successor node of a key, starting by asking node at location.
 */
pub async fn find_successor(location: &Location, key: &BigUint) -> Result<Location> {
    let pred = find_predecessor(location, key).await?;
    return get_successor(&pred).await;
}

/*
 * Find predecessor node of a key, starting by asking node at location.
 * This function is a part of lookup process.
 */
async fn find_predecessor(location: &Location, key: &BigUint) -> Result<Location> {
    let mut location = location.clone();
    while !arithmetic::is_in_range(
        key,
        (&location.identifier, false),
        (&get_successor(&location).await?.identifier, true)
    ) {
        location = find_closest_preceding_finger(&location, &key).await?;
    }
    Ok(location)
}

/*
 * Find successor node of a node at location.
 */
async fn get_successor(location: &Location) -> Result<Location> {
    let request = Request::GetSuccessor {
        virtual_node_id: location.virtual_node_id,
    };
    let mut client = Client::new(location).await?;
    client.send_request(request).await?;
    let response = client.receive().await?;
    let res_location = match response {
        Response::GetSuccessor { location } => location,
        _ => {
            return Err(
                "Error receiving response while doing GETSUCCESSOR. Got unexpected response type."
                .into()
            );
        }
    };
    Ok(res_location)
}

/*
 * Find predecessor node of a node at location.
 */
pub async fn get_predecessor(location: &Location) -> Result<Location> {
    let request = Request::GetPredecessor {
        virtual_node_id: location.virtual_node_id,
    };
    let mut client = Client::new(location).await?;
    client.send_request(request).await?;
    let response = client.receive().await?;
    let res_location = match response {
        Response::GetSuccessor { location } => location,
        _ => {
            return Err(
                "Error receiving response while doing GETPREDECESSOR. Got unexpected response type."
                .into()
            );
        }
    };
    Ok(res_location)
}

/*
 * Find closest preceding node of a key, by finding from fingers of a node at location.
 */
async fn find_closest_preceding_finger(location: &Location, key: &BigUint) -> Result<Location> {
    let request = Request::ClosestPrecedingFinger {
        virtual_node_id: location.virtual_node_id,
        key: key.clone(),
    };
    let mut client = Client::new(location).await?;
    client.send_request(request).await?;
    let response = client.receive().await?;
    let res_location = match response {
        Response::ClosestPrecedingFinger { location } => location,
        _ => {
            return Err(
                "Error receiving response while doing CLOSESTPRECEDINGFINGER. Got unexpected response type."
                .into()
            );
        }
    };
    Ok(res_location)
}

