# Linux host adapter

The future Linux process, socket, memory-accounting, and executable entry
adapter belongs in this directory. It will compile the controller in
`src/controller.rs`; WebKitGTK-specific browser integration remains in its worker.

WPE should reuse this host adapter when its OS primitives are the same.
