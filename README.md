# BGP Router Project

## Overview

This project is an implementation of a simple BGP router, designed to give a practical understanding of how core internet infrastructure operates. It involves building and managing forwarding tables, generating route announcements, forwarding data packets, and handling multiple sockets. The router is tested in a simulator environment, which mimics real-world BGP operations including peer connections and route updates.

## Technical Details

The router has been implemented to manage UDP sockets corresponding to BGP peers. It listens for incoming BGP messages, updates its routing table based on received announcements, and forwards data packets accordingly. The implementation also includes mechanisms for route `aggregation` and `disaggregation` to maintain an efficient and compressed forwarding table.

## High-Level Approach

1. **Initial Analysis**: We started by analyzing test cases to understand network topologies, crucial for planning our routing strategies.

2. **Socket Setup**: We established Unix domain sockets.

3. **Handling Basic Messages**: Next, we focused on processing "update" and "data" messages, creating a basic forwarding table for routing.

4. **Implementing Dump Responses**: Adding "dump" response capabilities allowed our router to pass initial tests, demonstrating its reporting functions.

5. **Withdraw Message Handling**: Supporting "withdraw" messages and "no route" scenarios increased our router's adaptability to changing network conditions.

6. **Enforcing BGP Relationships**: We applied logic to respect BGP peering and provider/customer relationships, ensuring economic and policy compliance in message forwarding.

7. **Aggregation/Disaggregation**: Finally, we tackled route aggregation and disaggregation, optimizing our routing table for efficiency and accuracy.

## Challenges Encountered

**Route Aggregation and Disaggregation**: 

Implementing route aggregation and disaggregation presented unique challenges. For disaggregation,  instead of storing every update and withdrawal message, we employed a mathematical approach to dynamically adjust our routing table. We spent some time understanding the BGP protocol and how to handle route aggregation and disaggregation. This strategy not only enhanced our understanding of routing complexities but also significantly conserved space in our routing table, making our router more efficient.

## Testing Strategy

The router was tested against a series of predefined configurations provided in the simulator. We start by testing the basic functionality of the router, including the ability to establish connections with peers, receive and process route announcements, and forward data packets. We then test the router's ability to handle route aggregation and disaggregation, and its ability to maintain an efficient routing table. 

## Lessons Learned

1. **Adherence to BGP Policies**: Our project underscored the critical importance of adhering to BGP policies. These policies are not mere suggestions but foundational elements that ensure the efficient and reliable flow of internet traffic. Our diligent adherence to these guidelines ensured our router contributed effectively to the global network ecosystem, facilitating seamless data transmission.

2. **Aggregation and Disaggregation**: Delving into route aggregation and disaggregation proved to be transformative. It challenged us to rethink our approach to routing table management, aiming not just for efficiency in storage but in the strategic movement of data across the web.
