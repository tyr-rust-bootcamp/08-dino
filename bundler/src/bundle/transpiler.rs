use anyhow::{bail, Result};
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::{FileName, Globals, Mark, SourceMap, GLOBALS};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

pub struct TypeScript;

impl TypeScript {
    /// Compiles TypeScript code into JavaScript.
    pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
        let globals = Globals::default();
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let filename = match filename {
            Some(filename) => FileName::Custom(filename.into()),
            None => FileName::Anon,
        };

        let fm = cm.new_source_file(filename, source.into());

        // Initialize the TypeScript lexer.
        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: true,
                decorators: true,
                no_early_errors: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        let program = match parser
            .parse_program()
            .map_err(|e| e.into_diagnostic(&handler).emit())
        {
            Ok(module) => module,
            Err(_) => bail!("TypeScript compilation failed."),
        };

        // This is where we're gonna store the JavaScript output.
        let mut buffer = vec![];

        GLOBALS.set(&globals, || {
            // Apply the rest SWC transforms to generated code.
            let program = program
                .fold_with(&mut resolver(Mark::new(), Mark::new(), true))
                .fold_with(&mut strip(Mark::new()))
                .fold_with(&mut hygiene())
                .fold_with(&mut fixer(None));

            {
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default(),
                    cm: cm.clone(),
                    comments: None,
                    wr: JsWriter::new(cm, "\n", &mut buffer, None),
                };

                emitter.emit_program(&program).unwrap();
            }
        });

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}
