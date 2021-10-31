use std::iter::once;
use std::mem::MaybeUninit;

use anyhow::bail;
use arrayvec::ArrayVec;
use eframe::{egui, epi};
use iced_x86;
use iced_x86::{Mnemonic, OpKind};

#[allow(unused)]
pub fn simulate(instructions: &[iced_x86::Instruction]) -> anyhow::Result<()> {
    let instructions = instructions
        .iter()
        .map(from_iced)
        .collect::<Result<_, _>>()?;

    let options = eframe::NativeOptions::default();
    let simulator = Simulator::new(instructions);
    eframe::run_native(Box::new(simulator), options);
}

fn get_operands<const N: usize>(inst: &iced_x86::Instruction) -> anyhow::Result<[Operand; N]> {
    let mut operands: [MaybeUninit<Operand>; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for n in 0..N {
        let operand = match inst.try_op_kind(n as _)? {
            OpKind::Register => Operand::Register(inst.try_op_register(n as _)?),
            OpKind::Immediate8
            | OpKind::Immediate8_2nd
            | OpKind::Immediate16
            | OpKind::Immediate32
            | OpKind::Immediate64
            | OpKind::Immediate8to16
            | OpKind::Immediate8to32
            | OpKind::Immediate8to64
            | OpKind::Immediate32to64 => Operand::Immediate(inst.try_immediate(n as _)?),
            otherwise => bail!("unsupported operand {:?}", otherwise),
        };
        operands[n].write(operand);
    }
    Ok(unsafe { MaybeUninit::array_assume_init(operands) })
}

// pattern match, and convert from iced_x86::Instruction into Instruction, and error if we don't have a match
fn from_iced(ii: &iced_x86::Instruction) -> anyhow::Result<Instruction> {
    let mnemonic = ii.mnemonic();
    match mnemonic {
        Mnemonic::Mov => {
            let [op1, op2] = get_operands(&ii)?;
            Ok(Instruction::Mov(op1, op2))
        }
        Mnemonic::Int => {
            let [op1] = get_operands(&ii)?;
            Ok(Instruction::Int(op1))
        }
        otherwise => bail!("unsupported instruction: {:?}", otherwise),
    }
}

#[derive(Clone, Copy)]
enum Instruction {
    Mov(Operand, Operand),
    Int(Operand),
}

#[derive(Clone, Copy)]
enum Operand {
    Register(iced_x86::Register),
    Immediate(u64),
}

#[allow(unused)]
struct Registers {
    rax: [u8; 8],
    rbx: [u8; 8],
    rcx: [u8; 8],
    rdx: [u8; 8],
    rsi: [u8; 8],
    rdi: [u8; 8],
    rsp: [u8; 8],
    rbp: [u8; 8],
}

impl Registers {
    fn new() -> Self {
        Self {
            rax: [0; 8],
            rbx: [0; 8],
            rcx: [0; 8],
            rdx: [0; 8],
            rsi: [0; 8],
            rdi: [0; 8],
            rsp: [0; 8],
            rbp: [0; 8],
        }
    }
}

struct Simulator {
    registers: Registers,
    text_buffer: String,
    instructions: Vec<Instruction>,
    ip: usize,
}

impl Simulator {
    fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            ip: 0,
            text_buffer: String::new(),
            registers: Registers::new(),
        }
    }

    fn simulate_one(&mut self) -> anyhow::Result<()> {
        match self.current_instruction() {
            Instruction::Mov(dest, src) => {
                let src = self.get_src(src)?;
                let dest = self.get_dest(dest)?;
                for (src, dest) in src.iter().zip(dest) {
                    *dest = *src;
                }
                self.ip += 1;
            }
            Instruction::Int(_) => todo!(),
        }
        Ok(())
    }

    fn current_instruction(&self) -> Instruction {
        self.instructions[self.ip]
    }

    pub fn get_dest(&mut self, op: Operand) -> anyhow::Result<&mut [u8]> {
        match op {
            Operand::Register(reg) => match reg {
                iced_x86::Register::RAX => Ok(&mut self.registers.rax),
                iced_x86::Register::AH => Ok(&mut self.registers.rax[7..=7]),
                iced_x86::Register::AL => Ok(&mut self.registers.rax[6..=6]),
                iced_x86::Register::RBX => Ok(&mut self.registers.rbx),
                iced_x86::Register::RCX => Ok(&mut self.registers.rcx),
                iced_x86::Register::RDX => Ok(&mut self.registers.rdx),
                iced_x86::Register::RSI => Ok(&mut self.registers.rsi),
                iced_x86::Register::RDI => Ok(&mut self.registers.rdi),
                iced_x86::Register::RSP => Ok(&mut self.registers.rsp),
                iced_x86::Register::RBP => Ok(&mut self.registers.rbp),
                otherwise => bail!("unsupported register {:?}", otherwise),
            },
            Operand::Immediate(_) => bail!("can't write into immediate"),
        }
    }

    pub fn get_src(&self, op: Operand) -> anyhow::Result<ArrayVec<u8, 8>> {
        Ok(match op {
            Operand::Register(reg) => match reg {
                iced_x86::Register::RAX => self.registers.rax.into(),
                iced_x86::Register::AH => ArrayVec::from_iter(once(self.registers.rax[7])),
                iced_x86::Register::AL => ArrayVec::from_iter(once(self.registers.rax[6])),
                iced_x86::Register::RBX => self.registers.rbx.into(),
                iced_x86::Register::RCX => self.registers.rcx.into(),
                iced_x86::Register::RDX => self.registers.rdx.into(),
                iced_x86::Register::RSI => self.registers.rsi.into(),
                iced_x86::Register::RDI => self.registers.rdi.into(),
                iced_x86::Register::RSP => self.registers.rsp.into(),
                iced_x86::Register::RBP => self.registers.rbp.into(),
                otherwise => bail!("unsupported register {:?}", otherwise),
            },
            Operand::Immediate(imm) => imm.to_le_bytes().into(),
        })
    }
}

impl epi::App for Simulator {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // TODO: more sophisticated than one instruction per frame
        self.simulate_one().unwrap(); // TODO: show error somewhere
        egui::CentralPanel::default().show(ctx, |ui| ui.label(&self.text_buffer));
        frame.set_window_size(ctx.used_size());
    }

    fn name(&self) -> &str {
        "oSMT simulator"
    }
}
