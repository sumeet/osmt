use std::fs::File;
use std::io::Write;
use std::iter::repeat;
use iced_x86::code_asm::CodeAssembler;

fn fill(v: &mut Vec<u8>, n: usize, with: u8) {
    if v.len() < n {
        v.extend(repeat(with).take(n - v.len()));
    }
}

fn gen_mbr(mut a: CodeAssembler) -> anyhow::Result<Vec<u8>> {
    let mut bs = a.assemble(0)?;
    fill(&mut bs, 512, 0);
    // holy shit, copilot filled in these two lines
    bs[510] = 0x55;
    bs[511] = 0xAA;
    Ok(bs)
}

fn mbr_instructions() -> anyhow::Result<CodeAssembler> {
    use iced_x86::code_asm::*;

    let mut a = CodeAssembler::new(64)?;
    let mut beginning = a.create_label();
    a.set_label(&mut beginning)?;
    a.jmp(beginning)?;
    Ok(a)
}

fn print_hex(bytes: &[u8]) {
    for b in bytes {
        print!("{:02x} ", b);
    }
}

fn main() -> anyhow::Result<()> {
    let a = mbr_instructions()?;
    dbg!(a.instructions());
    let bs = gen_mbr(a)?;
    print_hex(&bs);
    File::create("boot.bin")?.write_all(&bs)?;
    Ok(())
}
