use std::borrow::Cow::{self, Borrowed, Owned};

use radix_trie::{Trie, TrieCommon};
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hint, Hinter, HistoryHinter};
use rustyline::sqlite_history::SQLiteHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyEvent, Result};
use rustyline_derive::{Completer, Helper, Validator};
use tracing_subscriber::EnvFilter;

#[derive(Helper, Completer, Validator)]
struct DIYHinter {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    hints: Trie<String, CommandHint>,
    colored_prompt: String,
    history_hinter: HistoryHinter,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> Self {
        assert!(text.starts_with(complete_up_to));
        Self {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> Self {
        Self {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        // First try to get command hint
        let command_hint = self
            .hints
            .get_raw_descendant(line)
            .and_then(|node| node.value())
            .map(|hint| hint.suffix(pos));

        // If no command hint found, try history hint
        if command_hint.is_none() {
            if let Some(history_hint) = self.history_hinter.hint(line, pos, ctx) {
                return Some(CommandHint {
                    display: history_hint,
                    complete_up_to: 0, // History hints don't auto-complete
                });
            }
        }

        command_hint
    }
}

impl Highlighter for DIYHinter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

fn diy_hints() -> Trie<String, CommandHint> {
    let mut trie = Trie::new();
    let commands = [
        ("help", "help"),
        ("get key", "get "),
        ("set key value", "set "),
        ("hget key field", "hget "),
        ("hset key field value", "hset "),
    ];

    for (text, complete_up_to) in commands {
        trie.insert(text.to_string(), CommandHint::new(text, complete_up_to));
    }
    trie
}

fn main() -> Result<()> {
    let filter = EnvFilter::new(
        "symphonia_format_ogg=off,symphonia_core=off,symphonia_bundle_mp3::demuxer=off,tantivy::directory=off,tantivy::indexer=off,sea_orm_migration::migrator=off,info",
    );

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_test_writer()
        .init();

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .build();

    let history = SQLiteHistory::with_config(config)?;

    let h = DIYHinter {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hints: diy_hints(),
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
        history_hinter: HistoryHinter::new(),
    };

    let mut rl: Editor<DIYHinter, _> = Editor::with_history(config, history)?;
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

    println!("Welcome to the Rune Speaker Command Line Interface");

    let mut count = 1;
    loop {
        let p = format!("{count}> ");
        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                println!("input: {line}");
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Encountered Eof");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
        count += 1;
    }

    Ok(())
}
