#![allow(dead_code)]

mod ast;
mod codegen;
mod frontend;
mod layout;
mod lexer;
mod parser;
mod syscalls;

use tempfile::Builder;

use std::io::Write;
use std::process::exit;

use codegen::Codegen;
use frontend::FrontendOpts;
use lexer::{Lexer, ParseError};
use parser::Parser;
use crate::ast::Module;

const USAGE: &str = "usage: uvclang <input.c|.ll> [-o out.asm] [-O0|-O1|-O2|-O3]\n       \
                     [-D<macro>] [-I<dir>] [-U<macro>] [-Wall|-W<warn>] [-std=<std>]\n       \
                     [-f<feature>] [--emit-ir] [--stats] [--rtlib] [-l <lib-path>]";

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


pub enum Source
{
    Snippet(String),
    FilePath(String),
}

pub struct SourceCompilation
{
    source_name: String,
    source : Source,

}

fn ir_source(source : SourceCompilation, fe : FrontendOpts) -> Result<(String, String), String>
{
    if frontend::is_ir_path(&source.source_name) {
        match source.source {
            Source::Snippet(snippet) => Ok((snippet, source.source_name.clone())),
            Source::FilePath(path) => {
                match std::fs::read_to_string(path) {
                    Ok(s) => Ok((s, source.source_name.clone())),
                    Err(e) => Err(e.to_string())
                }
            }
        }
    } else {
        let mut this_fe = fe.clone();
        this_fe.is_cpp = frontend::is_cpp_path(&source.source_name);
        match source.source {
            Source::Snippet(snippet) => {
                let src_file = Builder::new()
                    .suffix(&source.source_name)
                    .tempfile()
                    .and_then(|mut f| {
                        writeln!(f, "{}", snippet)?;
                        Ok(f)
                    })
                    .map_err(|e| e.to_string())?;
                let tmp_file_name = src_file.path().to_str().ok_or("Path Error".to_string())?;
                let ir_result = frontend::compile_to_ir(tmp_file_name, &this_fe)?;
                Ok((ir_result, source.source_name.clone()))
            },
            Source::FilePath(path) => {
                let ir_result = frontend::compile_to_ir(&*path, &this_fe)?;
                Ok((ir_result, source.source_name.clone()))
            }
        }
    }
}
fn compile_source(source : SourceCompilation, fe : FrontendOpts) -> Result<Module, String>
{
    let (ir_str, src_name) = ir_source(source, fe)?;

    std::fs::write(src_name.clone() +".out.ir", ir_str.clone()); // REMOVE THIS!!!
    
    parse(&*ir_str, &*src_name).map_err(|e| e.to_string())
}

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
    let mut build_runtime_lib = false;
    let mut runtime_lib_path: Option<String> = None;
    let mut runtime_src_lib_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--stats" => stats = true,
            "--rtlib" => build_runtime_lib = true,
            "-l" => {
                i += 1;
                runtime_lib_path = args.get(i).cloned();
            }
            "--rtlib_path" => {
                i += 1;
                runtime_src_lib_path = args.get(i).cloned();
            }
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

    // --emit-ir: dump the front-end IR and stop (handy for growing C coverage).
    if emit_ir {
        if inputs.len() != 1 {
            eprintln!(
                "uvclang: --emit-ir only supports a single input file (got {}); \
                 invoke uvclang separately for each file",
                inputs.len()
            );
            exit(2);
        }
        let (ir, _) = match ir_source(
            SourceCompilation {
                source_name: inputs[0].clone(),
                source: Source::FilePath(inputs[0].clone()),
            },
            fe,
        ) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("parse error: {}", e);
                exit(1);
            }
        };
        emit(&out_path, ir);
        return;
    }

    let modules : Result<Vec<ast::Module>, String> = inputs.into_iter()
        .map(|input| {
            compile_source(
                SourceCompilation {
                    source_name: input.clone(),
                    source: Source::FilePath(input.clone()),
                },
                fe.clone(),
            )
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
    let runtime_c = compile_source(
        SourceCompilation {
            source_name: "stdio.c".to_string(),
            source: Source::Snippet(STDIO_C_CONTENT.to_string()),
        }, fe.clone());

    let runtime_new = compile_source(
        SourceCompilation {
            source_name: "new.cpp".to_string(),
            source: Source::Snippet(NEW_OPERATOR_DEF.to_string()),
        }, fe.clone());

    let mut runtime_modules : Vec<ast::Module> = vec![
        runtime_c.expect("REASON"),
        runtime_new.expect("REASON"),
    ];

    if runtime_src_lib_path.is_some() {
        let mut fe_rt_cpp_lib = fe.clone();
        fe_rt_cpp_lib.passthrough.push("-D_LIBCPP_BUILDING_LIBRARY".to_string());
        //_LIBCPP_BUILDING_LIBRARY
        let runtime_string = compile_source(
            SourceCompilation {
                source_name: "string.cpp".to_string(),
                source: Source::FilePath(runtime_src_lib_path.expect("") + "/string.cpp"),
            }, fe_rt_cpp_lib.clone());
        match runtime_string {
            Ok(m) => runtime_modules.push(m),
            Err(e) => {
                eprintln!("module error: {}", e);
                exit(1);
            }
        }
    }



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
