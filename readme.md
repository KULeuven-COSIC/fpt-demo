# FPT: a Fixed-Point Accelerator for Torus Fully Homomorphic Encryption

FPT accelerates the TFHE bootstrapping operation on FPGA. 

This repository provides a demo of FPT's performance improvements over an encrypted version of [Conway's Game-of-Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) implemented with [TFHE-rs](https://github.com/zama-ai/tfhe-rs/). The code in this repo is a fork of TFHE-rs, extending it with the use of FPT and with the demos implemented in `/demo` folder.

The design of FPT is published at ACM CCS 2023. Please consider citing our work if you use or build upon the results in this repository.

<details>
<summary>ACM Ref</summary>
<br>
Michiel Van Beirendonck, Jan-Pieter D'Anvers, Furkan Turan, and Ingrid Verbauwhede. 2023. FPT: A Fixed-Point Accelerator for Torus Fully Homomorphic Encryption. In Proceedings of the 2023 ACM SIGSAC Conference on Computer and Communications Security (CCS '23). Association for Computing Machinery, New York, NY, USA, 741–755. https://doi.org/10.1145/3576915.3623159
</details>

<details>
<summary>BibTex</summary>
<br>





```
@inproceedings{10.1145/3576915.3623159,
author = {Van Beirendonck, Michiel and D'Anvers, Jan-Pieter and Turan, Furkan and Verbauwhede, Ingrid},
title = {FPT: A Fixed-Point Accelerator for Torus Fully Homomorphic Encryption},
year = {2023},
isbn = {9798400700507},
publisher = {Association for Computing Machinery},
address = {New York, NY, USA},
url = {https://doi-org.kuleuven.e-bronnen.be/10.1145/3576915.3623159},
doi = {10.1145/3576915.3623159},
abstract = {Fully Homomorphic Encryption (FHE) is a technique that allows computation on encrypted data. It has the potential to drastically change privacy considerations in the cloud, but high computational and memory overheads are preventing its broad adoption. TFHE is a promising Torus-based FHE scheme that heavily relies on bootstrapping, the noise-removal tool invoked after each encrypted logical/arithmetical operation.  We present FPT, a Fixed-Point FPGA accelerator for TFHE bootstrapping. FPT is the first hardware accelerator to heavily exploit the inherent noise present in FHE calculations. Instead of double or single-precision floating-point arithmetic, it implements TFHE bootstrapping entirely with approximate fixed-point arithmetic. Using an in-depth analysis of noise propagation in bootstrapping FFT computations, FPT is able to use noise-trimmed fixed-point representations that are up to 50\% smaller than prior implementations that prefer floating-point or integer FFTs.  FPT is built as a streaming processor inspired by traditional streaming DSPs: it instantiates directly cascaded high-throughput computational stages, with minimal control logic and routing networks. We explore different throughput-balanced compositions of streaming kernels with a user-configurable streaming width in order to construct a full bootstrapping pipeline. Our proposed approach allows 100\% utilization of arithmetic units and requires only small bootstrapping key cache, enabling an entirely compute-bound bootstrapping throughput of 1 BS / 35us. This is in stark contrast to the established classical CPU approach to FHE bootstrapping acceleration, which is typically constrained by memory and bandwidth. FPT is fully implemented and evaluated as a bootstrapping FPGA kernel for an Alveo U280 datacenter accelerator card. FPT achieves two to three orders of magnitude higher bootstrapping throughput than existing CPU-based implementations, and 2.5x higher throughput compared to recent ASIC emulation experiments.},
booktitle = {Proceedings of the 2023 ACM SIGSAC Conference on Computer and Communications Security},
pages = {741–755},
numpages = {15},
keywords = {hardware accelerator, fpga, tfhe, fully homomorphic encryption},
location = {<conf-loc>, <city>Copenhagen</city>, <country>Denmark</country>, </conf-loc>},
series = {CCS '23}
}
```
</details>

## Abstract

Fully Homomorphic Encryption (FHE) is a technique that allows computation on encrypted data. It has the potential to drastically change privacy considerations in the cloud, but high computational and memory overheads are preventing its broad adoption. TFHE is a promising Torus-based FHE scheme that heavily relies on bootstrapping, the noise-removal tool invoked after each encrypted logical/arithmetical operation.

We present FPT, a Fixed-Point FPGA accelerator for TFHE bootstrapping. FPT is the first hardware accelerator to heavily exploit the inherent noise present in FHE calculations. Instead of double or single-precision floating-point arithmetic, it implements TFHE bootstrapping entirely with approximate fixed-point arithmetic. Using an in-depth analysis of noise propagation in bootstrapping FFT computations, FPT is able to use noise-trimmed fixed-point representations that are up to 50% smaller than prior implementations that prefer floating-point or integer FFTs.

FPT is built as a streaming processor inspired by traditional streaming DSPs: it instantiates directly cascaded high-throughput computational stages, with minimal control logic and routing networks. We explore different throughput-balanced compositions of streaming kernels with a user-configurable streaming width in order to construct a full bootstrapping pipeline. Our proposed approach allows 100% utilization of arithmetic units and requires only small bootstrapping key cache, enabling an entirely compute-bound bootstrapping throughput of 1 BS / 35us. This is in stark contrast to the established classical CPU approach to FHE bootstrapping acceleration, which is typically constrained by memory and bandwidth.

FPT is fully implemented and evaluated as a bootstrapping FPGA kernel for an Alveo U280 datacenter accelerator card. FPT achieves two to three orders of magnitude higher bootstrapping throughput than existing CPU-based implementations, and 2.5 times higher throughput compared to recent ASIC emulation experiments.

## Demo of Conway's Game of Life

This demo (`/demos/game-of-life/`) runs the Game of Life over TFHE with or without FPGA acceleration.

The screen recording below shows the acceleration of FPT (on the right) over the native version on `TFHE-rs` (on the left). Note that, FPT's acceleration is more capable than seen in this video; however, a reduced parameter set is preferred here to have the software version update the frames in acceptable time.

https://github.com/KULeuven-COSIC/fpt-demo/assets/4849663/73bd6242-6e77-44d4-9287-ad7d17e965f7

In running Game of Life over TFHE, the server receives an encrypted initial board configuration with encrypted cell states. Update rules are translated into Boolean equations, which are calculated by the server using encrypted gate arithmetic. Updating a single cell state requires exactly 44 encrypted gate computations, disregarding the cheaper NOT gates that do not include a bootstrap. As a whole, the encrypted Game of Life consists of a mix of homomorphic AND, XOR, OR, and NOT gates. These operations and their parallel computation should help estimating FPT's performance on a variety of applications. In addition, this is an application which demonstrates the performance improvements live: the FPGA-accelerated board updates visually appear much quicker than the software counterpart.

## Running the Demo of Conway's Game of Life

To run the demo yourself, FPT is made available on [AWS F1 instances](https://aws.amazon.com/ec2/instance-types/f1/). As noted above, this demo version prefers a reduced parameter set to have the software version of the board update the frames in acceptable time.

The description of how to run this demo yourself is detailed in [`demos/readme.md`](demos/readme.md).

---

Michiel Van Beirendonck, Jan-Pieter D'Anvers, Furkan Turan, and Ingrid Verbauwhede. 2023. FPT: A Fixed-Point Accelerator for Torus Fully Homomorphic Encryption. In Proceedings of the 2023 ACM SIGSAC Conference on Computer and Communications Security (CCS '23). Association for Computing Machinery, New York, NY, USA, 741–755. https://doi.org/10.1145/3576915.3623159

