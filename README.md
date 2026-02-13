# Agent Project

This project is a framework for building and testing multi-agent systems. For now I am planning to include:

* **Transport Layer**: A flexible communication interface supporting various protocols (UDP, TCP, Bluetooth, etc.).
* **Discovery Protocol**: A method for agents to discover each other in a distributed network.
* **CBBA Algorithm**: Implementation of the Consensus-Based Bundle Algorithm for collaborative decision-making among agents.
* **Embedded Testing**: Deployment and testing on embedded systems.
* **Security**: Integration of secure communication using the Noise protocol for encrypted data exchange.

Work in progess.

# Notes

1. Different agents may temporarily disagree on the alive set due to UDP loss and timing.

# CBBA Implemenation

```
let winners; // local winners
let bundle; // local bundle

// Add all task ids to the bundle
for each task_id in task_ids {
  bundle.insert(task_id);
}
// Initially all task ids are in the bundle, and
// all tasks owned by the local agent.
// Task ids are sorted by the corresponding task bid.
rebid_and_sort(bundle, winners)

loop for S seconds
  gossip = receive_gossip();
  was_changed = process_gossip(bundle, winners, gossip);
  
  if was_changed {
    rebid_and_sort(bundle, winners);
    new_gossip = winners.convert_to_gossip();
    send_gossip(new_gossip);
  }
end
```

```
fn rebid_and_sort(bundle, winners) {
  // Step 1
  tasks_with_bids = build_tasks_with_bids(bundle);
  
  // Step 2
  sort_by_bids(tasks_with_bids);
  
  // Step 3
  bundle.clear();
  for (task_id, bid) in tasks_with_bids {
      bundle.insert(task_id);
      now = timestamp();
      winners[task_id] = (task_id, agent_id, bid, now); // fresh timestamp
  }
}
```

```
// Build tasks, whose ids are in the bundle, with bids
fn build_tasks_with_bids(bundle) {
  map = HashMap::new();
  
  for task_id in bundle {
    task = get_task_by_id(task_id);
    map[task_id] = calculate_task_bid(task);
  }
  
  return map;
}
```

```
// Compare two candidates for a winner
fn comapre(remote, local) {

  if remote.bid > local.bid {
     return RemoteWins;
  }
  
  if remote.bid < local.bid {
    return LocalWins;
  }
  
  // Bids are equal.
  // Winner is the one with the fresher timestamp
  if remote.ts > local.ts {
    return RemoteWins;
  }
  
  if remote.ts < local.ts {
    return LocalWins;
  }
  
  // Both bids and timestamps are equal.
  // Break tie by agent id.
  if remote.agent_id < local.agent_id {
    return RemoteWins;
  }
  
  return LocalWins;
}
```

```
// Return true if there is a change in the bundle, false otherwise.
// `winners` is the local map of winners
// `bundle` is the local bundle of tasks

fn process_gossip(bundle, winners, gossip) {
  if agent_id == gossip.agent_id {
    return false; // no need to your own gossip
  }
  
  let bundle_changed = false;
  
  // Loop on winners received as the gossip
  for remote in gossip.winners {
    let task_id = remote.task_id;
    
    local = winners.get(task_id);
    
    if local is present {  
      let winner;
      
      let result = compare(remote, local);
      if result == LocalWins {
        winner = local;
      } else {
        winner = remote;
      }
      
      let was_in_bundle = bundle.contains?(task_id);
      if winner.agent_id == agent_id {
        // The local agent wins, so insert the task into the bundle.
        bundle.insert(task_id); 
      } else if was_in_bundle {
        // The local agent loses, so remove all tasks after the current one.
        bundle.truncate_after(task_id);
      }
      
      // Update the winners map with the new winner
      winners[task_id] = (task_id, winner.agent_id, winner.bid, winner.ts);
      
      let now_in_bundle = bundle.contains?(task_id);
      if was_in_bundle != now_in_bundle {
        bundle_changed = true;
      }
    }
    else {
      // Unexpected scenario:
      // Agent does not have a local winner for the task.
      // Potential inconsistency among the task stores.
      bundle.remove(task_id);
      winners[task_id] = (task_id, remote.agent_id, remote.bid, remote.ts);
      bundle_changed = true;
    }
  }
  
  bundle_changed
}
```
