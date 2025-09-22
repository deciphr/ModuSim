# ModuSim

<table align="center">
    <tr>
        <td>
            <img src="./docs/demo.png" height="200px" />
        </td>
    </tr>
</table>

A conceptual Industrial Control System (ICS) simulation written in Rust, using the [Modbus protocol](https://github.com/slowtec/tokio-modbus) to model real-world industrial plants. Inspired by [VirtuaPlant](https://github.com/jseidl/virtuaplant).

## Installation

```bash
git clone https://github.com/deciphr/ModuSim
cd ModuSim
cargo run
```

## Usage

When the simulation window loads, here are the keybinds to manually control the plant:

| Key        | Description                  |
| ------------ | ------------------------------ |
| Space      | Start/stop the conveyor belt |
| Up Arrow   | Increase conveyor belt speed |
| Down Arrow | Decrease conveyor belt speed |
| Enter      | Spawn a bottle               |
| V          | Open/close water valve       |

To manipulate the plant via Modbus, connect to port `5502`. This can be modified in `src/components/modbus.rs`.

## License

Copyright (C) 2025 deciphr

[GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/)
