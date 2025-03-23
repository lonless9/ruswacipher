use anyhow::{Result, anyhow};
use rand::Rng;
use log::debug;

/// Virtual machine instruction set opcodes
#[derive(Debug, Clone, Copy)]
pub enum VMOpcode {
    // Stack operations
    Push = 0x01,
    Pop = 0x02,
    Dup = 0x03,
    Swap = 0x04,
    
    // Arithmetic operations
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Rem = 0x14,
    
    // Logical operations
    And = 0x20,
    Or = 0x21,
    Xor = 0x22,
    Not = 0x23,
    
    // Control flow
    Jump = 0x30,
    JumpIf = 0x31,
    Call = 0x32,
    Return = 0x33,
    
    // Memory operations
    Load = 0x40,
    Store = 0x41,
    
    // Special instructions
    Nop = 0xF0,
    Exit = 0xFF,
}

/// Get VM opcode from byte
pub fn from_byte(byte: u8) -> Option<VMOpcode> {
    match byte {
        0x01 => Some(VMOpcode::Push),
        0x02 => Some(VMOpcode::Pop),
        0x03 => Some(VMOpcode::Dup),
        0x04 => Some(VMOpcode::Swap),
        
        0x10 => Some(VMOpcode::Add),
        0x11 => Some(VMOpcode::Sub),
        0x12 => Some(VMOpcode::Mul),
        0x13 => Some(VMOpcode::Div),
        0x14 => Some(VMOpcode::Rem),
        
        0x20 => Some(VMOpcode::And),
        0x21 => Some(VMOpcode::Or),
        0x22 => Some(VMOpcode::Xor),
        0x23 => Some(VMOpcode::Not),
        
        0x30 => Some(VMOpcode::Jump),
        0x31 => Some(VMOpcode::JumpIf),
        0x32 => Some(VMOpcode::Call),
        0x33 => Some(VMOpcode::Return),
        
        0x40 => Some(VMOpcode::Load),
        0x41 => Some(VMOpcode::Store),
        
        0xF0 => Some(VMOpcode::Nop),
        0xFF => Some(VMOpcode::Exit),
        
        _ => None,
    }
}

/// Obfuscated instruction table
pub fn generate_obfuscated_opcode_map() -> [u8; 256] {
    let mut rng = rand::thread_rng();
    let mut map = [0u8; 256];
    
    // Initialize as sequential mapping
    for i in 0..256 {
        map[i] = i as u8;
    }
    
    // Fisher-Yates shuffle algorithm
    for i in (1..256).rev() {
        let j = rng.random_range(0..=i);
        map.swap(i, j);
    }
    
    // Ensure certain critical instructions can be mapped correctly, avoiding excessive obfuscation
    // For example, ensure the exit instruction is correctly mapped
    map[0xFF] = 0xFF;
    
    map
}

/// Virtual machine state
pub struct VMState {
    pub pc: usize,              // Program counter
    pub stack: Vec<i32>,        // Operand stack
    pub memory: Vec<u8>,        // Memory space
    pub call_stack: Vec<usize>, // Call stack
    pub running: bool,          // Running status
    pub result: Option<i32>,    // Execution result
}

impl VMState {
    /// Create a new VM state
    pub fn new(memory_size: usize) -> Self {
        VMState {
            pc: 0,
            stack: Vec::with_capacity(64),
            memory: vec![0; memory_size],
            call_stack: Vec::with_capacity(16),
            running: false,
            result: None,
        }
    }
    
    /// Reset VM state
    pub fn reset(&mut self) {
        self.pc = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.running = false;
        self.result = None;
        for byte in &mut self.memory {
            *byte = 0;
        }
    }
}

/// Virtual machine interpreter
pub struct VMInterpreter {
    pub state: VMState,
    opcode_map: [u8; 256],       // Instruction mapping table
    bytecode: Vec<u8>,           // VM bytecode
}

impl VMInterpreter {
    /// Create a new VM interpreter
    pub fn new(bytecode: Vec<u8>, memory_size: usize) -> Self {
        let opcode_map = generate_obfuscated_opcode_map();
        VMInterpreter {
            state: VMState::new(memory_size),
            opcode_map,
            bytecode,
        }
    }
    
    /// Load bytecode
    pub fn load_bytecode(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
        self.state.reset();
    }
    
    /// Decode instruction
    fn decode(&self, opcode: u8) -> Option<VMOpcode> {
        // Decode opcode through mapping table
        let mapped_opcode = self.opcode_map[opcode as usize];
        from_byte(mapped_opcode)
    }
    
    /// Execute VM
    pub fn execute(&mut self) -> Result<i32> {
        debug!("Starting VM execution, bytecode size: {}", self.bytecode.len());
        
        self.state.running = true;
        self.state.pc = 0;
        
        while self.state.running && self.state.pc < self.bytecode.len() {
            // Get current instruction
            let opcode = self.bytecode[self.state.pc];
            self.state.pc += 1;
            
            // Decode and execute instruction
            if let Some(instruction) = self.decode(opcode) {
                self.execute_instruction(instruction)?;
            } else {
                debug!("Unknown opcode: 0x{:02X}", opcode);
                // Ignore unknown instructions, continue execution
            }
        }
        
        // Get execution result
        match self.state.result {
            Some(result) => Ok(result),
            None if !self.state.stack.is_empty() => Ok(self.state.stack.pop().unwrap()),
            None => Err(anyhow!("VM execution completed, but no return value")),
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: VMOpcode) -> Result<()> {
        match instruction {
            VMOpcode::Push => {
                // Read value to push from bytecode
                if self.state.pc < self.bytecode.len() {
                    let value = self.bytecode[self.state.pc] as i32;
                    self.state.pc += 1;
                    self.state.stack.push(value);
                }
            },
            VMOpcode::Pop => {
                if let Some(_) = self.state.stack.pop() {
                    // Value popped
                }
            },
            VMOpcode::Dup => {
                if let Some(&value) = self.state.stack.last() {
                    self.state.stack.push(value);
                }
            },
            VMOpcode::Swap => {
                let len = self.state.stack.len();
                if len >= 2 {
                    self.state.stack.swap(len - 1, len - 2);
                }
            },
            VMOpcode::Add => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a + b);
                }
            },
            VMOpcode::Sub => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a - b);
                }
            },
            VMOpcode::Mul => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a * b);
                }
            },
            VMOpcode::Div => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    if b == 0 {
                        return Err(anyhow!("VM execution error: division by zero"));
                    }
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a / b);
                }
            },
            VMOpcode::Rem => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    if b == 0 {
                        return Err(anyhow!("VM execution error: modulo zero"));
                    }
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a % b);
                }
            },
            VMOpcode::And => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a & b);
                }
            },
            VMOpcode::Or => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a | b);
                }
            },
            VMOpcode::Xor => {
                if self.state.stack.len() >= 2 {
                    let b = self.state.stack.pop().unwrap();
                    let a = self.state.stack.pop().unwrap();
                    self.state.stack.push(a ^ b);
                }
            },
            VMOpcode::Not => {
                if let Some(value) = self.state.stack.pop() {
                    self.state.stack.push(!value);
                }
            },
            VMOpcode::Jump => {
                // Read jump target from bytecode
                if self.state.pc + 1 < self.bytecode.len() {
                    let target = ((self.bytecode[self.state.pc] as usize) << 8) | 
                                  (self.bytecode[self.state.pc + 1] as usize);
                    self.state.pc = target;
                } else {
                    self.state.pc += 2; // Skip operand
                }
            },
            VMOpcode::JumpIf => {
                // Conditional jump
                if let Some(condition) = self.state.stack.pop() {
                    if condition != 0 {
                        // Read jump target from bytecode
                        if self.state.pc + 1 < self.bytecode.len() {
                            let target = ((self.bytecode[self.state.pc] as usize) << 8) | 
                                          (self.bytecode[self.state.pc + 1] as usize);
                            self.state.pc = target;
                            return Ok(());
                        }
                    }
                }
                self.state.pc += 2; // Skip operand
            },
            VMOpcode::Call => {
                // Subprogram call
                if self.state.pc + 1 < self.bytecode.len() {
                    let target = ((self.bytecode[self.state.pc] as usize) << 8) | 
                                  (self.bytecode[self.state.pc + 1] as usize);
                    self.state.pc += 2; // Skip operand
                    self.state.call_stack.push(self.state.pc);
                    self.state.pc = target;
                } else {
                    self.state.pc += 2; // Skip operand
                }
            },
            VMOpcode::Return => {
                // Return from call stack
                if let Some(return_addr) = self.state.call_stack.pop() {
                    self.state.pc = return_addr;
                } else {
                    // Main program return, set result and stop
                    if let Some(result) = self.state.stack.pop() {
                        self.state.result = Some(result);
                    }
                    self.state.running = false;
                }
            },
            VMOpcode::Load => {
                // Load value from memory
                if let Some(address) = self.state.stack.pop() {
                    let address = address as usize;
                    if address < self.state.memory.len() {
                        let value = self.state.memory[address] as i32;
                        self.state.stack.push(value);
                    } else {
                        return Err(anyhow!("VM execution error: memory access out of bounds"));
                    }
                }
            },
            VMOpcode::Store => {
                // Store value to memory
                if self.state.stack.len() >= 2 {
                    let value = self.state.stack.pop().unwrap();
                    let address = self.state.stack.pop().unwrap() as usize;
                    if address < self.state.memory.len() {
                        self.state.memory[address] = value as u8;
                    } else {
                        return Err(anyhow!("VM execution error: memory access out of bounds"));
                    }
                }
            },
            VMOpcode::Nop => {
                // No operation
            },
            VMOpcode::Exit => {
                // Set result and stop
                if let Some(exit_code) = self.state.stack.pop() {
                    self.state.result = Some(exit_code);
                }
                self.state.running = false;
            },
        }
        
        Ok(())
    }
}

pub fn initialize_vm_stack(size: usize) -> Result<Vec<i32>> {
    // 创建虚拟机栈
    let mut stack = vec![0; size];
    let mut rng = rand::thread_rng();
    
    // 随机填充栈底几个元素，增加混淆
    for i in 0..4 {
        stack[i] = rng.random_range(-1000..1000);
    }
    
    Ok(stack)
} 