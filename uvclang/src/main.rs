#![allow(dead_code)]

mod ast;
mod codegen;
mod frontend;
mod layout;
mod lexer;
mod parser;
mod syscalls;

use itertools::Itertools; 
use tempfile::Builder;

use std::io::Write;
use std::process::exit;

use codegen::Codegen;
use frontend::FrontendOpts;
use lexer::{Lexer, ParseError};
use parser::Parser;

const USAGE: &str = "usage: uvclang <input.c|.ll> [-o out.asm] [-O0|-O1|-O2|-O3]\n       \
                     [-D<macro>] [-I<dir>] [-U<macro>] [-Wall|-W<warn>] [-std=<std>]\n       \
                     [-f<feature>] [--emit-ir] [--stats]";

/// URL for reporting LLVM IR uvclang cannot parse yet.
const ISSUE_URL: &str = "https://github.com/maximecb/uvm/issues/new";

static STDIO_C_CONTENT: &str = include_str!("../lib_src/stdio.c");
static NEW_OPERATOR_DEF: &str = r#"
 #include <stdlib.h>   // malloc/free
 #include <assert.h>

 void* operator new(size_t size)   { if (size == 0) size = 1; return malloc(size); }
 void* operator new[](size_t size) { if (size == 0) size = 1; return malloc(size); }
 void  operator delete(void* p) noexcept              { free(p); }
 void  operator delete(void* p, size_t) noexcept      { free(p); }
 void  operator delete[](void* p) noexcept            { free(p); }
 void  operator delete[](void* p, size_t) noexcept    { free(p); }

 extern "C" void __cxa_pure_virtual() { assert(0); }
 "#;

fn main()
{
    // Args: <input.c|.ll> [-o out.asm] [-O<n>] [-D..] [-I..] [--emit-ir] [--stats]
    // A `.ll` input is parsed directly (the back-end path, unchanged); any other
    // source is first lowered to IR by clang (the Phase 9 front-end).
    let args: Vec<String> = std::env::args().collect();
    let mut inputs: Vec<String> = vec![];
    let mut out_path: Option<String> = None;
    let mut stats = false;
    let mut emit_ir = false;
    let mut fe = FrontendOpts::default();

    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--stats" => stats = true,
            "--emit-ir" | "--emit-llvm" => emit_ir = true,
            "-o" => {
                i += 1;
                out_path = args.get(i).cloned();
            }
            // Optimization level: forwarded to clang; ignored for `.ll` input.
            "-O0" | "-O1" | "-O2" | "-O3" | "-Os" | "-Oz" => fe.opt_level = a.to_string(),
            // -D / -I / -U, both the joined (`-DFOO`) and split (`-D FOO`) forms.
            "-D" | "-I" | "-U" => {
                if let Some(v) = args.get(i + 1) {
                    fe.passthrough.push(format!("{}{}", a, v));
                    i += 1;
                }
            }
            _ if a.starts_with("-D") || a.starts_with("-I") || a.starts_with("-U") => {
                fe.passthrough.push(a.to_string());
            }
            // Common clang diagnostic / language / feature flags (e.g. `-Wall`,
            // `-std=c11`, `-ffast-math`) forwarded verbatim to the front-end.
            _ if is_clang_passthrough(a) => fe.passthrough.push(a.to_string()),
            _ if a.starts_with('-') => {
                eprintln!("uvclang: unknown option \"{}\"\n{}", a, USAGE);
                exit(2);
            }
            s => {
                inputs.push(s.to_string());
            }
        }
        i += 1;
    }

    if inputs.is_empty() {
        eprintln!("{}", USAGE);
        exit(2);
    };

    let irs : Vec<Result<(String, String), String>> = inputs
    .into_iter()
    .map(|input| -> Result<(String, String), String> {
        // Front-end: obtain textual LLVM IR either straight from a `.ll` file or by
        // driving clang on a C/C++ source in-process (no temp `.ll` on disk).
        if frontend::is_ir_path(&input) {
            match std::fs::read_to_string(&input) {
                Ok(src) => Ok((src, input.clone())),
                Err(e) => Err(e.to_string())
            }
        } else {
            let mut this_fe = fe.clone();
            this_fe.is_cpp = frontend::is_cpp_path(&input);
            match frontend::compile_to_ir(&input, &this_fe) {
                Ok(ir) => Ok((ir, input.clone())),
                Err(e) => Err(e)
            }
        }
        
    })
    .collect();

    let (successes, errors): (Vec<(String, String)>, Vec<String>) = irs.into_iter().partition_result();

    if !errors.is_empty() {
        eprintln!("parse error: {}", errors.join("\n"));
        exit(1);
    }
    // --emit-ir: dump the front-end IR and stop (handy for growing C coverage).
    if emit_ir {
        if successes.len() != 1 {
            eprintln!(
                "uvclang: --emit-ir only supports a single input file (got {}); \
                 invoke uvclang separately for each file",
                successes.len()
            );
            exit(2);
        }
        let (ir, _) = successes.into_iter().next().unwrap();
        emit(&out_path, ir);
        return;
    }

    let modules : Result<Vec<ast::Module>, String> = successes.into_iter()
        .map(|(ir, ir_name)| {
            match parse(&ir, &ir_name) {
                Ok(m) => Ok(m),
                Err(e) => Err(e.to_string())
            }
        })
        .collect();

    let modules = match modules {
        Ok(a) => a,
        Err(e) => {
            eprintln!("module error: {}", e);
            print_unsupported_ir_hint();
            exit(1);
        }
    };
    
    if stats {
        for module in &modules {
            let path = module.source_filename.as_deref().unwrap_or("unknown module");
            print_stats(path, module);
        }
        return;
    }

    // Compute runtime modules
    let runtime_srcs_c = vec![
        ("stdio.c", STDIO_C_CONTENT),
        ("new.cpp", NEW_OPERATOR_DEF),
        ];

    let runtime_ir_c : Vec<(String, String)> = runtime_srcs_c
        .into_iter()
        .map(|(file_name, file_content)| -> (String, String) {
            let mut src_file = Builder::new()
                .suffix(file_name)
                .tempfile()
                .unwrap_or_else(|e| {
                    eprintln!("failed to create temp file for {}: {}", file_name, e);
                    exit(1)
                });
            if let Err(e) = writeln!(src_file, "{}", file_content) {
                eprintln!("failed to write temp file for {}: {}", file_name, e);
                exit(1);
            }

            let mut this_fe = fe.clone();
            this_fe.is_cpp = frontend::is_cpp_path(&file_name);
            let src_path = src_file.path().to_str().expect("temp path is not valid UTF-8");
            match frontend::compile_to_ir(src_path, &this_fe) {
                Ok(ir) => (ir, format!("runtime_{}", file_name)),
                Err(e) => {
                    eprintln!("runtime module error: {}", e);
                    exit(1)
                }
            }        
    })
    .collect();

    let runtime_modules : Vec<ast::Module> = runtime_ir_c
        .into_iter()
        .map(|(ir, ir_name)| {
            match parse(&ir, &ir_name) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("runtime module error: {}", e);
                    exit(1)
                }
            }
        })
        .collect();

    let all_modules: Vec<ast::Module> = modules.into_iter().chain(runtime_modules).collect();
    let asm = match Codegen::new(&all_modules).and_then(|cg| cg.generate()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("codegen error: {}", e);
            print_unsupported_ir_hint();
            exit(1);
        }
    };

    emit(&out_path, asm);
}

/// Common single-token clang flags uvclang forwards straight to the front-end:
/// warnings (`-Wall`, `-Wextra`, `-Werror`, `-Wno-...`), the language standard
/// (`-std=c11`, `-ansi`), and feature toggles (`-ffast-math`, `-fno-exceptions`).
/// None of these take a *separate* argument, so forwarding them can't desync arg
/// parsing the way `-o out` or `-D FOO` would; the split-argument flags stay
/// explicitly handled above. Forwarded flags land after uvclang's own canonical
/// flags, so a user flag overrides the matching default.
fn is_clang_passthrough(arg: &str) -> bool
{
    arg == "-w"
        || arg == "-ansi"
        || arg.starts_with("-W")
        || arg.starts_with("-f")
        || arg.starts_with("-std=")
        || arg.starts_with("-pedantic")
}

/// Write `text` to the `-o` path, or to stdout when none was given.
fn emit(out_path: &Option<String>, text: String)
{
    match out_path {
        Some(p) => {
            if let Err(e) = std::fs::write(p, text) {
                eprintln!("could not write {}: {}", p, e);
                exit(1);
            }
        }
        None => print!("{}", text),
    }
}

/// Print guidance shown when uvclang fails to handle a piece of LLVM IR, whether
/// the failure surfaced in the parser or later in codegen. Both cases usually mean
/// the IR uses a construct uvclang does not support yet.
fn print_unsupported_ir_hint()
{
    eprintln!(
        "\nuvclang could not compile this LLVM IR. This usually means the IR uses \
         a construct uvclang does not support yet.\n\
         - If your clang is out of date, please update it and try again.\n\
         - Please also consider opening an issue on the UVM repo so we can add \
         support for it: {}",
        ISSUE_URL
    );
}

fn parse(src: &str, name: &str) -> Result<ast::Module, ParseError>
{
    let lexer = Lexer::new(src, name);
    Parser::new(lexer).parse_module()
}

fn print_stats(path: &str, module: &ast::Module)
{
    let defined = module.functions.iter().filter(|f| !f.is_decl()).count();
    let declared = module.functions.iter().filter(|f| f.is_decl()).count();
    let total_blocks: usize = module.functions.iter().map(|f| f.blocks.len()).sum();
    let total_insts: usize = module
        .functions
        .iter()
        .flat_map(|f| f.blocks.iter())
        .map(|bb| bb.insts.len())
        .sum();

    println!("parsed {}", path);
    println!("  source_filename:    {:?}", module.source_filename);
    println!("  target triple:      {:?}", module.target_triple);
    println!("  struct types:       {}", module.struct_types.len());
    println!("  globals:            {}", module.globals.len());
    println!("  functions defined:  {}", defined);
    println!("  functions declared: {}", declared);
    println!("  basic blocks:       {}", total_blocks);
    println!("  non-term insts:     {}", total_insts);
}
