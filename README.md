# Windows Driver Rust Framework

## Overview

The `windows-driver-rust-framework` is designed to help users write safe Windows kernel code using Rust. This project is part of a bachelor's thesis aimed at improving the safety and reliability of Windows driver development.

## Project Structure

The repository is organized into a folder called `crates`, which contains four distinct crates:

1. **wdrf-std**
    - Implements some of the standard classes.
    - Includes event and semaphore from the Windows kernel.

2. **wdrf-framework**
    - Provides useful structs for Windows driver development.

3. **test-driver**
    - A test driver for validating and testing the framework.

4. **maple**
    - A simple logging crate to facilitate logging within the drivers.

## License

This project is licensed under the Apache2 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project is part of my bachelor's thesis.
