# Agent Project

This project is a framework for building and testing multi-agent systems. For now I am planning to include:

* **Transport Layer**: A flexible communication interface supporting various protocols (UDP, TCP, Bluetooth, etc.).
* **Discovery Protocol**: A method for agents to discover each other in a distributed network.
* **CBBA Algorithm**: Implementation of the Consensus-Based Bundle Algorithm for collaborative decision-making among agents.
* **Embedded Testing**: Deployment and testing on embedded systems.
* **Security**: Integration of secure communication using the Noise protocol for encrypted data exchange.


Currently I have been impementing discovery on the embedded system - ESP32-C6.
I am also planning to read about the convergence time (*Consensus-Based Decentralized Auctions for Robust Task Allocation*) and choose CBBA timeout time wisely. Integrating encrypted transport is also possible. I am also planing to study and implement other swarm algorithms for autonimous agents.
You can find the related article [here](https://fade2black.github.io/blog/cbba/).
