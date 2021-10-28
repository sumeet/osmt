use eframe::{egui, epi};
use iced_x86;

// pattern match, and convert from iced_x86::Instruction into Instruction, and error if we don't have a match
fn from_iced(ii: iced_x86::Instruction) -> anyhow::Result<Instruction> {
    let mnemonic = ii.mnemonic();
}




enum Instruction {
    Mov(Dest, Src),
    Int(u8),
}

enum Dest {
    Register(Register),
}

enum Register {
    AX,
    AL,
    AH,
}

pub fn simulate(instructions: Vec<Instruction>) {
    let options = eframe::NativeOptions::default();
    let simulator = Simulator::new(instructions);
    eframe::run_native(Box::new(simulator), options);
}

struct Simulator {
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
        }
    }

    fn simulate_one(&mut self) {
        match self.current_instruction() {
            &_ => {}
        }
    }

    fn current_instruction(&self) -> &Instruction {
        &self.instructions[self.ip]
    }
}

impl epi::App for Simulator {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.simulate_one();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.text_buffer)
        });
        frame.set_window_size(ctx.used_size());
    }

    fn name(&self) -> &str {
        "oSMT simulator"
    }
}