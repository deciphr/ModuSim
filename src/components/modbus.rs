// modbus.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

// Bevy implementation of: https://github.com/slowtec/tokio-modbus/blob/main/examples/tcp-server.rs
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use tokio::net::TcpListener;
use tokio_modbus::{
    prelude::*,
    server::tcp::{Server, accept_tcp_connection},
};

const MODBUS_IP: &str = "0.0.0.0";
const MODBUS_PORT: u16 = 5502;

pub struct ModbusPlugin;

impl Plugin for ModbusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ModbusState::default())
            .add_systems(Startup, start_modbus_server);
    }
}

#[derive(Resource, Default, Clone)]
pub struct ModbusState {
    pub coils: Arc<Mutex<HashMap<u16, bool>>>,
    pub discrete_inputs: Arc<Mutex<HashMap<u16, bool>>>,
    pub input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    pub holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl ModbusState {
    pub fn default() -> Self {
        Self {
            coils: Arc::new(Mutex::new(HashMap::new())),
            discrete_inputs: Arc::new(Mutex::new(HashMap::new())),
            input_registers: Arc::new(Mutex::new(HashMap::new())),
            holding_registers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct BevyService {
    state: ModbusState,
}

impl tokio_modbus::server::Service for BevyService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let res = match req {
            Request::ReadCoils(addr, cnt) => {
                let coils = self.state.coils.lock().unwrap();
                discrete_read(&coils, addr, cnt).map(Response::ReadCoils)
            }
            Request::WriteSingleCoil(addr, value) => {
                let mut coils = self.state.coils.lock().unwrap();
                coil_write(&mut coils, addr, std::slice::from_ref(&value))
                    .map(|_| Response::WriteSingleCoil(addr, value))
            }
            Request::ReadDiscreteInputs(addr, cnt) => {
                let discrete_inputs = self.state.discrete_inputs.lock().unwrap();
                discrete_read(&discrete_inputs, addr, cnt).map(Response::ReadDiscreteInputs)
            }
            Request::ReadInputRegisters(addr, cnt) => {
                let input_registers = self.state.input_registers.lock().unwrap();
                register_read(&input_registers, addr, cnt).map(Response::ReadInputRegisters)
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                let holding_registers = self.state.holding_registers.lock().unwrap();
                register_read(&holding_registers, addr, cnt).map(Response::ReadHoldingRegisters)
            }
            Request::WriteMultipleRegisters(addr, values) => {
                let mut holding_registers = self.state.holding_registers.lock().unwrap();
                register_write(&mut holding_registers, addr, &values)
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16))
            }
            Request::WriteSingleRegister(addr, value) => {
                let mut holding_registers = self.state.holding_registers.lock().unwrap();
                register_write(&mut holding_registers, addr, std::slice::from_ref(&value))
                    .map(|_| Response::WriteSingleRegister(addr, value))
            }
            _ => {
                println!(
                    "SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}"
                );
                Err(ExceptionCode::IllegalFunction)
            }
        };
        std::future::ready(res)
    }
}

fn discrete_read(bools: &HashMap<u16, bool>, addr: u16, cnt: u16) -> Result<Vec<bool>, ExceptionCode> {
    for reg_addr in addr..addr + cnt {
        if !bools.contains_key(&reg_addr) {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(ExceptionCode::IllegalDataAddress);
        }
    }
    Ok((addr..addr + cnt)
        .map(|reg_addr| bools[&reg_addr])
        .collect())
}

fn coil_write(
    coils: &mut HashMap<u16, bool>,
    addr: u16,
    values: &[bool],
) -> Result<(), ExceptionCode> {
    for i in 0..values.len() {
        let reg_addr = addr + i as u16;
        if !coils.contains_key(&reg_addr) {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(ExceptionCode::IllegalDataAddress);
        }
    }

    for (i, &value) in values.iter().enumerate() {
        let reg_addr = addr + i as u16;
        coils.insert(reg_addr, value);
    }

    Ok(())
}
fn register_read(
    registers: &HashMap<u16, u16>,
    addr: u16,
    cnt: u16,
) -> Result<Vec<u16>, ExceptionCode> {
    for reg_addr in addr..addr + cnt {
        if !registers.contains_key(&reg_addr) {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(ExceptionCode::IllegalDataAddress);
        }
    }

    Ok((addr..addr + cnt)
        .map(|reg_addr| registers[&reg_addr])
        .collect())
}

fn register_write(
    registers: &mut HashMap<u16, u16>,
    addr: u16,
    values: &[u16],
) -> Result<(), ExceptionCode> {
    for i in 0..values.len() {
        let reg_addr = addr + i as u16;
        if !registers.contains_key(&reg_addr) {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(ExceptionCode::IllegalDataAddress);
        }
    }

    for (i, &value) in values.iter().enumerate() {
        let reg_addr = addr + i as u16;
        registers.insert(reg_addr, value);
    }

    Ok(())
}

fn start_modbus_server(modbus_state: Res<ModbusState>) {
    let state = modbus_state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let socket_addr: SocketAddr = format!("{}:{}", MODBUS_IP, MODBUS_PORT).parse().unwrap();
            let listener = TcpListener::bind(socket_addr).await.unwrap();
            let server = Server::new(listener);
            let new_service = |_addr| {
                Ok(Some(BevyService {
                    state: state.clone(),
                }))
            };
            let on_connected = |stream, socket_addr| async move {
                accept_tcp_connection(stream, socket_addr, new_service)
            };
            let on_process_error = |err| eprintln!("{err}");
            println!("Modbus server running on {socket_addr}");
            let _ = server.serve(&on_connected, on_process_error).await;
        });
    });
}
