use zed::lsp::{Completion, CompletionKind};
use zed::{CodeLabel, CodeLabelSpan};
use zed_extension_api::{self as zed, Result};

const OPERATOR_CHAR: [char; 17] = [
    '~', '!', '?', '%', '<', ':', '.', '$', '&', '*', '+', '-', '/', '=', '>', '@', '^',
];

struct ReasonExtension;

impl zed::Extension for ReasonExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = worktree.which("ocamllsp").ok_or_else(|| {
            "ocamllsp (ocaml-language-server) must be installed manually.".to_string()
        })?;

        Ok(zed::Command {
            command: path,
            args: Vec::new(),
            env: worktree.shell_env(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &zed::LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        let name = &completion.label;
        let detail = completion.detail.as_ref().map(|s| s.replace('\n', " "));

        match completion.kind.zip(detail) {
            Some((CompletionKind::Constructor | CompletionKind::EnumMember, detail)) => {
                let (argument, return_t) = detail
                    .split_once("=>")
                    .map_or((None, detail.as_str()), |(arg, typ)| {
                        (Some(arg.trim()), typ.trim())
                    });

                let type_decl = "type t = ";
                let argument = argument.unwrap_or_default();
                let terminator = "\n";
                let let_decl = "let _ ";
                let let_colon = ": ";
                let let_suffix = " = ()";
                let code = format!(
                    "{type_decl}{name}{argument}{terminator}{let_decl}{let_colon}{return_t}{let_suffix}"
                );

                let name_start = type_decl.len();
                let argument_end = name_start + name.len() + argument.len();
                let colon_start = argument_end + terminator.len() + let_decl.len();
                let return_type_end = code.len() - let_suffix.len();
                Some(CodeLabel {
                    code,
                    spans: vec![
                        CodeLabelSpan::code_range(name_start..argument_end),
                        CodeLabelSpan::code_range(colon_start..return_type_end),
                    ],
                    filter_range: (0..name.len()).into(),
                })
            }

            Some((CompletionKind::Field, detail)) => {
                let filter_range_start = if name.starts_with(['~', '?']) { 1 } else { 0 };

                let record_prefix = "type t = { ";
                let record_suffix = "; }";
                let code = format!("{record_prefix}{name} : {detail}{record_suffix}");

                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(
                        record_prefix.len()..code.len() - record_suffix.len(),
                    )],
                    code,
                    filter_range: (filter_range_start..name.len()).into(),
                })
            }

            Some((CompletionKind::Value, detail)) => {
                let let_prefix = "let ";
                let suffix = " = ()";
                let (l_paren, r_paren) = if name.contains(OPERATOR_CHAR) {
                    ("( ", " )")
                } else {
                    ("", "")
                };
                let code = format!("{let_prefix}{l_paren}{name}{r_paren} : {detail}{suffix}");

                let name_start = let_prefix.len() + l_paren.len();
                let name_end = name_start + name.len();
                let type_annotation_start = name_end + r_paren.len();
                let type_annotation_end = code.len() - suffix.len();

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::code_range(name_start..name_end),
                        CodeLabelSpan::code_range(type_annotation_start..type_annotation_end),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }

            Some((CompletionKind::Method, detail)) => {
                let method_decl = "class c : object method ";
                let end = " end";
                let code = format!("{method_decl}{name} : {detail}{end}");

                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(
                        method_decl.len()..code.len() - end.len(),
                    )],
                    code,
                    filter_range: (0..name.len()).into(),
                })
            }

            Some((CompletionKind::TypeParameter, detail)) => {
                let detail_len = detail.len();
                Some(CodeLabel {
                    code: detail,
                    spans: vec![CodeLabelSpan::code_range(0..detail_len)],
                    filter_range: (5..5 + name.len()).into(),
                })
            }

            Some((kind, _)) => {
                let highlight_name = match kind {
                    CompletionKind::Module | CompletionKind::Interface => "title",
                    CompletionKind::Keyword => "keyword",
                    CompletionKind::TypeParameter => "type",
                    _ => return None,
                };

                Some(CodeLabel {
                    spans: vec![(CodeLabelSpan::literal(name, Some(highlight_name.to_string())))],
                    filter_range: (0..name.len()).into(),
                    code: String::new(),
                })
            }
            _ => None,
        }
    }
}

zed::register_extension!(ReasonExtension);
