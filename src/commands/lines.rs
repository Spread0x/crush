use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::path::Path;
use lazy_static::lazy_static;
use crate::{
    commands::command_util::find_field,
    errors::{JobError, argument_error},
    commands::{Call, Exec},
    data::{
        Argument,
        Row,
        CellFnurp,
        CellType,
        JobOutput,
        Cell,
    },
    stream::{OutputStream, InputStream, unlimited_streams},
};
use crate::printer::Printer;
use crate::env::Env;

lazy_static! {
    static ref sub_type: Vec<CellType> = {
        vec![CellFnurp::named("line", CellType::Text)]
    };
}

fn handle(file: Box<Path>, output: &mut OutputStream) -> Result<(), JobError> {
    let (output_stream, input_stream) = unlimited_streams();
    let out_row = Row {
        cells: vec![
            Cell::File(file.clone()),
            Cell::JobOutput(JobOutput {
                types: sub_type.clone(),
                stream: input_stream,
            }),
        ],
    };
    output.send(out_row)?;
    let file_copy = file.clone();
    thread::spawn(move || {
        let fff = File::open(file).unwrap();
        let mut reader = BufReader::new(&fff);
        let mut line = String::new();
        loop {
            reader.read_line(&mut line);
            if line.is_empty() {
                break;
            }
            output_stream.send(Row { cells: vec![Cell::Text(line[0..line.len() - 1].to_string().into_boxed_str())] });
            line.clear();
        }
    });
    return Ok(());
}


pub fn run(
    input_type: Vec<CellFnurp>,
    mut arguments: Vec<Argument>,
    input: InputStream,
    mut output: OutputStream,
    env: Env,
    printer: Printer,
) -> Result<(), JobError> {
    let mut files: Vec<Box<Path>> = Vec::new();
    if input_type.len() == 0 {
        for arg in &arguments {
            arg.cell.file_expand(&mut files)?;
        }
        for file in files {
            handle(file, &mut output)?;
        }
    } else {
        if arguments.len() != 1 {
            return Err(argument_error("Expected one argument: column spec"));
        }
        match &arguments[0].cell {
            Cell::Text(s) | Cell::Field(s) => {
                let idx = find_field(&s, &input_type)?;
                loop {
                    match input.recv() {
                        Ok(row) => {
                            let mut files: Vec<Box<Path>> = Vec::new();
                            row.cells[idx].file_expand(&mut files)?;
                            for file in files {
                                handle(file, &mut output)?;
                            }
                        },
                        Err(_) => break,
                    }
                }
            }
            _ => return Err(argument_error("Expected column of type Field")),
        }
    }
    return Ok(());
}

pub fn compile(input_type: Vec<CellFnurp>, input: InputStream, output: OutputStream, arguments: Vec<Argument>) -> Result<(Exec, Vec<CellFnurp>), JobError> {
    let output_type: Vec<CellFnurp> =
        vec![
            CellFnurp::named("file", CellType::File),
            CellFnurp::named("lines", CellType::Output(sub_type.clone())),
        ];

    return Ok(Call {
        name: String::from("lines"),
        input_type,
        arguments,
        output_type,
        exec: Exec::Command(run),
    });
}